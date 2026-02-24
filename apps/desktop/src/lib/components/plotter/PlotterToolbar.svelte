<script lang="ts">
  import { Power, Play, Pause, Home, Download, Camera, AlertTriangle, Gauge, Timer, Sliders } from 'lucide-svelte';
  import type { Plant } from '$lib/types/plant';

  let {
    plant,
    currentStats,
    showControllerPanel = $bindable(false),
    onToggleConnect,
    onTogglePause,
    onResetZoom,
    onExport,
    onPrint,
    formatTime
  }: {
    plant: Plant | undefined;
    currentStats: { errorAvg: number; stability: number; uptime: number };
    showControllerPanel: boolean;
    onToggleConnect: () => void;
    onTogglePause: () => void;
    onResetZoom: () => void;
    onExport: () => void;
    onPrint: () => void;
    formatTime: (seconds: number) => string;
  } = $props();
</script>

<div class="h-14 bg-white dark:bg-[#0c0c0e] border-b border-slate-200 dark:border-white/5 flex items-center justify-between px-6 shadow-sm z-20 print:hidden">
  <div class="flex items-center gap-3">
    <button
      onclick={onToggleConnect}
      class={`flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-bold transition-all border
        ${plant?.connected
          ? 'bg-red-50 text-red-600 border-red-200 hover:bg-red-100 dark:bg-red-900/20 dark:text-red-400 dark:border-red-900/30'
          : 'bg-emerald-50 text-emerald-600 border-emerald-200 hover:bg-emerald-100 dark:bg-emerald-900/20 dark:text-emerald-400 dark:border-emerald-900/30'}`}
    >
      <Power size={18} />
      {plant?.connected ? 'DESLIGAR' : 'LIGAR'}
    </button>
    <div class="h-8 w-px bg-slate-200 dark:bg-white/10 mx-1"></div>
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
    <button onclick={onExport} class="p-2 rounded-lg hover:bg-slate-100 dark:hover:bg-white/5 text-slate-500 transition-colors" title="Exportar CSV">
      <Download size={20} />
    </button>
    <button onclick={onPrint} class="p-2 rounded-lg hover:bg-slate-100 dark:hover:bg-white/5 text-slate-500 transition-colors" title="Imprimir">
      <Camera size={20} />
    </button>
  </div>

  <div class="flex items-center gap-6">
    {#if plant?.connected}
      <div class="hidden md:flex items-center gap-4 mr-4">
        <div class="flex flex-col items-end">
          <span class="text-[9px] font-bold text-slate-400 uppercase">Erro Médio</span>
          <div class="text-xs font-mono font-bold text-slate-600 dark:text-slate-300 flex items-center gap-1">
            <AlertTriangle size={10} class="text-amber-500" />
            {currentStats.errorAvg.toFixed(1)}%
          </div>
        </div>
        <div class="h-6 w-px bg-slate-200 dark:bg-white/10 mx-1"></div>
        <div class="flex flex-col items-end">
          <span class="text-[9px] font-bold text-slate-400 uppercase">Estabilidade</span>
          <div class="text-xs font-mono font-bold text-slate-600 dark:text-slate-300 flex items-center gap-1">
            <Gauge size={10} class="text-blue-500" />
            {currentStats.stability.toFixed(1)}%
          </div>
        </div>
        <div class="flex flex-col items-end">
          <span class="text-[9px] font-bold text-slate-400 uppercase">Uptime</span>
          <div class="text-xs font-mono font-bold text-slate-600 dark:text-slate-300 flex items-center gap-1">
            <Timer size={10} class="text-purple-500" />
            {formatTime(currentStats.uptime)}
          </div>
        </div>
      </div>
    {/if}
    <div class="flex flex-col items-end mr-2">
      <span class="text-[10px] font-bold text-slate-400 uppercase tracking-wider">Status</span>
      <div class="flex items-center gap-1.5">
        <span class={`w-2 h-2 rounded-full ${plant?.connected ? 'bg-emerald-500 animate-pulse' : 'bg-slate-400'}`}></span>
        <span class={`text-xs font-bold ${plant?.connected ? 'text-emerald-600 dark:text-emerald-400' : 'text-slate-500'}`}>
          {plant?.connected ? 'ONLINE' : 'OFFLINE'}
        </span>
      </div>
    </div>
    <div class="h-8 w-px bg-slate-200 dark:bg-white/10"></div>
    <button
      onclick={() => showControllerPanel = !showControllerPanel}
      class={`p-2 rounded-lg border shadow-sm transition-all ${showControllerPanel ? 'bg-blue-600 text-white border-blue-600' : 'bg-white dark:bg-[#18181b] text-slate-500 border-slate-200 dark:border-white/10 hover:bg-slate-50 dark:hover:bg-white/5'}`}
    >
      <Sliders size={20} />
    </button>
  </div>
</div>
