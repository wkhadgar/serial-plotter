<script lang="ts">
  /**
   * ============================================================================
   * PLUGIN INSTANCE CONFIG MODAL - Configuração de Instância de Plugin
   * ============================================================================
   *
   * Modal modular para configurar uma instância de plugin para uma planta.
   * Renderiza cada campo do schema com o tipo correto:
   * - bool   → toggle switch (on/off)
   * - int    → input numérico (inteiro)
   * - float  → input numérico (decimal)
   * - string → input de texto
   * - list   → array builder com add/remove itens
   *
   * Reutilizável em qualquer contexto.
   */
  import { X, Plus, Trash2, Settings, AlertCircle, Check } from 'lucide-svelte';
  import type { PluginDefinition, PluginInstance, SchemaFieldValue } from '$lib/types/plugin';
  import { SCHEMA_FIELD_TYPE_LABELS, getDefaultValueForType, isFieldRequired } from '$lib/types/plugin';
  import { validatePluginInstanceConfig } from '$lib/services/pluginBackend';

  interface Props {
    visible: boolean;
    plugin: PluginDefinition | null;
    /** Valores pré-existentes (para edição) */
    existingConfig?: Record<string, SchemaFieldValue>;
    /** Nome customizado para a instância (opcional) */
    instanceLabel?: string;
    onClose: () => void;
    onConfigured: (instance: PluginInstance) => void;
  }

  let {
    visible = $bindable(),
    plugin,
    existingConfig,
    instanceLabel,
    onClose,
    onConfigured,
  }: Props = $props();

  // ─── Config State ─────────────────────────────────────────────────────────

  let config = $state<Record<string, SchemaFieldValue>>({});
  let listInputs = $state<Record<string, string>>({});
  let isLoading = $state(false);
  let error = $state<string | null>(null);

  // ─── Init config when plugin changes ──────────────────────────────────────

  $effect(() => {
    if (plugin && visible) {
      const initial: Record<string, SchemaFieldValue> = {};
      for (const field of plugin.schema) {
        if (existingConfig && field.name in existingConfig) {
          initial[field.name] = existingConfig[field.name];
        } else {
          initial[field.name] = field.defaultValue ?? getDefaultValueForType(field.type);
        }
      }
      config = initial;
      listInputs = {};
      error = null;
    }
  });

  // ─── Derived ──────────────────────────────────────────────────────────────

  const pluginLabel = $derived(
    instanceLabel || plugin?.name || 'Plugin'
  );

  // ─── Value Handlers ───────────────────────────────────────────────────────

  function setBool(fieldName: string, value: boolean) {
    config = { ...config, [fieldName]: value };
  }

  function setNumber(fieldName: string, value: string, isFloat: boolean) {
    const parsed = isFloat ? parseFloat(value) : parseInt(value, 10);
    config = { ...config, [fieldName]: isNaN(parsed) ? 0 : parsed };
  }

  function setString(fieldName: string, value: string) {
    config = { ...config, [fieldName]: value };
  }

  function addListItem(fieldName: string) {
    const input = (listInputs[fieldName] ?? '').trim();
    if (!input) return;
    const current = (config[fieldName] as SchemaFieldValue[]) ?? [];
    config = { ...config, [fieldName]: [...current, input] };
    listInputs = { ...listInputs, [fieldName]: '' };
  }

  function removeListItem(fieldName: string, index: number) {
    const current = (config[fieldName] as SchemaFieldValue[]) ?? [];
    config = { ...config, [fieldName]: current.filter((_, i) => i !== index) };
  }

  function handleListKeydown(event: KeyboardEvent, fieldName: string) {
    if (event.key === 'Enter') {
      event.preventDefault();
      addListItem(fieldName);
    }
  }

  // ─── Submit ─────────────────────────────────────────────────────────────

  async function handleSubmit() {
    if (!plugin) return;
    error = null;

    // Verificar campos obrigatórios (sem defaultValue = obrigatório)
    for (const field of plugin.schema) {
      if (isFieldRequired(field)) {
        const value = config[field.name];
        if (value === undefined || value === null || value === '') {
          error = `Campo "${field.name}" é obrigatório`;
          return;
        }
        if (field.type === 'list' && Array.isArray(value) && value.length === 0) {
          error = `Campo "${field.name}" precisa de pelo menos um item`;
          return;
        }
      }
    }

    isLoading = true;

    try {
      const result = await validatePluginInstanceConfig(plugin.id, config);

      if (result.success) {
        const instance: PluginInstance = {
          pluginId: plugin.id,
          pluginName: plugin.name,
          pluginKind: plugin.kind,
          config,
        };
        onConfigured(instance);
        onClose();
      } else {
        error = result.error || 'Erro na validação da configuração';
      }
    } catch (e) {
      error = e instanceof Error ? e.message : 'Erro inesperado';
    } finally {
      isLoading = false;
    }
  }

  function handleClose() {
    error = null;
    onClose();
  }
