<script lang="ts">
  import { Power, Play, Pause, Home, Download, Camera, Timer, Sliders, ChevronDown, Pencil } from 'lucide-svelte';
  import type { Plant, PlantStats } from '$lib/types/plant';

  let {
    plant,
    currentStats,
    dt,
    showControllerPanel = $bindable(false),
    onToggleConnect,
    onTogglePause,
    onEditPlant,
    onResetZoom,
    onExportCSV,
    onExportJSON,
    onPrint,
    formatTime
  }: {
    plant: Plant | undefined;
    currentStats: PlantStats;
    dt: number;
    showControllerPanel: boolean;
    onToggleConnect: () => void;
    onTogglePause: () => void;
    onEditPlant: () => void;
    onResetZoom: () => void;
    onExportCSV: () => void;
    onExportJSON: () => void;
    onPrint: () => void;
    formatTime: (seconds: number) => string;
  } = $props();

  let exportMenuOpen = $state(false);
  const effectiveSamplingMs = $derived(dt > 0 ? dt * 1000 : 0);

  function handleExportCSV() {
    exportMenuOpen = false;
    onExportCSV();
  }

  function handleExportJSON() {
    exportMenuOpen = false;
    onExportJSON();
  }

  function handleClickOutside(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (!target.closest('.export-dropdown')) {
      exportMenuOpen = false;
    }
  }
</script>

<svelte:window onclick={handleClickOutside} />

