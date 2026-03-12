<script lang="ts">
  import { AlertCircle, Plus, Search, Settings, X } from 'lucide-svelte';
  import { listControllerTemplates } from '$lib/services/plugin';
  import type { Controller } from '$lib/types/controller';

  interface Props {
    visible: boolean;
    onClose: () => void;
    onSelect: (controller: Controller) => void;
  }

  let {
    visible = $bindable(false),
    onClose,
    onSelect,
  }: Props = $props();

  let templates = $state<Controller[]>([]);
  let search = $state('');
  let isLoading = $state(false);
  let error = $state<string | null>(null);

  const filteredTemplates = $derived.by(() => {
    const query = search.trim().toLowerCase();

    if (!query) {
      return templates;
    }

    return templates.filter(
      (controller) =>
        controller.name.toLowerCase().includes(query) ||
        controller.type.toLowerCase().includes(query)
    );
  });

  async function loadTemplates() {
    isLoading = true;
    error = null;

    try {
      templates = await listControllerTemplates();
    } catch (exception) {
      error = exception instanceof Error ? exception.message : 'Erro ao carregar controladores da biblioteca';
    } finally {
      isLoading = false;
    }
  }

  function handleClose() {
    visible = false;
    search = '';
    error = null;
    onClose();
  }

  function handleSelect(template: Controller) {
    onSelect(template);
    handleClose();
  }

  $effect(() => {
    if (!visible) {
      return;
    }

    search = '';
    void loadTemplates();
  });
</script>

{#if visible}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed inset-0 z-[72] flex items-center justify-center bg-black/60 px-4"
    onclick={handleClose}
  >
    <div
      class="flex max-h-[85vh] w-full max-w-2xl flex-col overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-2xl dark:border-white/10 dark:bg-[#0c0c0e]"
      onclick={(event) => event.stopPropagation()}
    >
      <div class="flex items-center justify-between border-b border-slate-200 px-6 py-4 dark:border-white/5">
        <div>
          <h2 class="text-lg font-bold text-slate-800 dark:text-white">Biblioteca de Controladores</h2>
          <p class="mt-0.5 text-xs text-slate-500 dark:text-zinc-400">
            Selecione um controlador já cadastrado na biblioteca de plugins.
          </p>
        </div>
        <button
          type="button"
          onclick={handleClose}
          class="rounded-lg p-2 text-slate-400 transition-colors hover:bg-slate-100 hover:text-slate-600 dark:hover:bg-white/5 dark:hover:text-white"
        >
          <X size={20} />
        </button>
      </div>

      <div class="flex-1 overflow-y-auto p-6">
        <div class="relative mb-4">
          <Search size={16} class="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" />
          <input
            type="text"
            bind:value={search}
            placeholder="Buscar controlador..."
            class="w-full rounded-lg border border-slate-200 bg-white py-2 pl-9 pr-4 text-sm text-slate-800 outline-none transition focus:border-blue-500 focus:ring-2 focus:ring-blue-500/20 dark:border-white/10 dark:bg-[#18181b] dark:text-white"
          />
        </div>

        {#if error}
          <div class="mb-4 flex items-center gap-2 rounded-lg border border-red-200 bg-red-50 p-3 text-sm text-red-700 dark:border-red-900/50 dark:bg-red-900/20 dark:text-red-400">
            <AlertCircle size={16} class="shrink-0" />
            {error}
          </div>
        {/if}

        {#if isLoading}
          <div class="flex items-center justify-center py-12 text-sm text-slate-500 dark:text-zinc-400">
            Carregando controladores...
          </div>
        {:else if filteredTemplates.length === 0}
          <div class="rounded-xl border border-dashed border-slate-200 bg-slate-50 p-8 text-center dark:border-white/10 dark:bg-white/[0.02]">
            <div class="mx-auto mb-3 flex h-12 w-12 items-center justify-center rounded-xl bg-slate-200 text-slate-500 dark:bg-white/5 dark:text-zinc-400">
              <Settings size={20} />
            </div>
            <div class="text-sm font-medium text-slate-700 dark:text-zinc-200">
              Nenhum controlador encontrado
            </div>
            <p class="mt-1 text-xs text-slate-500 dark:text-zinc-400">
              Cadastre plugins do tipo `controller` no módulo Plugins para usá-los aqui.
            </p>
          </div>
        {:else}
          <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
            {#each filteredTemplates as template (template.id)}
              <button
                type="button"
                onclick={() => handleSelect(template)}
                class="rounded-xl border border-slate-200 bg-white p-4 text-left transition-all hover:border-emerald-400 hover:shadow-sm dark:border-white/10 dark:bg-[#18181b] dark:hover:border-emerald-500"
              >
                <div class="mb-2 flex items-center gap-2">
                  <div class="flex h-9 w-9 items-center justify-center rounded-lg bg-emerald-100 text-emerald-600 dark:bg-emerald-900/30 dark:text-emerald-400">
                    <Settings size={16} />
                  </div>
                  <div class="min-w-0">
                    <div class="truncate text-sm font-semibold text-slate-800 dark:text-white">{template.name}</div>
                    <div class="text-xs text-slate-500 dark:text-zinc-400">{template.type}</div>
                  </div>
                </div>

                <div class="mb-3 text-xs text-slate-500 dark:text-zinc-400">
                  {Object.keys(template.params ?? {}).length} parâmetro(s) configurável(is)
                </div>

                <div class="inline-flex items-center gap-2 rounded-full bg-emerald-50 px-3 py-1 text-xs font-medium text-emerald-700 dark:bg-emerald-900/20 dark:text-emerald-300">
                  <Plus size={12} />
                  Adicionar à planta
                </div>
              </button>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}
