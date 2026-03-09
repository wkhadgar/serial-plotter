<script lang="ts">
  import { X, Plus, Trash2, Code, FileCode, AlertCircle, Pencil } from 'lucide-svelte';
  import type {
    PluginKind,
    PluginRuntime,
    SchemaFieldType,
    PluginSchemaField,
    PluginDefinition,
    PluginDependency,
  } from '$lib/types/plugin';
  import {
    PLUGIN_KIND_LABELS,
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
  import { openFileDialog } from '$lib/services/fileDialog';
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

  const runtimeExtension = $derived(runtime === 'python' ? '.py' : '.dll / .so');
  const schemaValid = $derived(formFields.every((f, i) => !fieldErrors[i]));
  const isEditing = $derived(initialPlugin !== null);
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
    }));
  }

  $effect(() => {
    if (forceKind) applyKindPreset(forceKind);
  });

  $effect(() => {
    if (!visible) return;

    if (initialPlugin) {
      pluginName = initialPlugin.name;
      applyKindPreset(forceKind ?? initialPlugin.kind);
      runtime = initialPlugin.runtime;
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
      i === index ? { ...f, type: value, defaultValue: '', defaultBool: false, hasDefault: false } : f
    );
  }

  function updateFieldHasDefault(index: number, value: boolean) {
    formFields = formFields.map((f, i) => (i === index ? { ...f, hasDefault: value } : f));
  }

  function updateFieldDefaultValue(index: number, value: string) {
    formFields = formFields.map((f, i) => (i === index ? { ...f, defaultValue: value } : f));
  }

  function updateFieldDefaultBool(index: number, value: boolean) {
    formFields = formFields.map((f, i) => (i === index ? { ...f, defaultBool: value } : f));
  }

  function buildSchema(): PluginSchemaField[] {
    const pluginKind = resolvedKind;
    const userFields: PluginSchemaField[] = formFields.map((f) => {
      const field: PluginSchemaField = { name: f.name, type: f.type };
      if (f.hasDefault) {
        switch (f.type) {
          case 'bool': field.defaultValue = f.defaultBool; break;
          case 'int': field.defaultValue = parseInt(f.defaultValue, 10) || 0; break;
          case 'float': field.defaultValue = parseFloat(f.defaultValue) || 0; break;
          case 'string': field.defaultValue = f.defaultValue; break;
          case 'list': field.defaultValue = []; break;
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
    const ext = runtime === 'python' ? ['py'] : ['dll', 'so'];
    const result = await openFileDialog({
      title: 'Selecionar Código Fonte',
      filters: [{ name: 'Plugin Source', extensions: ext }],
    });
    if (result) {
      sourceFileName = result.name;
      sourceCode = '';
    }
  }

  function handleOpenCodeEditor() {
    if (!sourceCode && isDriverKind && runtime === 'python') {
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

    if (!pluginName.trim()) {
      error = 'Nome do plugin é obrigatório';
      return;
    }

    if (!resolvedKind) {
      error = 'Defina um tipo de plugin válido';
      return;
    }

    if (!sourceFileName && !sourceCode) {
      error = 'Selecione ou escreva o código fonte';
      return;
    }

    if (sourceCode && resolvedKind === 'driver' && runtime === 'python') {
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
        error = `Campo de schema #${i + 1} precisa de um nome`;
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

    if (!schemaValid) {
      error = 'Corrija os erros nos campos do schema';
      return;
    }

    isLoading = true;

    try {
      const payload = {
        name: pluginName.trim(),
        kind: resolvedKind,
        runtime,
        sourceFile: sourceFileName || (runtime === 'python' ? 'main.py' : 'plugin.rs'),
        sourceCode: sourceCode || undefined,
        schema: buildSchema(),
        dependencies: runtime === 'python' && dependencies.length > 0
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
    class="fixed inset-0 z-[60] flex items-center justify-center bg-black/60"
    onclick={handleClose}
  >
    <div
      class="bg-white dark:bg-[#0c0c0e] rounded-2xl shadow-2xl w-full max-w-2xl max-h-[85vh] flex flex-col overflow-hidden border border-slate-200 dark:border-white/10"
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

      <div class="flex-1 overflow-y-auto p-6 space-y-5">
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
              placeholder="Ex: Modbus TCP Driver"
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

        <div class="grid grid-cols-2 gap-4">
          <label class="block">
            <span class="text-[10px] font-bold text-slate-400 dark:text-zinc-500 uppercase mb-1.5 block">Tipo *</span>
            <select
              value={kindSelectValue}
              onchange={(e) => handleKindSelectChange((e.target as HTMLSelectElement).value)}
              disabled={!!forceKind}
              class="w-full h-10 px-3 rounded-lg border border-slate-200 dark:border-white/10 bg-white dark:bg-[#18181b] text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50 cursor-pointer disabled:opacity-60"
            >
              {#each Object.entries(PLUGIN_KIND_LABELS) as [value, label]}
                <option {value} class="dark:bg-zinc-900">{label}</option>
              {/each}
              <option value={CUSTOM_KIND_VALUE} class="dark:bg-zinc-900">Personalizado</option>
            </select>
          </label>
          <label class="block">
            <span class="text-[10px] font-bold text-slate-400 dark:text-zinc-500 uppercase mb-1.5 block">Runtime *</span>
            <select
              bind:value={runtime}
              class="w-full h-10 px-3 rounded-lg border border-slate-200 dark:border-white/10 bg-white dark:bg-[#18181b] text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50 cursor-pointer"
            >
              {#each Object.entries(PLUGIN_RUNTIME_LABELS) as [value, label]}
                <option {value} class="dark:bg-zinc-900">{label}</option>
              {/each}
            </select>
          </label>
        </div>

        {#if !forceKind && kindMode === 'custom'}
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
          <div class="grid grid-cols-2 gap-2">
            <button
              onclick={handlePickSourceFile}
              class="flex items-center gap-3 p-3 rounded-lg border border-dashed border-slate-300 dark:border-white/10 hover:border-blue-400 dark:hover:border-blue-500 bg-slate-50 dark:bg-white/[0.02] transition-colors text-left"
            >
              <FileCode size={20} class="text-slate-400 shrink-0" />
              {#if sourceFileName && !sourceCode}
                <div class="min-w-0">
                  <div class="text-sm font-medium text-slate-700 dark:text-zinc-300 truncate">{sourceFileName}</div>
                  <div class="text-xs text-slate-400 dark:text-zinc-500">Upload de arquivo</div>
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
                <div class="flex items-center gap-2">
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
                    class="w-28 h-9 px-2 rounded border border-slate-200 dark:border-white/10 bg-white dark:bg-[#18181b] text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50 font-mono"
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
              <span class="text-[10px] font-bold text-slate-400 dark:text-zinc-500 uppercase">Schema de Configuração</span>
              {#if isDriverKind}
                <span class="text-[10px] text-slate-400 dark:text-zinc-500 ml-2">(num_sensors e num_actuators são adicionados automaticamente)</span>
              {/if}
            </div>
            <span class="text-[10px] text-slate-400 dark:text-zinc-500">{formFields.length} campo(s)</span>
          </div>

          <div class="space-y-2">
            {#each formFields as field, i (i)}
              <div class="p-3 rounded-lg border border-slate-200 dark:border-white/10 bg-slate-50 dark:bg-white/[0.02] space-y-2">
                <div class="flex items-start gap-2">
                  <div class="flex-1 grid grid-cols-[1fr_120px] gap-2 items-start">
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
                    {:else if field.type === 'list'}
                      <span class="text-[10px] text-slate-400 dark:text-zinc-500">lista vazia</span>
                    {/if}
                  {:else}
                    <span class="text-[10px] text-amber-600 dark:text-amber-400">obrigatório</span>
                  {/if}
                </div>
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

      <div class="flex items-center justify-between px-6 py-4 border-t border-slate-200 dark:border-white/5 bg-slate-50 dark:bg-white/[0.02] shrink-0">
        <button
          onclick={handleClose}
          class="px-4 py-2 rounded-lg text-sm font-medium text-slate-600 dark:text-zinc-400 hover:bg-slate-200 dark:hover:bg-white/10 transition-colors"
        >
          Cancelar
        </button>
        <button
          onclick={handleSubmit}
          disabled={isLoading}
          class="px-6 py-2 rounded-lg text-sm font-bold bg-blue-600 hover:bg-blue-700 disabled:bg-blue-400 text-white transition-colors flex items-center gap-2"
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