<div class="plotter-toolbar min-h-14 bg-white dark:bg-[#0c0c0e] border-b border-slate-200 dark:border-white/5 flex flex-wrap items-center justify-between gap-2 px-3 py-2 shadow-sm z-20 print:hidden sm:px-4 sm:py-1 lg:px-6">
  <div class="plotter-toolbar__controls flex min-w-0 flex-1 items-center gap-2 overflow-x-auto sm:gap-3">
    <button
      onclick={onToggleConnect}
      class={`flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-bold transition-all border
        ${plant?.connected
          ? 'bg-red-50 text-red-600 border-red-200 hover:bg-red-100 dark:bg-red-900/20 dark:text-red-400 dark:border-red-900/30'
          : 'bg-emerald-50 text-emerald-600 border-emerald-200 hover:bg-emerald-100 dark:bg-emerald-900/20 dark:text-emerald-400 dark:border-emerald-900/30'}`}
    >
      <Power size={18} />
      <span class="hidden sm:inline">{plant?.connected ? 'DESLIGAR' : 'LIGAR'}</span>
      <span class="sm:hidden">{plant?.connected ? 'OFF' : 'ON'}</span>
    </button>
    <div class="mx-1 hidden h-8 w-px bg-slate-200 dark:bg-white/10 sm:block"></div>
    <button
      onclick={onTogglePause}
      disabled={!plant?.connected}
      class="p-2 rounded-lg hover:bg-slate-100 dark:hover:bg-white/5 text-slate-500 disabled:opacity-30 transition-colors"
      title={plant?.paused ? 'Retomar' : 'Pausar'}
    >
      {#if plant?.paused}
        <Play size={20} class="text-blue-500" fill="currentColor" />
      {:else}
        <Pause size={20} />
      {/if}
    </button>
    <button onclick={onResetZoom} class="p-2 rounded-lg hover:bg-slate-100 dark:hover:bg-white/5 text-slate-500 transition-colors" title="Home (Ver Tudo)">
      <Home size={20} />
    </button>
    <button onclick={onEditPlant} class="p-2 rounded-lg hover:bg-slate-100 dark:hover:bg-white/5 text-slate-500 transition-colors" title="Editar planta">
      <Pencil size={18} />
    </button>
    <div class="relative export-dropdown">
      <button
        onclick={(e) => { e.stopPropagation(); exportMenuOpen = !exportMenuOpen; }}
        class="flex items-center gap-0.5 p-2 rounded-lg hover:bg-slate-100 dark:hover:bg-white/5 text-slate-500 transition-colors"
        title="Exportar dados"
      >
        <Download size={20} />
        <ChevronDown size={12} />
      </button>
      {#if exportMenuOpen}
        <div class="absolute top-full left-0 mt-1 bg-white dark:bg-[#18181b] border border-slate-200 dark:border-white/10 rounded-lg shadow-lg z-50 min-w-[160px] py-1">
          <button
            onclick={handleExportCSV}
            class="w-full text-left px-4 py-2 text-sm text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-white/5 transition-colors"
          >
            Exportar CSV
          </button>
          <button
            onclick={handleExportJSON}
            class="w-full text-left px-4 py-2 text-sm text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-white/5 transition-colors"
          >
            Exportar JSON
          </button>
        </div>
      {/if}
    </div>
    <button onclick={onPrint} class="p-2 rounded-lg hover:bg-slate-100 dark:hover:bg-white/5 text-slate-500 transition-colors" title="Imprimir">
      <Camera size={20} />
    </button>
  </div>

  <div class="plotter-toolbar__meta ml-auto flex shrink-0 items-center gap-3 sm:gap-6">
    <div class="plotter-toolbar__timing hidden md:flex items-center gap-4 mr-4">
      {#if plant}
        <div class="flex flex-col items-end">
          <span class="text-[9px] font-bold text-slate-400 uppercase">Dt. Config.</span>
          <div class="text-xs font-mono font-bold text-slate-600 dark:text-slate-300">
            {plant.sampleTimeMs} ms
          </div>
        </div>
      {/if}
      <div class="flex flex-col items-end">
        <span class="text-[9px] font-bold text-slate-400 uppercase">Dt. Real</span>
        <div class="text-xs font-mono font-bold text-slate-600 dark:text-slate-300">
          {effectiveSamplingMs > 0 ? `${effectiveSamplingMs.toFixed(1)} ms` : '--'}
        </div>
      </div>
      {#if plant?.connected}
        <div class="flex flex-col items-end">
          <span class="text-[9px] font-bold text-slate-400 uppercase">Uptime</span>
          <div class="text-xs font-mono font-bold text-slate-600 dark:text-slate-300 flex items-center gap-1">
            <Timer size={10} class="text-purple-500" />
            {formatTime(currentStats.uptime)}
          </div>
        </div>
      {/if}
    </div>
    <div class="plotter-toolbar__status hidden sm:flex flex-col items-end mr-1">
      <span class="text-[10px] font-bold text-slate-400 uppercase tracking-wider">Status</span>
      <div class="flex items-center gap-1.5">
        <span class={`w-2 h-2 rounded-full ${plant?.connected ? 'bg-emerald-500 animate-pulse' : 'bg-slate-400'}`}></span>
        <span class={`text-xs font-bold ${plant?.connected ? 'text-emerald-600 dark:text-emerald-400' : 'text-slate-500'}`}>
          {plant?.connected ? 'ONLINE' : 'OFFLINE'}
        </span>
      </div>
    </div>
    <div class="plotter-toolbar__meta-divider hidden h-8 w-px bg-slate-200 dark:bg-white/10 sm:block"></div>
    <button
      onclick={() => showControllerPanel = !showControllerPanel}
      class={`p-2 rounded-lg border shadow-sm transition-all ${showControllerPanel ? 'bg-blue-600 text-white border-blue-600' : 'bg-white dark:bg-[#18181b] text-slate-500 border-slate-200 dark:border-white/10 hover:bg-slate-50 dark:hover:bg-white/5'}`}
    >
      <Sliders size={20} />
    </button>
  </div>
</div>

<style>
  @media (max-height: 900px) {
    .plotter-toolbar {
      min-height: 2.75rem;
      padding-top: 0.25rem;
      padding-bottom: 0.25rem;
      gap: 0.375rem;
    }

    .plotter-toolbar__timing {
      display: none;
    }
  }

  @media (max-height: 760px) {
    .plotter-toolbar {
      padding-left: 0.5rem;
      padding-right: 0.5rem;
    }

    .plotter-toolbar__status,
    .plotter-toolbar__meta-divider {
      display: none;
    }

    .plotter-toolbar__controls {
      gap: 0.35rem;
    }
  }
</style>