</script>

{#if visible && plugin}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed inset-0 z-[70] flex items-center justify-center bg-black/60"
    onclick={handleClose}
  >
    <div
      class="bg-white dark:bg-[#0c0c0e] rounded-2xl shadow-2xl w-full max-w-xl max-h-[85vh] flex flex-col overflow-hidden border border-slate-200 dark:border-white/10"
      onclick={(e) => e.stopPropagation()}
    >
      <!-- Header -->
      <div class="flex items-center justify-between px-6 py-4 border-b border-slate-200 dark:border-white/5 shrink-0">
        <div>
          <h2 class="text-lg font-bold text-slate-800 dark:text-white">Configurar {pluginLabel}</h2>
          <p class="text-xs text-slate-500 dark:text-zinc-400 mt-0.5">
            {plugin.schema.length} parâmetro(s) · {plugin.kind === 'driver' ? 'Driver' : 'Controlador'} · {plugin.runtime === 'python' ? 'Python' : 'Rust'}
          </p>
        </div>
        <button
          onclick={handleClose}
          class="p-2 rounded-lg hover:bg-slate-100 dark:hover:bg-white/5 text-slate-400 transition-colors"
        >
          <X size={20} />
        </button>
      </div>

      <!-- Content -->
      <div class="flex-1 overflow-y-auto p-6 space-y-4">
        {#if error}
          <div class="p-3 rounded-lg bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-900/50 text-red-700 dark:text-red-400 text-sm flex items-center gap-2">
            <AlertCircle size={16} class="shrink-0" />
            {error}
          </div>
        {/if}

        {#if plugin.schema.length === 0}
          <div class="text-center py-8 text-sm text-slate-400 dark:text-zinc-500">
            Este plugin não requer configuração.
          </div>
        {:else}
          {#each plugin.schema as field (field.name)}
            <div class="space-y-1.5">
              <div class="flex items-center gap-2">
                <span class="text-[10px] font-bold text-slate-400 dark:text-zinc-500 uppercase">
                  {field.name}
                </span>
                <span class="text-[9px] px-1.5 py-0.5 rounded bg-slate-100 dark:bg-white/5 text-slate-400 dark:text-zinc-500">
                  {SCHEMA_FIELD_TYPE_LABELS[field.type]}
                </span>
                {#if isFieldRequired(field)}
                  <span class="text-[9px] px-1.5 py-0.5 rounded bg-amber-100 dark:bg-amber-900/30 text-amber-600 dark:text-amber-400">
                    obrigatório
                  </span>
                {/if}
              </div>

              {#if field.description}
                <p class="text-xs text-slate-400 dark:text-zinc-500">{field.description}</p>
              {/if}

              <!-- ───── Bool: toggle switch ───── -->
              {#if field.type === 'bool'}
                <button
                  onclick={() => setBool(field.name, !config[field.name])}
                  class="flex items-center gap-3 h-10 px-3 rounded-lg border border-slate-200 dark:border-white/10 bg-white dark:bg-[#18181b] w-full text-left"
                >
                  <div
                    class="w-10 h-5 rounded-full transition-colors relative {config[field.name] ? 'bg-blue-500' : 'bg-slate-300 dark:bg-zinc-600'}"
                  >
                    <div
                      class="absolute top-0.5 w-4 h-4 rounded-full bg-white shadow transition-transform {config[field.name] ? 'translate-x-5' : 'translate-x-0.5'}"
                    ></div>
                  </div>
                  <span class="text-sm {config[field.name] ? 'text-blue-600 dark:text-blue-400 font-medium' : 'text-slate-500 dark:text-zinc-400'}">
                    {config[field.name] ? 'Ativado' : 'Desativado'}
                  </span>
                </button>

              <!-- ───── Int: numeric input ───── -->
              {:else if field.type === 'int'}
                <input
                  type="number"
                  step="1"
                  value={config[field.name] as number}
                  oninput={(e) => setNumber(field.name, (e.target as HTMLInputElement).value, false)}
                  class="w-full h-10 px-3 rounded-lg border border-slate-200 dark:border-white/10 bg-white dark:bg-[#18181b] text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50"
                />

              <!-- ───── Float: numeric input (decimal) ───── -->
              {:else if field.type === 'float'}
                <input
                  type="number"
                  step="any"
                  value={config[field.name] as number}
                  oninput={(e) => setNumber(field.name, (e.target as HTMLInputElement).value, true)}
                  class="w-full h-10 px-3 rounded-lg border border-slate-200 dark:border-white/10 bg-white dark:bg-[#18181b] text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50"
                />

              <!-- ───── String: text input ───── -->
              {:else if field.type === 'string'}
                <input
                  type="text"
                  value={config[field.name] as string}
                  oninput={(e) => setString(field.name, (e.target as HTMLInputElement).value)}
                  placeholder={`Valor de ${field.name}`}
                  class="w-full h-10 px-3 rounded-lg border border-slate-200 dark:border-white/10 bg-white dark:bg-[#18181b] text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50"
                />

              <!-- ───── List: array builder ───── -->
              {:else if field.type === 'list'}
                <div class="space-y-2">
                  <!-- Itens existentes -->
                  {#if Array.isArray(config[field.name]) && (config[field.name] as SchemaFieldValue[]).length > 0}
                    <div class="space-y-1">
                      {#each (config[field.name] as SchemaFieldValue[]) as item, idx}
                        <div class="flex items-center gap-2 pl-3 pr-1 py-1 rounded-lg bg-slate-50 dark:bg-white/[0.03] border border-slate-200 dark:border-white/5">
                          <span class="flex-1 text-sm font-mono text-slate-700 dark:text-zinc-300 truncate">{item}</span>
                          <button
                            onclick={() => removeListItem(field.name, idx)}
                            class="p-1 rounded hover:bg-red-100 dark:hover:bg-red-900/30 text-slate-400 hover:text-red-500 transition-colors shrink-0"
                          >
                            <Trash2 size={12} />
                          </button>
                        </div>
                      {/each}
                    </div>
                  {/if}
                  <!-- Input para novo item -->
                  <div class="flex items-center gap-2">
                    <input
                      type="text"
                      bind:value={listInputs[field.name]}
                      onkeydown={(e) => handleListKeydown(e, field.name)}
                      placeholder="Adicionar item..."
                      class="flex-1 h-9 px-3 rounded-lg border border-slate-200 dark:border-white/10 bg-white dark:bg-[#18181b] text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50"
                    />
                    <button
                      onclick={() => addListItem(field.name)}
                      class="h-9 w-9 rounded-lg border border-slate-200 dark:border-white/10 bg-white dark:bg-[#18181b] hover:bg-blue-50 dark:hover:bg-blue-900/20 text-slate-400 hover:text-blue-500 transition-colors flex items-center justify-center shrink-0"
                    >
                      <Plus size={16} />
                    </button>
                  </div>
                </div>
              {/if}
            </div>
          {/each}
        {/if}
      </div>

      <!-- Footer -->
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
            Validando...
          {:else}
            <Check size={16} />
            Confirmar Configuração
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}
