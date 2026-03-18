<script lang="ts">
  import { Plus, Search, Puzzle, Upload, RefreshCw } from 'lucide-svelte';
  import type { PluginDefinition, PluginKind } from '$lib/types/plugin';
  import { BUILTIN_PLUGIN_KINDS, getPluginKindLabel } from '$lib/types/plugin';
  import PluginCard from '$lib/components/plugins/PluginCard.svelte';
  import PluginCategoryTabs, { type PluginCategoryTab } from '$lib/components/plugins/PluginCategoryTabs.svelte';
  import CreatePluginModal from '$lib/components/modals/CreatePluginModal.svelte';
  import CodeEditorModal from '$lib/components/modals/CodeEditorModal.svelte';
  import GenericModal from '$lib/components/modals/GenericModal.svelte';
  import { deletePlugin, getPlugin, listPlugins, loadSystemPlugins, registerPlugin, validatePluginFile } from '$lib/services/plugin';
  import { FILE_FILTERS, openFileDialog, readFileAsJSON } from '$lib/services/fileDialog';

  interface Props {
    theme: 'dark' | 'light';
    active?: boolean;
  }

  let { theme, active = true }: Props = $props();

  let plugins = $state<PluginDefinition[]>([]);
  let isLoading = $state(false);
  let loadError = $state<string | null>(null);
  let activeCategory = $state<PluginKind>('driver');
  let searchQuery = $state('');

  let showCreateModal = $state(false);
  let showCodeViewer = $state(false);
  let showDeleteConfirm = $state(false);
  let showImportModal = $state(false);
  let selectedPlugin = $state<PluginDefinition | null>(null);
  let createModalInitialKind = $state<PluginKind | undefined>(undefined);

  const pluginCategories = $derived.by<PluginCategoryTab[]>(() => {
    const counts = new Map<PluginKind, number>();

    for (const kind of BUILTIN_PLUGIN_KINDS) {
      counts.set(kind, 0);
    }

    for (const plugin of plugins) {
      counts.set(plugin.kind, (counts.get(plugin.kind) ?? 0) + 1);
    }

    return Array.from(counts.entries()).map(([key, count]) => ({ key, count }));
  });

  const normalizedSearchQuery = $derived(searchQuery.trim().toLowerCase());
  const filteredPlugins = $derived.by(() => {
    const query = normalizedSearchQuery;
    return plugins.filter((plugin) => {
      if (plugin.kind !== activeCategory) return false;
      if (!query) return true;
      return (
        plugin.name.toLowerCase().includes(query) ||
        plugin.description?.toLowerCase().includes(query)
      );
    });
  });

  const isEmpty = $derived(filteredPlugins.length === 0);
  const pluginCount = $derived(plugins.length);
  const activeCategoryLabel = $derived(getPluginKindLabel(activeCategory));

  $effect(() => {
    if (!pluginCategories.length) return;
    if (!pluginCategories.some((category) => category.key === activeCategory)) {
      activeCategory = pluginCategories[0].key;
    }
  });

  async function loadCatalog() {
    isLoading = true;
    loadError = null;

    try {
      await loadSystemPlugins();
      plugins = await listPlugins();
    } catch (error) {
      loadError = error instanceof Error ? error.message : 'Erro ao carregar os plugins';
    } finally {
      isLoading = false;
    }
  }

  $effect(() => {
    if (!active) return;
    void loadCatalog();
  });

  function handlePluginCreated(plugin: PluginDefinition) {
    plugins = [plugin, ...plugins.filter((entry) => entry.id !== plugin.id)];
    activeCategory = plugin.kind;
    selectedPlugin = null;
    createModalInitialKind = undefined;
    showCreateModal = false;
  }

  function handleViewCode(plugin: PluginDefinition) {
    selectedPlugin = plugin;
    showCodeViewer = true;
  }

  async function handleEditPlugin(plugin: PluginDefinition) {
    loadError = null;
    const pluginFromStore = await getPlugin(plugin.id);
    if (!pluginFromStore) {
      loadError = 'Não foi possível carregar os dados mais recentes do plugin';
      return;
    }

    selectedPlugin = pluginFromStore;
    createModalInitialKind = undefined;
    showCreateModal = true;
  }

  function handleDeletePlugin(plugin: PluginDefinition) {
    selectedPlugin = plugin;
    showDeleteConfirm = true;
  }

  async function confirmDelete() {
    const pluginToDelete = selectedPlugin;
    if (pluginToDelete) {
      const result = await deletePlugin(pluginToDelete.id);
      if (result.success) {
        plugins = plugins.filter((plugin) => plugin.id !== pluginToDelete.id);
        selectedPlugin = null;
        showDeleteConfirm = false;
      } else {
        loadError = result.error || 'Erro ao excluir plugin';
      }
    }
  }

  function handleCodeEditorClose() {
    showCodeViewer = false;
    selectedPlugin = null;
  }

  function applyImportedPlugin(plugin: PluginDefinition) {
    plugins = [plugin, ...plugins.filter((entry) => entry.id !== plugin.id)];
    activeCategory = plugin.kind;
  }

  async function importPluginFromFile(file: File): Promise<{ success: boolean; error?: string }> {
    try {
      const json = await readFileAsJSON(file);
      const validation = await validatePluginFile(json);
      if (!validation.success || !validation.plugin) {
        return { success: false, error: validation.error || 'Não foi possível validar o plugin' };
      }

      const registration = await registerPlugin(validation.plugin);
      if (!registration.success || !registration.plugin) {
        return { success: false, error: registration.error || 'Não foi possível salvar o plugin' };
      }

      applyImportedPlugin(registration.plugin);
      return { success: true };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Erro ao importar plugin',
      };
    }
  }

  async function handleImportPlugin() {
    loadError = null;

    try {
      const result = await openFileDialog({
        title: 'Importar plugin',
        filters: FILE_FILTERS.json,
      });

      if (!result) return;

      const importResult = await importPluginFromFile(result.file);
      if (!importResult.success) {
        loadError = importResult.error || 'Não foi possível importar o arquivo selecionado';
        return;
      }

      showImportModal = false;
    } catch (error) {
      loadError = error instanceof Error ? error.message : 'Erro ao importar plugin';
    }
  }

  function openCreateModal(kind?: PluginKind) {
    selectedPlugin = null;
    createModalInitialKind = kind;
    showCreateModal = true;
  }

  let dragOver = $state(false);

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    dragOver = true;
  }

  function handleDragLeave(e: DragEvent) {
    e.preventDefault();
    dragOver = false;
  }

  async function handleDrop(e: DragEvent) {
    e.preventDefault();
    dragOver = false;
    loadError = null;

    const files = e.dataTransfer?.files;
    if (!files || files.length === 0) return;

    const file = files[0];
    if (!file.name.toLowerCase().endsWith('.json')) {
      loadError = 'Apenas arquivos .json são aceitos para importação de plugins';
      return;
    }

    const importResult = await importPluginFromFile(file);
    if (!importResult.success) {
      loadError = importResult.error || 'Não foi possível importar o plugin';
    }
  }
