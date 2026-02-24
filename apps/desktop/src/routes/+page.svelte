<script lang="ts">
  import { appStore } from '$lib/stores/data.svelte';
  import { getPlantData, getPlantStats, setPlantStats } from '$lib/stores/plantData';
  import Sidebar from '$lib/components/layout/Sidebar.svelte';
  import PlotterModule from '$lib/components/modules/PlotterModule.svelte';
  import PoleAnalysisModule from '$lib/components/modules/PoleAnalysisModule.svelte';
  import GlobalSettingsModal from '$lib/components/modals/GlobalSettingsModal.svelte';
  import type { Plant } from '$lib/types/plant';

  let showControllerPanel = $state(false);
  const thermalState = new Map<string, { x1: number; x2: number }>();

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
    const SIM_INTERVAL = 100;
    const DT = SIM_INTERVAL / 1000;

    const interval = setInterval(() => {
      appStore.state.plants.forEach((plant: Plant) => {
        if (!plant.connected || plant.paused) return;

        const data = getPlantData(plant.id);
        const last = data.length > 0
          ? data[data.length - 1]
          : { pv: 0, mv: 0, time: 0, sp: plant.setpoint };

        const time = last.time + DT;
        let totalMv = 0;
        const error = plant.setpoint - last.pv;

        plant.controllers.forEach((ctrl) => {
          if (!ctrl.active) return;
          const p = ctrl.params as any;
          if (p.manualMode?.value) { totalMv = 50; return; }
          const kp = Number(p.kp?.value) || 0;
          const ki = Number(p.ki?.value) || 0;
          const kd = Number(p.kd?.value) || 0;
          const prevPv = data.length > 1 ? data[data.length - 2].pv : last.pv;
          totalMv += kp * error + ki * error * DT - kd * (last.pv - prevPv) / DT;
        });

        totalMv = Math.max(0, Math.min(100, totalMv + (Math.random() * 1.5 - 0.75)));

        let newPv: number;
        if (plant.id === 'p2') {
          const k = 1.63, t1 = 0.003, t2 = 3.03;
          const st = thermalState.get(plant.id) ?? { x1: 0, x2: 0 };
          const x1 = st.x1 + DT * (-t1 * st.x1 + t1 * totalMv);
          const x2 = st.x2 + DT * (-t2 * st.x2 + t2 * x1);
          thermalState.set(plant.id, { x1, x2 });
          newPv = k * x2 + (Math.random() * 0.2 - 0.1);
        } else {
          newPv = last.pv * 0.94 + totalMv * 0.06 + (Math.random() * 0.4 - 0.2);
        }

        data.push({ time, sp: plant.setpoint, pv: newPv, mv: totalMv });

        const prev = getPlantStats(plant.id);
        setPlantStats(plant.id, {
          errorAvg: prev.errorAvg * 0.95 + Math.abs(error) * 0.05,
          stability: Math.max(0, 100 - (prev.errorAvg * 0.95 + Math.abs(error) * 0.05) * 2),
          uptime: prev.uptime + 1
        });
      });
    }, SIM_INTERVAL);

    return () => clearInterval(interval);
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



