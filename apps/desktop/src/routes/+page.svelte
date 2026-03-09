<script lang="ts">
  import { onMount } from 'svelte';
  import { appStore } from '$lib/stores/data.svelte';
  import Sidebar from '$lib/components/layout/Sidebar.svelte';
  import PlotterModule from '$lib/components/modules/PlotterModule.svelte';
  import AnalyzerModule from '$lib/components/modules/AnalyzerModule.svelte';
  import PluginsModule from '$lib/components/modules/PluginsModule.svelte';
  import GlobalSettingsModal from '$lib/components/modals/GlobalSettingsModal.svelte';
  import { listPlants } from '$lib/services/plant';
  import { setPlantStats } from '$lib/stores/plantData';

  let showControllerPanel = $state(false);

  onMount(async () => {
    try {
      const plants = await listPlants();
      appStore.setPlants(plants);
      for (const plant of plants) {
        setPlantStats(plant.id, plant.stats);
      }
    } catch (error) {
      console.error('Erro ao carregar plantas iniciais:', error);
    }
  });

  $effect(() => {
    const theme = appStore.state.theme || 'dark';
    if (typeof document === 'undefined') return;
    const root = document.documentElement;
    const body = document.body;
    const isDark = theme === 'dark';
    root.classList.toggle('dark', isDark);
    body.classList.toggle('dark', isDark);
    root.style.colorScheme = isDark ? 'dark' : 'light';
  });
</script>

<div class="h-screen w-full select-none">
  <div class="flex h-full w-full bg-slate-100 dark:bg-zinc-950 text-slate-800 dark:text-zinc-100 font-sans overflow-hidden transition-colors duration-300">
    <Sidebar
      theme={appStore.state.theme || 'dark'}
      sidebarCollapsed={appStore.state.sidebarCollapsed ?? true}
      activeModule={appStore.state.activeModule || 'plotter'}
    />

    <main class="flex-1 flex flex-col min-w-0 relative">
      <div class="flex-1 flex flex-col min-w-0" style:display={appStore.state.activeModule === 'plotter' ? 'flex' : 'none'}>
        <PlotterModule
          plants={appStore.state.plants || []}
          activePlantId={appStore.state.activePlantId ?? appStore.state.plants[0]?.id ?? ''}
          theme={appStore.state.theme || 'dark'}
          active={appStore.state.activeModule === 'plotter'}
          bind:showControllerPanel
        />
      </div>
      <div class="flex-1 flex flex-col min-w-0" style:display={appStore.state.activeModule === 'analyzer' ? 'flex' : 'none'}>
        <AnalyzerModule
          theme={appStore.state.theme || 'dark'}
          active={appStore.state.activeModule === 'analyzer'}
        />
      </div>
      <div class="flex-1 flex flex-col min-w-0" style:display={appStore.state.activeModule === 'plugins' ? 'flex' : 'none'}>
        <PluginsModule
          theme={appStore.state.theme || 'dark'}
          active={appStore.state.activeModule === 'plugins'}
        />
      </div>

      <GlobalSettingsModal showGlobalSettings={appStore.state.showGlobalSettings || false} />
    </main>
  </div>
</div>


