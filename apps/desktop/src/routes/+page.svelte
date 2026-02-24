<script lang="ts">
  import { appStore } from '$lib/stores/data.svelte';
  import { startSimulation } from '$lib/services/simulation';
  import Sidebar from '$lib/components/layout/Sidebar.svelte';
  import PlotterModule from '$lib/components/modules/PlotterModule.svelte';
  import PoleAnalysisModule from '$lib/components/modules/PoleAnalysisModule.svelte';
  import GlobalSettingsModal from '$lib/components/modals/GlobalSettingsModal.svelte';

  let showControllerPanel = $state(false);

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

  $effect(() => {
    if (typeof window === 'undefined') return;
    return startSimulation(() => appStore.state.plants);
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
      {#if appStore.state.activeModule === 'plotter'}
        <PlotterModule
          plants={appStore.state.plants || []}
          activePlantId={appStore.state.activePlantId || 'p1'}
          theme={appStore.state.theme || 'dark'}
          bind:showControllerPanel
        />
      {:else if appStore.state.activeModule === 'poles'}
        <PoleAnalysisModule theme={appStore.state.theme || 'dark'} />
      {:else}
        <div class="flex-1 flex items-center justify-center text-slate-400 dark:text-zinc-600 bg-slate-50 dark:bg-zinc-950">
          <div class="text-center">
            <svg width="48" height="48" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24" class="opacity-20 mb-4 mx-auto">
              <polyline points="22 12 18 12 15 21 9 3 6 12 2 12" />
            </svg>
            <p class="text-lg font-bold text-slate-500 dark:text-zinc-400">
              Módulo não implementado
            </p>
          </div>
        </div>
      {/if}

      <GlobalSettingsModal showGlobalSettings={appStore.state.showGlobalSettings || false} />
    </main>
  </div>
</div>



