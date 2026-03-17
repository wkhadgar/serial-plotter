<script lang="ts">
  import { X, Plus, Trash2, Code, FileCode, AlertCircle, Pencil } from 'lucide-svelte';
  import type {
    PluginKind,
    PluginRuntime,
    SchemaFieldType,
    SchemaFieldValue,
    PluginSchemaField,
    PluginDefinition,
    PluginDependency,
  } from '$lib/types/plugin';
  import {
    PLUGIN_KIND_LABELS,
    PLUGIN_CREATION_RUNTIMES,
    PLUGIN_RUNTIME_LABELS,
    SCHEMA_FIELD_TYPE_LABELS,
    getPluginKindLabel,
    isBuiltInPluginKind,
    isValidFieldName,
    AUTO_SCHEMA_FIELDS,
    RESERVED_FIELD_NAMES,
    generateDriverTemplate,
    normalizePluginKind,
    toDriverClassName,
  } from '$lib/types/plugin';
  import { createPlugin, updatePlugin } from '$lib/services/plugin';
  
  async function validateDriverSourceCode(_code: string, _runtime: string): Promise<{ success: boolean; errors?: string[] }> {
    return { success: true };
  }
  import { openFileDialog, readFileAsText } from '$lib/services/fileDialog';
  import CodeEditorModal from './CodeEditorModal.svelte';
  import type { CodeEditorResult } from './CodeEditorModal.svelte';

  interface Props {
    visible: boolean;
    forceKind?: PluginKind;
    initialKind?: PluginKind;
    initialPlugin?: PluginDefinition | null;
    onClose: () => void;
    onPluginCreated: (plugin: PluginDefinition) => void;
  }

  let {
    visible = $bindable(),
    forceKind,
    initialKind,
    initialPlugin = null,
    onClose,
    onPluginCreated,
  }: Props = $props();

  const CUSTOM_KIND_VALUE = '__custom__';

  interface FormField {
    name: string;
    type: SchemaFieldType;
    hasDefault: boolean;
    defaultValue: string;
    defaultBool: boolean;
    defaultListValues: ListDefaultItem[];
  }

  interface ListDefaultItem {
    value: string;
    type: SchemaFieldType;
  }

  let pluginName = $state('');
  let kind = $state<PluginKind>('driver');
  let kindMode = $state<'builtin' | 'custom'>('builtin');
  let customKind = $state('');
  let runtime = $state<PluginRuntime>('python');
  let sourceFileName = $state('');
  let sourceCode = $state('');
  let description = $state('');
  let formFields = $state<FormField[]>([]);
  let dependencies = $state<PluginDependency[]>([]);

  let isLoading = $state(false);
  let error = $state<string | null>(null);
  let fieldErrors = $state<Record<number, string>>({});
  let showCodeEditor = $state(false);
  const LIST_ITEM_TYPE_OPTIONS: SchemaFieldType[] = ['string', 'int', 'float', 'bool', 'list'];

  const runtimeExtension = $derived('.py');
  const schemaValid = $derived(formFields.every((f, i) => !fieldErrors[i]));
  const isEditing = $derived(initialPlugin !== null);
  const isKindLocked = $derived(!!forceKind || isEditing);
  const resolvedKind = $derived.by(() => {
    if (forceKind) return normalizePluginKind(forceKind);
    if (kindMode === 'custom') {
      return normalizePluginKind(customKind);
    }
    return normalizePluginKind(kind);
  });
  const kindSelectValue = $derived(
    forceKind
      ? normalizePluginKind(forceKind)
      : kindMode === 'custom'
        ? CUSTOM_KIND_VALUE
        : kind
  );
  const kindLabel = $derived(
    resolvedKind ? getPluginKindLabel(resolvedKind) : 'Plugin personalizado'
  );
  const isDriverKind = $derived(resolvedKind === 'driver');
  const modalTitle = $derived(isEditing ? 'Editar Plugin' : 'Criar Novo Plugin');
  const modalDescription = $derived(
    isEditing
      ? `Atualize as informações do plugin ${kindLabel}`
      : `Defina um plugin reutilizável do tipo ${kindLabel}`
  );
  const submitLabel = $derived(isEditing ? 'Salvar Alterações' : 'Criar Plugin');

  function applyKindPreset(nextKind?: PluginKind) {
    if (!nextKind) {
      kind = 'driver';
      kindMode = 'builtin';
      customKind = '';
      return;
    }

    const normalized = normalizePluginKind(nextKind);
    if (isBuiltInPluginKind(normalized)) {
      kind = normalized;
      kindMode = 'builtin';
      customKind = '';
      return;
    }

    kind = normalized;
    kindMode = 'custom';
    customKind = normalized;
  }

  function schemaToFormFields(schema: PluginSchemaField[], pluginKind: PluginKind): FormField[] {
    const editableSchema = pluginKind === 'driver'
      ? schema.filter((field) => !AUTO_SCHEMA_FIELDS.some((autoField) => autoField.name === field.name))
      : schema;

    return editableSchema.map((field) => ({
      name: field.name,
      type: field.type,
      hasDefault: field.defaultValue !== undefined,
      defaultValue:
        typeof field.defaultValue === 'number' || typeof field.defaultValue === 'string'
          ? String(field.defaultValue)
          : '',
      defaultBool: field.defaultValue === true,
      defaultListValues: Array.isArray(field.defaultValue)
        ? field.defaultValue.map(toListDefaultItem)
        : [],
    }));
  }

  function toListDefaultItem(value: SchemaFieldValue): ListDefaultItem {
    return {
      value: stringifyListItem(value),
      type: inferListItemType(value),
    };
  }

  function inferListItemType(value: SchemaFieldValue): SchemaFieldType {
    if (Array.isArray(value)) return 'list';
    if (typeof value === 'boolean') return 'bool';
    if (typeof value === 'number') return Number.isInteger(value) ? 'int' : 'float';
    return 'string';
  }

  function stringifyListItem(value: SchemaFieldValue): string {
    if (Array.isArray(value)) {
      return JSON.stringify(value);
    }

    return String(value);
  }

  $effect(() => {
    if (forceKind) applyKindPreset(forceKind);
  });

  $effect(() => {
    if (!visible) return;

    if (initialPlugin) {
      pluginName = initialPlugin.name;
      applyKindPreset(forceKind ?? initialPlugin.kind);
      runtime = 'python';
      sourceFileName = initialPlugin.sourceFile;
      sourceCode = initialPlugin.sourceCode ?? '';
      description = initialPlugin.description ?? '';
      formFields = schemaToFormFields(initialPlugin.schema, initialPlugin.kind);
      dependencies = [...(initialPlugin.dependencies ?? [])];
      error = null;
      fieldErrors = {};
      return;
    }

    resetForm();
  });

  function handleKindSelectChange(value: string) {
    if (isKindLocked) return;

    if (value === CUSTOM_KIND_VALUE) {
      kindMode = 'custom';
      if (!customKind.trim()) {
        customKind = initialKind && !isBuiltInPluginKind(normalizePluginKind(initialKind))
          ? normalizePluginKind(initialKind)
          : '';
      }
      return;
    }

    kindMode = 'builtin';
    kind = value as PluginKind;
  }

  function updateCustomKind(value: string) {
    if (isKindLocked) return;
    customKind = value;
  }

  function addSchemaField() {
    formFields = [
      ...formFields,
      {
        name: '',
        type: 'string',
        hasDefault: false,
        defaultValue: '',
        defaultBool: false,
        defaultListValues: [],
      },
    ];
  }

  function removeSchemaField(index: number) {
    formFields = formFields.filter((_, i) => i !== index);
    const newErrors = { ...fieldErrors };
    delete newErrors[index];
    fieldErrors = newErrors;
  }

  function updateFieldName(index: number, value: string) {
    formFields = formFields.map((f, i) => (i === index ? { ...f, name: value } : f));
    const newErrors = { ...fieldErrors };
    if (value && !isValidFieldName(value)) {
      newErrors[index] = 'Apenas letras, números e _';
    } else if (value && RESERVED_FIELD_NAMES.includes(value)) {
      newErrors[index] = 'Nome reservado';
    } else if (value && formFields.some((f, i) => i !== index && f.name === value)) {
      newErrors[index] = 'Nome duplicado';
    } else {
      delete newErrors[index];
    }
    fieldErrors = newErrors;
  }

  function updateFieldType(index: number, value: SchemaFieldType) {
    formFields = formFields.map((f, i) =>
      i === index
        ? { ...f, type: value, defaultValue: '', defaultBool: false, defaultListValues: [], hasDefault: false }
        : f
    );
  }

  function updateFieldHasDefault(index: number, value: boolean) {
    formFields = formFields.map((f, i) => {
      if (i !== index) return f;

      if (f.type === 'list' && value && f.defaultListValues.length === 0) {
        return { ...f, hasDefault: value, defaultListValues: [createEmptyListDefaultItem()] };
      }

      return { ...f, hasDefault: value };
    });
  }

  function updateFieldDefaultValue(index: number, value: string) {
    formFields = formFields.map((f, i) => (i === index ? { ...f, defaultValue: value } : f));
  }

  function updateFieldDefaultBool(index: number, value: boolean) {
    formFields = formFields.map((f, i) => (i === index ? { ...f, defaultBool: value } : f));
  }

  function createEmptyListDefaultItem(): ListDefaultItem {
    return { value: '', type: 'string' };
  }

  function updateListDefaultItemValue(fieldIndex: number, itemIndex: number, value: string) {
    formFields = formFields.map((field, currentIndex) => {
      if (currentIndex !== fieldIndex) return field;

      const nextValues = field.defaultListValues.map((entry, currentItemIndex) =>
        currentItemIndex === itemIndex ? { ...entry, value } : entry
      );

      return { ...field, defaultListValues: nextValues };
    });
  }

  function formatListDefaultItemValue(rawValue: string, type: SchemaFieldType): string {
    const trimmed = rawValue.trim();

    switch (type) {
      case 'bool':
        return trimmed.toLowerCase() === 'true' ? 'true' : 'false';
      case 'int': {
        if (!trimmed) return '';
        const parsed = Number.parseInt(trimmed, 10);
        return Number.isFinite(parsed) ? String(parsed) : '';
      }
      case 'float': {
        if (!trimmed) return '';
        const parsed = Number.parseFloat(trimmed);
        return Number.isFinite(parsed) ? String(parsed) : '';
      }
      case 'list': {
        if (!trimmed) return '[]';
        try {
          const parsed = JSON.parse(trimmed);
          return Array.isArray(parsed) ? JSON.stringify(parsed) : rawValue;
        } catch {
          return rawValue;
        }
      }
      case 'string':
      default:
        return rawValue;
    }
  }

  function formatListDefaultItemByType(fieldIndex: number, itemIndex: number) {
    formFields = formFields.map((field, currentIndex) => {
      if (currentIndex !== fieldIndex) return field;

      const nextValues = field.defaultListValues.map((entry, currentItemIndex) => {
        if (currentItemIndex !== itemIndex) return entry;

        return {
          ...entry,
          value: formatListDefaultItemValue(entry.value, entry.type),
        };
      });

      return { ...field, defaultListValues: nextValues };
    });
  }

  function updateListDefaultItemType(fieldIndex: number, itemIndex: number, value: SchemaFieldType) {
    formFields = formFields.map((field, currentIndex) => {
      if (currentIndex !== fieldIndex) return field;

      const nextValues = field.defaultListValues.map((entry, currentItemIndex) =>
        currentItemIndex === itemIndex
          ? { ...entry, type: value, value: formatListDefaultItemValue(entry.value, value) }
          : entry
      );

      return { ...field, defaultListValues: nextValues };
    });
  }

  function addListDefaultItem(fieldIndex: number) {
    formFields = formFields.map((field, currentIndex) => {
      if (currentIndex !== fieldIndex) return field;
      return { ...field, defaultListValues: [...field.defaultListValues, createEmptyListDefaultItem()] };
    });
  }

  function removeListDefaultItem(fieldIndex: number, itemIndex: number) {
    formFields = formFields.map((field, currentIndex) => {
      if (currentIndex !== fieldIndex) return field;
      return {
        ...field,
        defaultListValues: field.defaultListValues.filter((_, currentItemIndex) => currentItemIndex !== itemIndex),
      };
    });
  }

  function normalizeUnknownToSchemaValue(value: unknown): SchemaFieldValue {
    if (Array.isArray(value)) {
      return value.map(normalizeUnknownToSchemaValue);
    }

    if (
      typeof value === 'string' ||
      typeof value === 'number' ||
      typeof value === 'boolean'
    ) {
      return value;
    }

    return String(value ?? '');
  }

  function parseListDefaultItem(item: ListDefaultItem): SchemaFieldValue {
    const trimmed = item.value.trim();

    switch (item.type) {
      case 'bool':
        return trimmed.toLowerCase() === 'true';
      case 'int':
        return Number.parseInt(trimmed, 10) || 0;
      case 'float':
        return Number.parseFloat(trimmed) || 0;
      case 'list':
        try {
          const parsed = JSON.parse(trimmed);
          return Array.isArray(parsed) ? parsed.map(normalizeUnknownToSchemaValue) : [];
        } catch {
          return [];
        }
      case 'string':
      default:
        return item.value;
    }
  }

  function getListDefaultItemTypeError(item: ListDefaultItem): string | null {
    const trimmed = item.value.trim();

    switch (item.type) {
      case 'bool':
        return trimmed === 'true' || trimmed === 'false'
          ? null
          : 'Use true ou false';
      case 'int':
        return /^-?\d+$/.test(trimmed) ? null : 'Inteiro inválido';
      case 'float': {
        const parsed = Number.parseFloat(trimmed);
        return Number.isFinite(parsed) ? null : 'Decimal inválido';
      }
      case 'list':
        try {
          const parsed = JSON.parse(trimmed || '[]');
          return Array.isArray(parsed) ? null : 'Use um JSON de array';
        } catch {
          return 'JSON inválido para lista';
        }
      case 'string':
      default:
        return null;
    }
  }

  function validateListDefaults(): string | null {
    for (let fieldIndex = 0; fieldIndex < formFields.length; fieldIndex++) {
      const field = formFields[fieldIndex];
      if (!field.hasDefault || field.type !== 'list') continue;

      for (let itemIndex = 0; itemIndex < field.defaultListValues.length; itemIndex++) {
        const item = field.defaultListValues[itemIndex];
        const itemError = getListDefaultItemTypeError(item);
        if (itemError) {
          const fieldLabel = field.name.trim() || `#${fieldIndex + 1}`;
          return `Campo "${fieldLabel}" item ${itemIndex + 1}: ${itemError}`;
        }
      }
    }

    return null;
  }

  function buildSchema(pluginKind: PluginKind): PluginSchemaField[] {
    const userFields: PluginSchemaField[] = formFields.map((f) => {
      const field: PluginSchemaField = { name: f.name, type: f.type };
      if (f.hasDefault) {
        switch (f.type) {
          case 'bool': field.defaultValue = f.defaultBool; break;
          case 'int': field.defaultValue = parseInt(f.defaultValue, 10) || 0; break;
          case 'float': field.defaultValue = parseFloat(f.defaultValue) || 0; break;
          case 'string': field.defaultValue = f.defaultValue; break;
          case 'list': field.defaultValue = f.defaultListValues.map(parseListDefaultItem); break;
        }
      }
      return field;
    });
    if (pluginKind === 'driver') {
      return [...AUTO_SCHEMA_FIELDS, ...userFields];
    }
    return userFields;
  }

  async function handlePickSourceFile() {
    const result = await openFileDialog({
      title: 'Selecionar Código Fonte',
      filters: [{ name: 'Python Source', extensions: ['py'] }],
    });
    if (result) {
      sourceFileName = result.name;
      sourceCode = await readFileAsText(result.file);
    }
  }

  function handleOpenCodeEditor() {
    if (!sourceCode && !isEditing && isDriverKind && runtime === 'python') {
      sourceCode = generateDriverTemplate(pluginName);
    }
    showCodeEditor = true;
  }

  function handleCodeEditorSave(result: CodeEditorResult) {
    sourceFileName = result.fileName;
    sourceCode = result.code;
    showCodeEditor = false;
  }

  function addDependency() {
    dependencies = [...dependencies, { name: '', version: '' }];
  }

  function removeDependency(index: number) {
    dependencies = dependencies.filter((_, i) => i !== index);
  }

  function updateDependencyName(index: number, value: string) {
    dependencies = dependencies.map((d, i) => (i === index ? { ...d, name: value } : d));
  }

  function updateDependencyVersion(index: number, value: string) {
    dependencies = dependencies.map((d, i) => (i === index ? { ...d, version: value } : d));
  }

  async function handleSubmit() {
    error = null;
    const effectiveKind = initialPlugin
      ? normalizePluginKind(initialPlugin.kind)
      : resolvedKind;

    if (!pluginName.trim()) {
      error = 'Nome do plugin é obrigatório';
      return;
    }

    if (!effectiveKind) {
      error = 'Defina um tipo de plugin válido';
      return;
    }

    if (!sourceFileName && !sourceCode) {
      error = 'Selecione ou escreva o código fonte';
      return;
    }

    if (sourceCode && effectiveKind === 'driver' && runtime === 'python') {
      const expectedClass = toDriverClassName(pluginName);
      const codeValidation = await validateDriverSourceCode(sourceCode, expectedClass);
      if (!codeValidation.success && codeValidation.errors) {
        error = codeValidation.errors.join('; ');
        return;
      }
    }

    if (runtime === 'python') {
      for (let i = 0; i < dependencies.length; i++) {
        if (!dependencies[i].name.trim()) {
          error = `Dependência #${i + 1} precisa de um nome`;
          return;
        }
      }
    }

    for (let i = 0; i < formFields.length; i++) {
      if (!formFields[i].name.trim()) {
        error = `Campo #${i + 1} precisa de um nome`;
        return;
      }
      if (!isValidFieldName(formFields[i].name)) {
        error = `Campo "${formFields[i].name}" contém caracteres inválidos`;
        return;
      }
      if (RESERVED_FIELD_NAMES.includes(formFields[i].name)) {
        error = `Campo "${formFields[i].name}" é um nome reservado`;
        return;
      }
    }

    const listValidationError = validateListDefaults();
    if (listValidationError) {
      error = listValidationError;
      return;
    }

    if (!schemaValid) {
      error = 'Corrija os erros dos campos de configuração';
      return;
    }

    isLoading = true;

    try {
      const payload = {
        name: pluginName.trim(),
        kind: effectiveKind,
        runtime: 'python' as PluginRuntime,
        sourceFile: sourceFileName || 'main.py',
        sourceCode: sourceCode || undefined,
        schema: buildSchema(effectiveKind),
        dependencies: dependencies.length > 0
          ? dependencies.filter((dependency) => dependency.name.trim())
          : undefined,
        description: description.trim() || undefined,
      };

      const finalResponse = initialPlugin
        ? await updatePlugin({
            ...initialPlugin,
            ...payload,
            dependencies: payload.dependencies ?? [],
          })
        : await createPlugin(payload);

      if (finalResponse.success && finalResponse.plugin) {
        onPluginCreated(finalResponse.plugin);
        resetForm();
        onClose();
      } else {
        error = finalResponse.error || 'Erro ao registrar plugin';
      }
    } catch (e) {
      error = e instanceof Error ? e.message : 'Erro inesperado';
    } finally {
      isLoading = false;
    }
  }

  function resetForm() {
    pluginName = '';
    applyKindPreset(forceKind ?? initialKind ?? 'driver');
    runtime = 'python';
    sourceFileName = '';
    sourceCode = '';
    description = '';
    formFields = [];
    dependencies = [];
    error = null;
    fieldErrors = {};
  }

  function handleClose() {
    resetForm();
    onClose();
  }