</script>

<div 
  class="flex-1 flex flex-col min-h-0 bg-slate-50 dark:bg-zinc-950"
  ondragover={handleDragOver}
  ondragleave={handleDragLeave}
  ondrop={handleDrop}
  role="region"
  aria-label="Gerenciador de Plugins"
>
  <header class="flex-shrink-0 border-b border-slate-200 dark:border-white/5 bg-white/90 px-4 py-5 backdrop-blur dark:bg-zinc-900/90 sm:px-6">
    <div class="mx-auto flex w-full max-w-[1440px] flex-col gap-4">
      <div class="flex flex-col gap-4 xl:flex-row xl:items-center xl:justify-between">
        <div class="flex min-w-0 flex-wrap items-center gap-4">
          <div class="flex items-center gap-3">
            <div class="flex h-11 w-11 items-center justify-center rounded-2xl bg-gradient-to-br from-sky-500 via-blue-500 to-indigo-600 shadow-md shadow-blue-500/20">
              <Puzzle class="h-5 w-5 text-white" />
            </div>
            <div>
              <div class="flex flex-wrap items-center gap-2">
                <h1 class="text-lg font-semibold text-slate-800 dark:text-white">Plugins</h1>
                <span class="rounded-full border border-slate-200 bg-slate-100 px-2.5 py-1 text-[11px] font-medium text-slate-500 dark:border-white/10 dark:bg-zinc-800 dark:text-zinc-400">
                  {pluginCount} itens
                </span>
              </div>
              <p class="text-xs text-slate-500 dark:text-zinc-400">Encontre, edite e organize seus plugins em um só lugar.</p>
            </div>
          </div>
        </div>

        <div class="flex flex-col gap-3 sm:flex-row sm:flex-wrap xl:justify-end">
          <div class="relative min-w-0 sm:min-w-[280px]">
            <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-400 dark:text-zinc-500" />
            <input
              type="text"
              placeholder="Buscar plugins..."
              bind:value={searchQuery}
              class="h-10 w-full rounded-xl border border-transparent bg-slate-100 py-2.5 pl-10 pr-4 text-sm text-slate-700 placeholder:text-slate-400 transition-colors focus:border-blue-500 focus:outline-none dark:bg-zinc-800 dark:text-zinc-200 dark:placeholder:text-zinc-500 dark:focus:border-blue-500"
            />
          </div>

          <div class="flex flex-wrap items-center gap-3">
            <button
              onclick={() => showImportModal = true}
              class="flex h-10 items-center justify-center gap-2 whitespace-nowrap rounded-xl border border-slate-200 bg-white px-3 text-sm font-medium text-slate-700 transition-colors hover:bg-slate-50 dark:border-white/10 dark:bg-zinc-800 dark:text-zinc-200 dark:hover:bg-zinc-700"
            >
              <Upload class="w-4 h-4" />
              <span>Importar</span>
            </button>

            <button
              onclick={() => openCreateModal(activeCategory)}
              class="flex h-10 items-center justify-center gap-2 whitespace-nowrap rounded-xl bg-blue-600 px-4 text-sm font-medium text-white shadow-sm transition-colors hover:bg-blue-700"
            >
              <Plus class="w-4 h-4" />
              <span>Novo Plugin</span>
            </button>

            <button
              onclick={loadCatalog}
              class="flex h-10 items-center justify-center gap-2 whitespace-nowrap rounded-xl border border-slate-200 bg-white px-3 text-sm font-medium text-slate-700 transition-colors hover:bg-slate-50 dark:border-white/10 dark:bg-zinc-800 dark:text-zinc-200 dark:hover:bg-zinc-700"
            >
              <RefreshCw class={`w-4 h-4 ${isLoading ? 'animate-spin' : ''}`} />
              <span>Atualizar</span>
            </button>
          </div>
        </div>
      </div>

      <div class="rounded-2xl border border-slate-200 bg-slate-100/80 p-2 dark:border-white/10 dark:bg-zinc-900/70">
        <PluginCategoryTabs
          {activeCategory}
          categories={pluginCategories}
          onCategoryChange={(cat) => activeCategory = cat}
        />
      </div>
    </div>
  </header>

  <main class="flex-1 overflow-auto px-4 py-6 sm:px-6">
    <section class="mx-auto w-full max-w-[1440px] min-w-0">
      <div class="relative rounded-[28px] border border-slate-200 bg-white/80 p-4 shadow-sm dark:border-white/10 dark:bg-zinc-900/80 sm:p-5">
        {#if dragOver}
          <div class="pointer-events-none absolute inset-3 z-40 flex items-center justify-center rounded-[22px] border-2 border-dashed border-blue-500 bg-blue-500/10 backdrop-blur-sm sm:inset-4">
            <div class="text-center">
              <Upload class="mx-auto h-10 w-10 text-blue-600 dark:text-blue-400" />
              <p class="mt-3 text-sm font-semibold text-blue-700 dark:text-blue-300">Solte o arquivo JSON para importar o plugin</p>
            </div>
          </div>
        {/if}

        <div class="mb-4 flex items-center justify-between gap-3 px-1">
          <div>
            <h2 class="text-sm font-semibold text-slate-700 dark:text-zinc-200">Biblioteca</h2>
            <p class="text-xs text-slate-500 dark:text-zinc-400">Use os filtros para encontrar, editar ou adicionar plugins rapidamente.</p>
          </div>
        </div>

        {#if loadError}
          <div class="mb-4 rounded-2xl border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800 dark:border-amber-500/20 dark:bg-amber-500/10 dark:text-amber-200">
            {loadError}
          </div>
        {/if}

        {#if isLoading}
          <div class="flex items-center justify-center rounded-[24px] border border-slate-200 bg-white p-12 dark:border-white/10 dark:bg-zinc-900">
            <div class="text-center">
              <RefreshCw class="mx-auto h-8 w-8 animate-spin text-blue-500" />
              <p class="mt-3 text-sm text-slate-500 dark:text-zinc-400">Carregando biblioteca de plugins...</p>
            </div>
          </div>
        {:else if isEmpty && !searchQuery}
          <div 
            class="flex min-h-[420px] flex-col items-center justify-center gap-6 rounded-[24px] border-2 border-dashed border-slate-300 bg-white p-12 text-center transition-colors hover:border-blue-400 dark:border-zinc-700 dark:bg-zinc-900 dark:hover:border-blue-500"
          >
            <div class="w-20 h-20 rounded-3xl bg-slate-100 dark:bg-zinc-800 flex items-center justify-center">
              <Puzzle class="w-10 h-10 text-slate-400 dark:text-zinc-500" />
            </div>
            <div class="space-y-2">
              <h2 class="text-2xl font-semibold text-slate-800 dark:text-white">
                Nenhum plugin do tipo {activeCategoryLabel} cadastrado
              </h2>
              <p class="mx-auto max-w-md text-sm text-slate-500 dark:text-zinc-400">
                Crie um novo plugin ou importe um arquivo JSON para começar sua biblioteca.
              </p>
            </div>
            <div class="flex flex-wrap items-center justify-center gap-3">
              <button
                onclick={() => openCreateModal(activeCategory)}
                class="flex items-center gap-2 rounded-xl bg-blue-600 px-5 py-3 font-medium text-white shadow-sm transition-colors hover:bg-blue-700"
              >
                <Plus class="w-4 h-4" />
                Criar Plugin
              </button>
              <button
                onclick={() => showImportModal = true}
                class="flex items-center gap-2 rounded-xl border border-slate-200 bg-white px-5 py-3 font-medium text-slate-700 transition-colors hover:bg-slate-50 dark:border-white/10 dark:bg-zinc-800 dark:text-zinc-200 dark:hover:bg-zinc-700"
              >
                <Upload class="w-4 h-4" />
                Importar JSON
              </button>
            </div>
            <p class="text-xs text-slate-400 dark:text-zinc-500">
              Dica: arraste um arquivo <code class="rounded bg-slate-100 px-1.5 py-0.5 dark:bg-zinc-800">.json</code> para esta área
            </p>
          </div>
        {:else if isEmpty && searchQuery}
          <div class="flex min-h-[320px] flex-col items-center justify-center rounded-[24px] border border-slate-200 bg-white p-8 text-center shadow-sm dark:border-white/10 dark:bg-zinc-900">
            <Search class="mb-4 h-12 w-12 text-slate-300 dark:text-zinc-600" />
            <h2 class="text-lg font-semibold text-slate-700 dark:text-zinc-200">Nenhum resultado encontrado</h2>
            <p class="mt-2 text-sm text-slate-500 dark:text-zinc-400">
              Não encontramos plugins correspondentes a "{searchQuery}".
            </p>
            <button
              onclick={() => searchQuery = ''}
              class="mt-4 text-sm font-medium text-blue-600 hover:underline dark:text-blue-400"
            >
              Limpar busca
            </button>
          </div>
        {:else}
          <div class="grid grid-cols-1 gap-4 md:grid-cols-2 md:auto-rows-fr xl:grid-cols-3 2xl:grid-cols-4">
            {#each filteredPlugins as plugin (plugin.id)}
              <PluginCard
                {plugin}
                onEdit={handleEditPlugin}
                onDelete={handleDeletePlugin}
                onViewCode={handleViewCode}
              />
            {/each}
          </div>
        {/if}
      </div>
    </section>
  </main>
</div>

<!-- Modal de criação -->
{#if showCreateModal}
  <CreatePluginModal
    visible={showCreateModal}
    initialKind={createModalInitialKind}
    initialPlugin={selectedPlugin}
    onClose={() => { showCreateModal = false; selectedPlugin = null; createModalInitialKind = undefined; }}
    onPluginCreated={handlePluginCreated}
  />
{/if}

<!-- Visualizador de código -->
{#if showCodeViewer && selectedPlugin?.sourceCode}
  <CodeEditorModal
    visible={showCodeViewer}
    initialCode={selectedPlugin.sourceCode}
    initialFileName={`${selectedPlugin.name}.${selectedPlugin.runtime === 'python' ? 'py' : 'rs'}`}
    title={`Código: ${selectedPlugin.name}`}
    onClose={handleCodeEditorClose}
    onSave={handleCodeEditorClose}
  />
{/if}

<!-- Confirmação de exclusão -->
<GenericModal
  visible={showDeleteConfirm}
  type="warning"
  title="Excluir plugin"
  message={`Tem certeza que deseja excluir o plugin "${selectedPlugin?.name}"? Esta ação não pode ser desfeita.`}
  confirmLabel="Excluir"
  onConfirm={confirmDelete}
  onClose={() => { showDeleteConfirm = false; selectedPlugin = null; }}
/>

<GenericModal
  visible={showImportModal}
  type="info"
  title="Importar plugin"
  message="Selecione um arquivo JSON para adicionar um novo plugin à biblioteca."
  confirmLabel="Selecionar arquivo"
  onConfirm={handleImportPlugin}
  onClose={() => showImportModal = false}
/>