</script>

{#if visible}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed inset-0 z-[60] flex items-start justify-center overflow-y-auto bg-black/60 p-4 sm:items-center sm:p-6"
    onclick={handleClose}
  >
    <div
      class="my-4 flex max-h-[85vh] w-full max-w-3xl flex-col overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-2xl dark:border-white/10 dark:bg-[#0c0c0e] sm:my-0"
      onclick={(e) => e.stopPropagation()}
    >
      <div class="flex items-center justify-between px-6 py-4 border-b border-slate-200 dark:border-white/5 shrink-0">
        <div>
          <h2 class="text-lg font-bold text-slate-800 dark:text-white">{modalTitle}</h2>
          <p class="text-xs text-slate-500 dark:text-zinc-400 mt-0.5">
            {modalDescription}
          </p>
        </div>
        <button
          onclick={handleClose}
          class="p-2 rounded-lg hover:bg-slate-100 dark:hover:bg-white/5 text-slate-400 transition-colors"
        >
          <X size={20} />
        </button>
      </div>

      <div class="flex-1 space-y-5 overflow-y-auto p-5 sm:p-6">
        {#if error}
          <div class="p-3 rounded-lg bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-900/50 text-red-700 dark:text-red-400 text-sm flex items-center gap-2">
            <AlertCircle size={16} class="shrink-0" />
            {error}
          </div>
        {/if}

        <div class="grid grid-cols-1 gap-4">
          <label class="block">
            <span class="text-[10px] font-bold text-slate-400 dark:text-zinc-500 uppercase mb-1.5 block">Nome do Plugin *</span>
            <input
              type="text"
              bind:value={pluginName}
              placeholder="Ex: Conexão Modbus TCP"
              class="w-full h-10 px-3 rounded-lg border border-slate-200 dark:border-white/10 bg-white dark:bg-[#18181b] text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50"
            />
          </label>
          <label class="block">
            <span class="text-[10px] font-bold text-slate-400 dark:text-zinc-500 uppercase mb-1.5 block">Descrição</span>
            <input
              type="text"
              bind:value={description}
              placeholder="Descrição opcional"
              class="w-full h-10 px-3 rounded-lg border border-slate-200 dark:border-white/10 bg-white dark:bg-[#18181b] text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50"
            />
          </label>
        </div>

        <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
          <label class="block">
            <span class="text-[10px] font-bold text-slate-400 dark:text-zinc-500 uppercase mb-1.5 block">Tipo *</span>
            <select
              value={kindSelectValue}
              onchange={(e) => handleKindSelectChange((e.target as HTMLSelectElement).value)}
              disabled={isKindLocked}
              class="w-full h-10 px-3 rounded-lg border border-slate-200 dark:border-white/10 bg-white dark:bg-[#18181b] text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50 cursor-pointer disabled:opacity-60"
            >
              {#each Object.entries(PLUGIN_KIND_LABELS) as [value, label]}
                <option {value} class="dark:bg-zinc-900">{label}</option>
              {/each}
              <option value={CUSTOM_KIND_VALUE} class="dark:bg-zinc-900">Personalizado</option>
            </select>
          </label>
          <div class="block">
            <span class="text-[10px] font-bold text-slate-400 dark:text-zinc-500 uppercase mb-1.5 block">Linguagem *</span>
            <div class="flex h-10 items-center rounded-lg border border-slate-200 bg-slate-50 px-3 text-sm text-slate-600 dark:border-white/10 dark:bg-[#18181b] dark:text-zinc-300">
              {PLUGIN_RUNTIME_LABELS[PLUGIN_CREATION_RUNTIMES[0]]}
            </div>
          </div>
        </div>

        {#if !isKindLocked && kindMode === 'custom'}
          <label class="block">
            <span class="text-[10px] font-bold text-slate-400 dark:text-zinc-500 uppercase mb-1.5 block">Tipo Personalizado *</span>
            <input
              type="text"
              value={customKind}
              oninput={(e) => updateCustomKind((e.target as HTMLInputElement).value)}
              placeholder="Ex: estimator, parser, monitor"
              class="w-full h-10 px-3 rounded-lg border border-slate-200 dark:border-white/10 bg-white dark:bg-[#18181b] text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50 font-mono"
            />
            <p class="mt-1 text-[11px] text-slate-400 dark:text-zinc-500">
              O identificador final será normalizado para snake_case.
            </p>
          </label>
        {/if}

        <div>
          <span class="text-[10px] font-bold text-slate-400 dark:text-zinc-500 uppercase mb-1.5 block">Código Fonte * ({runtimeExtension})</span>
          <div class="grid grid-cols-1 gap-2 sm:grid-cols-2">
            <button
              onclick={handlePickSourceFile}
              class="flex items-center gap-3 p-3 rounded-lg border border-dashed border-slate-300 dark:border-white/10 hover:border-blue-400 dark:hover:border-blue-500 bg-slate-50 dark:bg-white/[0.02] transition-colors text-left"
            >
              <FileCode size={20} class="text-slate-400 shrink-0" />
              {#if sourceFileName}
                <div class="min-w-0">
                  <div class="text-sm font-medium text-slate-700 dark:text-zinc-300 truncate">{sourceFileName}</div>
                  <div class="text-xs text-slate-400 dark:text-zinc-500">
                    {sourceCode ? `${sourceCode.split('\n').length} linhas carregadas` : 'Upload de arquivo'}
                  </div>
                </div>
              {:else}
                <div class="text-sm text-slate-500 dark:text-zinc-400">Upload de arquivo</div>
              {/if}
            </button>
            <button
              onclick={handleOpenCodeEditor}
              class="flex items-center gap-3 p-3 rounded-lg border border-dashed border-slate-300 dark:border-white/10 hover:border-blue-400 dark:hover:border-blue-500 bg-slate-50 dark:bg-white/[0.02] transition-colors text-left"
            >
              <Pencil size={20} class="text-slate-400 shrink-0" />
              {#if sourceCode}
                <div class="min-w-0">
                  <div class="text-sm font-medium text-blue-600 dark:text-blue-400 truncate">{sourceFileName || 'código inline'}</div>
                  <div class="text-xs text-slate-400 dark:text-zinc-500">{sourceCode.split('\n').length} linhas</div>
                </div>
              {:else}
                <div class="text-sm text-slate-500 dark:text-zinc-400">Editor de código</div>
              {/if}
            </button>
          </div>
        </div>

        {#if runtime === 'python'}
          <div>
            <div class="flex items-center justify-between mb-2">
              <span class="text-[10px] font-bold text-slate-400 dark:text-zinc-500 uppercase">Dependências Python (pip)</span>
              <span class="text-[10px] text-slate-400 dark:text-zinc-500">{dependencies.length} pacote(s)</span>
            </div>

            <div class="space-y-1.5">
              {#each dependencies as dep, i (i)}
                <div class="flex flex-col items-stretch gap-2 sm:flex-row sm:items-center">
                  <input
                    type="text"
                    value={dep.name}
                    oninput={(e) => updateDependencyName(i, (e.target as HTMLInputElement).value)}
                    placeholder="nome-do-pacote"
                    class="flex-1 h-9 px-2 rounded border border-slate-200 dark:border-white/10 bg-white dark:bg-[#18181b] text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50 font-mono"
                  />
                  <input
                    type="text"
                    value={dep.version}
                    oninput={(e) => updateDependencyVersion(i, (e.target as HTMLInputElement).value)}
                    placeholder=">=1.0.0"
                    class="h-9 w-full rounded border border-slate-200 bg-white px-2 text-sm font-mono focus:outline-none focus:ring-2 focus:ring-blue-500/50 dark:border-white/10 dark:bg-[#18181b] sm:w-28"
                  />
                  <button
                    onclick={() => removeDependency(i)}
                    class="p-1.5 rounded hover:bg-red-100 dark:hover:bg-red-900/30 text-slate-400 hover:text-red-500 transition-colors shrink-0"
                  >
                    <Trash2 size={14} />
                  </button>
                </div>
              {/each}
            </div>

            <button
              onclick={addDependency}
              class="w-full mt-2 p-2 rounded-lg border-2 border-dashed border-slate-200 dark:border-white/10 hover:border-blue-400 dark:hover:border-blue-500 transition-colors text-slate-500 dark:text-zinc-400 hover:text-blue-600 dark:hover:text-blue-400 flex items-center justify-center gap-2 text-xs"
            >
              <Plus size={14} />
              Adicionar Dependência
            </button>
          </div>
        {/if}

        <div>
          <div class="flex items-center justify-between mb-2">
            <div>
              <span class="text-[10px] font-bold text-slate-400 dark:text-zinc-500 uppercase">Campos de Configuração</span>
              {#if isDriverKind}
                <span class="text-[10px] text-slate-400 dark:text-zinc-500 ml-2">(sensores e atuadores são preenchidos automaticamente)</span>
              {/if}
            </div>
            <span class="text-[10px] text-slate-400 dark:text-zinc-500">{formFields.length} campo(s)</span>
          </div>

          <div class="space-y-2">
            {#each formFields as field, i (i)}
              <div class="p-3 rounded-lg border border-slate-200 dark:border-white/10 bg-slate-50 dark:bg-white/[0.02] space-y-2">
                <div class="flex items-start gap-2">
                  <div class="flex-1 grid grid-cols-1 gap-2 items-start sm:grid-cols-[1fr_140px]">
                    <div>
                      <input
                        type="text"
                        value={field.name}
                        oninput={(e) => updateFieldName(i, (e.target as HTMLInputElement).value)}
                        placeholder="nome_campo"
                        class="w-full h-9 px-2 rounded border border-slate-200 dark:border-white/10 bg-white dark:bg-[#18181b] text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50 font-mono {fieldErrors[i] ? 'border-red-400 dark:border-red-500' : ''}"
                      />
                      {#if fieldErrors[i]}
                        <p class="text-[10px] text-red-500 mt-0.5">{fieldErrors[i]}</p>
                      {/if}
                    </div>
                    <select
                      value={field.type}
                      onchange={(e) => updateFieldType(i, (e.target as HTMLSelectElement).value as SchemaFieldType)}
                      class="h-9 px-2 rounded border border-slate-200 dark:border-white/10 bg-white dark:bg-[#18181b] text-sm focus:outline-none cursor-pointer"
                    >
                      {#each Object.entries(SCHEMA_FIELD_TYPE_LABELS) as [value, label]}
                        <option {value} class="dark:bg-zinc-900">{label}</option>
                      {/each}
                    </select>
                  </div>
                  <button
                    onclick={() => removeSchemaField(i)}
                    class="p-1.5 rounded hover:bg-red-100 dark:hover:bg-red-900/30 text-slate-400 hover:text-red-500 transition-colors shrink-0 mt-0.5"
                  >
                    <Trash2 size={14} />
                  </button>
                </div>
                {#if field.hasDefault && field.type === 'list'}
                  <div class="space-y-2">
                    <div class="flex items-center justify-between gap-2">
                      <label class="flex items-center gap-1.5 cursor-pointer select-none">
                        <input
                          type="checkbox"
                          checked={field.hasDefault}
                          onchange={(e) => updateFieldHasDefault(i, (e.target as HTMLInputElement).checked)}
                          class="w-3.5 h-3.5 rounded border-slate-300 dark:border-white/20 text-blue-600 focus:ring-blue-500"
                        />
                        <span class="text-[10px] text-slate-500 dark:text-zinc-400 whitespace-nowrap">Valor padrão</span>
                      </label>
                      <span class="text-[10px] text-slate-400 dark:text-zinc-500">
                        {field.defaultListValues.length} item(ns)
                      </span>
                    </div>

                    <div class="ml-5 p-2 rounded-md border border-slate-200 dark:border-white/10 bg-white/70 dark:bg-[#18181b]/60 space-y-1.5">
                      {#if field.defaultListValues.length === 0}
                        <p class="text-[10px] text-slate-400 dark:text-zinc-500">Nenhum item na lista</p>
                      {/if}
                      <div class="space-y-1.5">
                        {#each field.defaultListValues as item, itemIndex (itemIndex)}
                          {@const itemError = getListDefaultItemTypeError(item)}
                          <div class="flex items-start gap-2">
                            <div class="flex-1 space-y-1">
                              {#if item.type === 'bool'}
                                <select
                                  value={item.value || 'false'}
                                  onchange={(e) => {
                                    updateListDefaultItemValue(i, itemIndex, (e.target as HTMLSelectElement).value);
                                    formatListDefaultItemByType(i, itemIndex);
                                  }}
                                  class="w-full h-8 px-2 rounded border border-slate-200 dark:border-white/10 bg-white dark:bg-[#18181b] text-xs focus:outline-none focus:ring-2 focus:ring-blue-500/50"
                                >
                                  <option value="true" class="dark:bg-zinc-900">true</option>
                                  <option value="false" class="dark:bg-zinc-900">false</option>
                                </select>
                              {:else if item.type === 'int' || item.type === 'float'}
                                <input
                                  type="number"
                                  step={item.type === 'float' ? 'any' : '1'}
                                  value={item.value}
                                  oninput={(e) => updateListDefaultItemValue(i, itemIndex, (e.target as HTMLInputElement).value)}
                                  onblur={() => formatListDefaultItemByType(i, itemIndex)}
                                  placeholder={item.type === 'float' ? '0.0' : '0'}
                                  class="w-full h-8 px-2 rounded border border-slate-200 dark:border-white/10 bg-white dark:bg-[#18181b] text-xs focus:outline-none focus:ring-2 focus:ring-blue-500/50 font-mono"
                                />
                              {:else if item.type === 'list'}
                                <textarea
                                  value={item.value}
                                  oninput={(e) => updateListDefaultItemValue(i, itemIndex, (e.target as HTMLTextAreaElement).value)}
                                  onblur={() => formatListDefaultItemByType(i, itemIndex)}
                                  placeholder='["item1", 2]'
                                  rows={2}
                                  class="w-full px-2 py-1.5 rounded border border-slate-200 dark:border-white/10 bg-white dark:bg-[#18181b] text-xs leading-relaxed focus:outline-none focus:ring-2 focus:ring-blue-500/50 font-mono resize-y"
                                ></textarea>
                              {:else}
                                <input
                                  type="text"
                                  value={item.value}
                                  oninput={(e) => updateListDefaultItemValue(i, itemIndex, (e.target as HTMLInputElement).value)}
                                  onblur={() => formatListDefaultItemByType(i, itemIndex)}
                                  placeholder={`item_${itemIndex + 1}`}
                                  class="w-full h-8 px-2 rounded border border-slate-200 dark:border-white/10 bg-white dark:bg-[#18181b] text-xs focus:outline-none focus:ring-2 focus:ring-blue-500/50 font-mono"
                                />
                              {/if}
                              {#if itemError}
                                <p class="text-[10px] text-red-500">{itemError}</p>
                              {/if}
                            </div>
                            <select
                              value={item.type}
                              onchange={(e) => updateListDefaultItemType(i, itemIndex, (e.target as HTMLSelectElement).value as SchemaFieldType)}
                              class="h-8 w-24 px-1.5 rounded border border-slate-200 dark:border-white/10 bg-white dark:bg-[#18181b] text-xs focus:outline-none cursor-pointer"
                            >
                              {#each LIST_ITEM_TYPE_OPTIONS as itemType}
                                <option value={itemType} class="dark:bg-zinc-900">
                                  {SCHEMA_FIELD_TYPE_LABELS[itemType]}
                                </option>
                              {/each}
                            </select>
                            <button
                              onclick={() => removeListDefaultItem(i, itemIndex)}
                              class="p-1.5 rounded hover:bg-red-100 dark:hover:bg-red-900/30 text-slate-400 hover:text-red-500 transition-colors shrink-0 mt-1"
                              title="Remover item"
                            >
                              <Trash2 size={12} />
                            </button>
                          </div>
                        {/each}
                      </div>
                      <button
                        onclick={() => addListDefaultItem(i)}
                        class="h-7 px-2.5 rounded border border-dashed border-slate-300 dark:border-white/10 hover:border-blue-400 dark:hover:border-blue-500 text-[11px] text-slate-500 dark:text-zinc-400 hover:text-blue-600 dark:hover:text-blue-400 flex items-center gap-1.5"
                      >
                        <Plus size={12} />
                        Adicionar item
                      </button>
                    </div>
                  </div>
                {:else}
                  <div class="flex items-center gap-2">
                    <label class="flex items-center gap-1.5 cursor-pointer select-none shrink-0">
                      <input
                        type="checkbox"
                        checked={field.hasDefault}
                        onchange={(e) => updateFieldHasDefault(i, (e.target as HTMLInputElement).checked)}
                        class="w-3.5 h-3.5 rounded border-slate-300 dark:border-white/20 text-blue-600 focus:ring-blue-500"
                      />
                      <span class="text-[10px] text-slate-500 dark:text-zinc-400 whitespace-nowrap">Valor padrão</span>
                    </label>
                    {#if field.hasDefault}
                      {#if field.type === 'bool'}
                        <button
                          onclick={() => updateFieldDefaultBool(i, !field.defaultBool)}
                          class="flex items-center gap-2 h-8 px-2 rounded border border-slate-200 dark:border-white/10 bg-white dark:bg-[#18181b] text-xs"
                        >
                          <div class="w-7 h-3.5 rounded-full transition-colors relative {field.defaultBool ? 'bg-blue-500' : 'bg-slate-300 dark:bg-zinc-600'}">
                            <div class="absolute top-0.5 w-2.5 h-2.5 rounded-full bg-white shadow transition-transform {field.defaultBool ? 'translate-x-3.5' : 'translate-x-0.5'}"></div>
                          </div>
                          <span class="text-slate-500 dark:text-zinc-400">{field.defaultBool ? 'true' : 'false'}</span>
                        </button>
                      {:else if field.type === 'int' || field.type === 'float'}
                        <input
                          type="number"
                          step={field.type === 'float' ? 'any' : '1'}
                          value={field.defaultValue}
                          oninput={(e) => updateFieldDefaultValue(i, (e.target as HTMLInputElement).value)}
                          placeholder="0"
                          class="flex-1 h-8 px-2 rounded border border-slate-200 dark:border-white/10 bg-white dark:bg-[#18181b] text-xs focus:outline-none focus:ring-2 focus:ring-blue-500/50"
                        />
                      {:else if field.type === 'string'}
                        <input
                          type="text"
                          value={field.defaultValue}
                          oninput={(e) => updateFieldDefaultValue(i, (e.target as HTMLInputElement).value)}
                          placeholder="valor padrão"
                          class="flex-1 h-8 px-2 rounded border border-slate-200 dark:border-white/10 bg-white dark:bg-[#18181b] text-xs focus:outline-none focus:ring-2 focus:ring-blue-500/50"
                        />
                      {/if}
                    {:else}
                      <span class="text-[10px] text-amber-600 dark:text-amber-400">obrigatório</span>
                    {/if}
                  </div>
                {/if}
              </div>
            {/each}
          </div>

          <button
            onclick={addSchemaField}
            class="w-full mt-2 p-2.5 rounded-lg border-2 border-dashed border-slate-200 dark:border-white/10 hover:border-blue-400 dark:hover:border-blue-500 transition-colors text-slate-500 dark:text-zinc-400 hover:text-blue-600 dark:hover:text-blue-400 flex items-center justify-center gap-2 text-sm"
          >
            <Plus size={16} />
            Adicionar Campo
          </button>
        </div>
      </div>

      <div class="flex flex-wrap items-center justify-between gap-2 border-t border-slate-200 bg-slate-50 px-5 py-4 dark:border-white/5 dark:bg-white/[0.02] sm:px-6 shrink-0">
        <button
          onclick={handleClose}
          class="rounded-lg px-4 py-2 text-sm font-medium text-slate-600 transition-colors hover:bg-slate-200 dark:text-zinc-400 dark:hover:bg-white/10"
        >
          Cancelar
        </button>
        <button
          onclick={handleSubmit}
          disabled={isLoading}
          class="flex min-w-[170px] items-center justify-center gap-2 rounded-lg bg-blue-600 px-6 py-2 text-sm font-bold text-white transition-colors hover:bg-blue-700 disabled:bg-blue-400"
        >
          {#if isLoading}
            <div class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
            Criando...
          {:else}
            <Code size={16} />
            {submitLabel}
          {/if}
        </button>
      </div>
    </div>
  </div>

  <CodeEditorModal
    bind:visible={showCodeEditor}
    initialCode={sourceCode}
    initialFileName={sourceFileName}
    title="Editor de Código — Python"
    onClose={() => { showCodeEditor = false; }}
    onSave={handleCodeEditorSave}
  />
{/if}
