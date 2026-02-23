<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { appStore } from '$lib/stores/data.svelte';
  import { getPlantData, getPlantStats } from '$lib/stores/plantData';
  import { 
    Plus, X, Power, Play, Pause, Download, Camera, Home, 
    AlertTriangle, Gauge, Timer, Target, Zap, ChevronsRight, Trash2,
    Palette, Eye, EyeOff, Sliders
  } from 'lucide-svelte';
  import SimpleToggle from '../ui/SimpleToggle.svelte';
  import DynamicParamInput from '../ui/DynamicParamInput.svelte';
  import PlotlyChart from '../charts/PlotlyChart.svelte';
  import type { Plant, PlantDataPoint, Controller } from '$lib/types/plant';
  import type { ControllerParam } from '$lib/types/controller';

  let { plants, activePlantId, theme, showControllerPanel = $bindable(false) } = $props();

  // ── Per-plant chart states ──────────────────────────────────────────────
  interface ChartStateType {
    xMode: 'auto' | 'sliding' | 'manual';
    yMode: 'auto' | 'manual';
    xMin: number | null;
    xMax: number | null;
    yMin: number;
    yMax: number;
    windowSize: number;
    visible: { pv: boolean; sp: boolean; mv: boolean };
  }

  function defaultChartState(): ChartStateType {
    return {
      xMode: 'auto',
      yMode: 'manual',
      xMin: null,
      xMax: null,
      yMin: 0,
      yMax: 100,
      windowSize: 30,
      visible: { pv: true, sp: true, mv: true }
    };
  }

  // Eagerly create states for known plants; $effect handles dynamic additions
  let chartStates: Record<string, ChartStateType> = $state(
    Object.fromEntries(plants.map((p: Plant) => [p.id, defaultChartState()]))
  );

  $effect(() => {
    for (const plant of plants) {
      if (!(plant.id in chartStates)) {
        chartStates[plant.id] = defaultChartState();
      }
    }
  });

  // Active plant's chart state (reactive — mutations go to per-plant record)
  const chartState = $derived(chartStates[activePlantId] ?? defaultChartState());

  // Line Colors State
  let lineColors = $state({
    pv: '#3b82f6',
    sp: '#f59e0b',
    mv: '#10b981'
  });

  // Context Menu State
  let contextMenu = $state({ visible: false, x: 0, y: 0 });
  let graphContainerRef: HTMLDivElement;

  const activePlant = $derived(plants.find((p: Plant) => p.id === activePlantId));

  // Theme colors - only recalculate when theme changes
  const colors = $derived({
    grid: theme === 'dark' ? '#333333' : '#e5e7eb',
    gridStroke: theme === 'dark' ? 0.3 : 0.8,
    axis: theme === 'dark' ? '#6b7280' : '#9ca3af',
    text: theme === 'dark' ? '#9ca3af' : '#64748b',
    tooltipBg: theme === 'dark' ? '#18181b' : '#ffffff',
    border: theme === 'dark' ? '#27272a' : '#e2e8f0',
    limitLine: '#ef4444',
    bg: theme === 'dark' ? '#0c0c0e' : '#ffffff'
  });

  // Format time helper
  function formatTime(seconds: number): string {
    if (!Number.isFinite(seconds)) return '--:--';
    const m = Math.floor(seconds / 60);
    const s = Math.floor(seconds % 60);
    return `${m}:${s.toString().padStart(2, '0')}`;
  }

  // Handlers
  function handleAddPlant() {
    const newId = Math.random().toString(36).substr(2, 9);
    appStore.addPlant({
      id: newId,
      name: `Unidade ${plants.length + 1}`,
      connected: false,
      paused: false,
      setpoint: 50,
      limits: { high: 85, low: 15 },
      controllers: []
    });
    appStore.setActivePlantId(newId);
  }

  function handleRemovePlant(plantId: string) {
    if (plants.length <= 1) {
      alert('É necessário manter ao menos uma unidade ativa.');
      return;
    }
    if (!confirm('Confirmar remoção da unidade?')) return;
    appStore.removePlant(plantId);
  }

  function handleToggleConnect() {
    if (activePlant) {
      appStore.toggleConnect(activePlant.id);
    }
  }

  function handleTogglePause() {
    if (activePlant) {
      appStore.togglePause(activePlant.id);
    }
  }

  function handleExportData() {
    const data = getPlantData(activePlantId);
    if (!activePlant || data.length === 0) {
      alert('Sem dados para exportar.');
      return;
    }
    const csvContent =
      'data:text/csv;charset=utf-8,' +
      'Time,Setpoint,PV,MV\n' +
      data.map((e) => `${e.time.toFixed(2)},${e.sp},${e.pv.toFixed(2)},${e.mv.toFixed(2)}`).join('\n');
    const encodedUri = encodeURI(csvContent);
    const link = document.createElement('a');
    link.setAttribute('href', encodedUri);
    link.setAttribute('download', `${activePlant.name.replace(/\s+/g, '_')}_data.csv`);
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  }

  function handlePrint() {
    window.print();
  }

  function resetZoom() {
    chartState.xMode = 'auto';
    chartState.yMode = 'manual';
    chartState.yMin = 0;
    chartState.yMax = 100;
    chartState.xMin = null;
    chartState.xMax = null;
  }

  // Zoom handler - smooth zoom with center point tracking
  function handleChartZoom(factor: number, centerTime: number) {
    const data = getPlantData(activePlantId);
    if (!data.length) return;
    
    const lastTime = data[data.length - 1].time;
    
    // Get current visible range
    let currentStart: number, currentEnd: number;
    
    if (chartState.xMode === 'manual' && chartState.xMin != null && chartState.xMax != null) {
      currentStart = chartState.xMin;
      currentEnd = chartState.xMax;
    } else if (chartState.xMode === 'sliding') {
      currentEnd = lastTime;
      currentStart = Math.max(0, lastTime - chartState.windowSize);
    } else {
      // Auto mode: from 0 to current time
      currentStart = 0;
      currentEnd = lastTime;
    }
    
    // Calculate new range with smooth zoom
    const currentRange = currentEnd - currentStart;
    const newRange = currentRange * factor;
    
    // Constrain range: min 0.5s, max total data range
    const minRange = 0.5;
    const maxRange = lastTime > 0 ? lastTime : 10;
    const clampedRange = Math.max(minRange, Math.min(maxRange, newRange));
    
    // Keep zoom centered on the interaction point (usually mouse position)
    // If centerTime is outside range, use range center
    const effectiveCenter = centerTime >= currentStart && centerTime <= currentEnd ? centerTime : (currentStart + currentEnd) / 2;
    
    // Calculate ratio: how far from left edge is the center?
    const leftRatio = (effectiveCenter - currentStart) / currentRange;
    
    // Apply new range while keeping center point in same relative position
    const newStart = Math.max(0, effectiveCenter - clampedRange * leftRatio);
    const newEnd = Math.min(lastTime, newStart + clampedRange);
    
    // Adjust if we hit boundaries
    chartState.xMin = newEnd - clampedRange < 0 ? 0 : newStart;
    chartState.xMax = newStart + clampedRange > lastTime ? lastTime : newEnd;
    chartState.xMode = 'manual';
  }

  // Pan handler - smooth movement with boundary constraints
  function handleChartPan(deltaTime: number) {
    const data = getPlantData(activePlantId);
    if (!data.length) return;
    
    const lastTime = data[data.length - 1].time;
    
    // Get current visible range
    let currentStart: number, currentEnd: number;
    
    if (chartState.xMode === 'manual' && chartState.xMin != null && chartState.xMax != null) {
      currentStart = chartState.xMin;
      currentEnd = chartState.xMax;
    } else if (chartState.xMode === 'sliding') {
      currentEnd = lastTime;
      currentStart = Math.max(0, lastTime - chartState.windowSize);
    } else {
      // Auto mode: from 0 to current time
      currentStart = 0;
      currentEnd = lastTime;
    }
    
    const currentRange = currentEnd - currentStart;
    
    // Apply pan with smooth movement
    let newStart = currentStart + deltaTime;
    let newEnd = currentEnd + deltaTime;
    
    // Boundary constraints: keep within data bounds
    if (newStart < 0) {
      newStart = 0;
      newEnd = currentRange;
    } else if (newEnd > lastTime) {
      newEnd = lastTime;
      newStart = lastTime - currentRange;
    }
    
    // Only update if values actually changed (to avoid unnecessary re-renders)
    if (newStart !== chartState.xMin || newEnd !== chartState.xMax) {
      chartState.xMin = newStart;
      chartState.xMax = newEnd;
      chartState.xMode = 'manual';
    }
  }

  function handleContextMenu(e: MouseEvent) {
    e.preventDefault();
    if (!graphContainerRef) return;
    const bounds = graphContainerRef.getBoundingClientRect();
    const menuWidth = 250;
    const menuHeight = 360;
    let x = e.clientX - bounds.left;
    let y = e.clientY - bounds.top;
    if (x + menuWidth > bounds.width) x -= menuWidth;
    if (y + menuHeight > bounds.height) y -= menuHeight;
    contextMenu = { visible: true, x, y };
  }

  function closeContextMenu() {
    contextMenu.visible = false;
  }

  // Controller handlers
  function addController() {
    if (!activePlant) return;
    appStore.addController(activePlant.id, {
      name: 'Nova Malha',
      type: 'PID',
      active: false,
      params: {
        kp: { type: 'number', value: 1.0, label: 'Kp' },
        ki: { type: 'number', value: 0.0, label: 'Ki' },
        kd: { type: 'number', value: 0.0, label: 'Kd' },
        manualMode: { type: 'boolean', value: false, label: 'Manual' }
      }
    });
  }

  function deleteController(controllerId: string) {
    if (!activePlant) return;
    appStore.deleteController(activePlant.id, controllerId);
  }

  function updateControllerMeta(controllerId: string, field: string, value: any) {
    if (!activePlant) return;
    appStore.updateControllerMeta(activePlant.id, controllerId, field, value);
  }

  function updateControllerParam(controllerId: string, paramKey: string, value: any) {
    if (!activePlant) return;
    appStore.updateControllerParam(activePlant.id, controllerId, paramKey, value);
  }

  function updateSetpoint(value: number) {
    if (!activePlant) return;
    appStore.updateSetpoint(activePlant.id, value);
  }

  function updateLimits(field: 'high' | 'low', value: number) {
    if (!activePlant) return;
    appStore.updateLimits(activePlant.id, { ...activePlant.limits, [field]: value });
  }

  // ── Display tick: 5 Hz counter drives DOM-visible values (PV, MV, stats)
  //    without coupling to the simulation's 10 Hz data pushes. ──
  let _displayTick = $state(0);
  let _displayTimer: ReturnType<typeof setInterval>;
  onMount(() => { _displayTimer = setInterval(() => _displayTick++, 200); });
  onDestroy(() => clearInterval(_displayTimer));

  // Current values - refreshed at 5 Hz via _displayTick
  const currentPV = $derived.by(() => {
    _displayTick;
    const data = getPlantData(activePlantId);
    return data.length > 0 ? data[data.length - 1].pv : 0;
  });
  const currentMV = $derived.by(() => {
    _displayTick;
    const data = getPlantData(activePlantId);
    return data.length > 0 ? data[data.length - 1].mv : 0;
  });
  const currentStats = $derived.by(() => {
    _displayTick;
    return getPlantStats(activePlantId);
  });
  const alarmState = $derived.by(() => {
    _displayTick;
    if (!activePlant) return 'NORMAL';
    const data = getPlantData(activePlantId);
    const pv = data.length > 0 ? data[data.length - 1].pv : 0;
    return pv > activePlant.limits.high ? 'HIGH' :
           pv < activePlant.limits.low ? 'LOW' : 'NORMAL';
  });

  // Plain array reference for charts — stable per-plant, no proxy overhead.
  const plantData = $derived(getPlantData(activePlantId));
  
  // Chart data series for PV/SP chart - stable references
  const pvSpSeries = $derived([
    {
      key: 'pv',
      label: 'PV (Process Variable)',
      color: lineColors.pv,
      visible: chartState.visible.pv,
      data: plantData,
      dataKey: 'pv' as const,
      type: 'line' as const,
      strokeWidth: 2
    },
    {
      key: 'sp',
      label: 'SP (Setpoint)',
    color: lineColors.sp,
      visible: chartState.visible.sp,
      data: plantData,
      dataKey: 'sp' as const,
      type: 'step' as const,
      strokeWidth: 1.5,
      dashed: true
    }
  ]);

  // Chart data series for MV chart - stable reference
  const mvSeries = $derived([
    {
      key: 'mv',
      label: 'MV (Output)',
      color: lineColors.mv,
      visible: chartState.visible.mv,
      data: plantData,
      dataKey: 'mv' as const,
      type: 'area' as const,
      strokeWidth: 1.5
    }
  ]);

  // Chart config for PV/SP
  const pvSpConfig = $derived({
    yMin: chartState.yMin,
    yMax: chartState.yMax,
    yMode: chartState.yMode,
    xMode: chartState.xMode,
    windowSize: chartState.windowSize,
    xMin: chartState.xMin,
    xMax: chartState.xMax,
    showGrid: true,
    showLimits: true,
    limitHigh: activePlant?.limits?.high,
    limitLow: activePlant?.limits?.low,
    showHover: true,
    showLegend: false,
    showModeBar: false
  });

  // Chart config for MV
  const mvConfig = $derived({
    yMin: 0,
    yMax: 100,
    yMode: 'manual' as const,
    xMode: chartState.xMode,
    windowSize: chartState.windowSize,
    xMin: chartState.xMin,
    xMax: chartState.xMax,
    showGrid: true,
    showLimits: false,
    showHover: true,
    showLegend: false,
    showModeBar: false
  });

  // Handler for Plotly range change events
  function handleRangeChange(xMin: number, xMax: number) {
    chartState.xMin = xMin;
    chartState.xMax = xMax;
    chartState.xMode = 'manual';
  }
</script>

<div class="flex flex-col h-full w-full bg-white dark:bg-[#09090b] text-slate-900 dark:text-white">
  <!-- Header Tabs -->
  <header class="h-10 bg-white dark:bg-[#0c0c0e] border-b border-slate-200 dark:border-white/5 flex items-end px-4 gap-2 select-none z-10 print:hidden">
    {#each plants as plant (plant.id)}
      <div class="group relative flex items-center h-8 min-w-[140px] max-w-[200px]">
        <button
          onclick={() => appStore.setActivePlantId(plant.id)}
          class={`w-full h-full pl-3 pr-8 rounded-t-lg text-xs font-medium cursor-pointer transition-all border-t border-x flex items-center gap-2
            ${activePlantId === plant.id
              ? 'bg-slate-50 dark:bg-[#18181b] border-slate-300 dark:border-white/10 text-blue-600 dark:text-blue-400 border-b-slate-50 dark:border-b-[#18181b] mb-[-1px]'
              : 'bg-transparent border-transparent text-slate-500 hover:bg-slate-100 dark:hover:bg-white/5 mb-0'}`}
        >
          <div class={`w-1.5 h-1.5 rounded-full ${alarmState !== 'NORMAL' && plant.id === activePlantId ? 'bg-red-500 animate-pulse' : plant.connected ? 'bg-emerald-500' : 'bg-slate-300 dark:bg-zinc-700'}`}></div>
          <span class="truncate">{plant.name}</span>
        </button>
        <button
          onclick={(e: MouseEvent) => { e.stopPropagation(); handleRemovePlant(plant.id); }}
          class="absolute right-1 top-1/2 -translate-y-1/2 p-1 rounded opacity-0 group-hover:opacity-100 hover:bg-red-100 dark:hover:bg-red-900/30 hover:text-red-600 transition-all"
        >
          <X size={12} strokeWidth={2.5} />
        </button>
      </div>
    {/each}
    <button onclick={handleAddPlant} class="h-7 w-7 mb-0.5 flex items-center justify-center rounded-lg hover:bg-slate-100 dark:hover:bg-white/5 text-slate-500 transition-colors">
      <Plus size={16} />
    </button>
  </header>

  <!-- Main Area -->
  <div class="flex-1 flex overflow-hidden bg-slate-50 dark:bg-[#09090b] relative">
    <div class="flex-1 flex flex-col min-w-0 relative">
      
      <!-- Toolbar -->
      <div class="h-14 bg-white dark:bg-[#0c0c0e] border-b border-slate-200 dark:border-white/5 flex items-center justify-between px-6 shadow-sm z-20 print:hidden">
        <div class="flex items-center gap-3">
          <button
            onclick={handleToggleConnect}
            class={`flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-bold transition-all border
              ${activePlant?.connected
                ? 'bg-red-50 text-red-600 border-red-200 hover:bg-red-100 dark:bg-red-900/20 dark:text-red-400 dark:border-red-900/30'
                : 'bg-emerald-50 text-emerald-600 border-emerald-200 hover:bg-emerald-100 dark:bg-emerald-900/20 dark:text-emerald-400 dark:border-emerald-900/30'}`}
          >
            <Power size={18} />
            {activePlant?.connected ? 'DESLIGAR' : 'LIGAR'}
          </button>
          <div class="h-8 w-px bg-slate-200 dark:bg-white/10 mx-1"></div>
          <button
            onclick={handleTogglePause}
            disabled={!activePlant?.connected}
            class="p-2 rounded-lg hover:bg-slate-100 dark:hover:bg-white/5 text-slate-500 disabled:opacity-30 transition-colors"
            title={activePlant?.paused ? 'Retomar' : 'Pausar'}
          >
            {#if activePlant?.paused}
              <Play size={20} class="text-blue-500" fill="currentColor" />
            {:else}
              <Pause size={20} />
            {/if}
          </button>
          <button onclick={resetZoom} class="p-2 rounded-lg hover:bg-slate-100 dark:hover:bg-white/5 text-slate-500 transition-colors" title="Home (Ver Tudo)">
            <Home size={20} />
          </button>
          <button onclick={handleExportData} class="p-2 rounded-lg hover:bg-slate-100 dark:hover:bg-white/5 text-slate-500 transition-colors" title="Exportar CSV">
            <Download size={20} />
          </button>
          <button onclick={handlePrint} class="p-2 rounded-lg hover:bg-slate-100 dark:hover:bg-white/5 text-slate-500 transition-colors" title="Imprimir">
            <Camera size={20} />
          </button>
        </div>

        <div class="flex items-center gap-6">
          {#if activePlant?.connected}
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
              <span class={`w-2 h-2 rounded-full ${alarmState === 'HIGH' || alarmState === 'LOW' ? 'bg-red-500 animate-ping' : activePlant?.connected ? 'bg-emerald-500 animate-pulse' : 'bg-slate-400'}`}></span>
              <span class={`text-xs font-bold ${alarmState !== 'NORMAL' ? 'text-red-500' : activePlant?.connected ? 'text-emerald-600 dark:text-emerald-400' : 'text-slate-500'}`}>
                {alarmState !== 'NORMAL' ? `ALARM ${alarmState}` : activePlant?.connected ? 'ONLINE' : 'OFFLINE'}
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

      <!-- Charts Area -->
      <div
        bind:this={graphContainerRef}
        class="flex-1 flex flex-col p-3 gap-3 overflow-hidden relative cursor-crosshair"
        oncontextmenu={handleContextMenu}
        ondblclick={resetZoom}
        role="application"
        aria-label="Área de gráficos"
      >
        <!-- Context Menu -->
        {#if contextMenu.visible}
          <div
            class="absolute z-50 bg-white dark:bg-[#18181b] border border-slate-200 dark:border-white/10 rounded-lg shadow-2xl p-3 min-w-[240px] flex flex-col gap-2"
            style="top: {contextMenu.y}px; left: {contextMenu.x}px"
            onclick={(e: MouseEvent) => e.stopPropagation()}
            onkeydown={(e: KeyboardEvent) => e.key === 'Escape' && closeContextMenu()}
            onmouseleave={() => setTimeout(closeContextMenu, 1000)}
            role="menu"
            tabindex="-1"
          >
            <!-- X Axis -->
            <div>
              <div class="px-1 text-[10px] font-bold uppercase text-slate-400 tracking-wider mb-1 flex justify-between items-center">
                Eixo X (Tempo) <span class="text-[9px] bg-slate-100 dark:bg-white/5 px-1 rounded">{chartState.xMode}</span>
              </div>
              <div class="flex gap-1 mb-1">
                <button onclick={() => chartState.xMode = 'auto'} class={`flex-1 text-[10px] font-bold py-1 px-2 rounded border transition-colors ${chartState.xMode === 'auto' ? 'bg-blue-600 text-white border-blue-600' : 'bg-slate-50 dark:bg-white/5 text-slate-500 border-slate-200 dark:border-white/10 hover:bg-slate-100 dark:hover:bg-white/10'}`}>Auto</button>
                <button onclick={() => chartState.xMode = 'sliding'} class={`flex-1 text-[10px] font-bold py-1 px-2 rounded border transition-colors ${chartState.xMode === 'sliding' ? 'bg-blue-600 text-white border-blue-600' : 'bg-slate-50 dark:bg-white/5 text-slate-500 border-slate-200 dark:border-white/10 hover:bg-slate-100 dark:hover:bg-white/10'}`}>Janela</button>
                <button onclick={() => chartState.xMode = 'manual'} class={`flex-1 text-[10px] font-bold py-1 px-2 rounded border transition-colors ${chartState.xMode === 'manual' ? 'bg-blue-600 text-white border-blue-600' : 'bg-slate-50 dark:bg-white/5 text-slate-500 border-slate-200 dark:border-white/10 hover:bg-slate-100 dark:hover:bg-white/10'}`}>Manual</button>
              </div>
              {#if chartState.xMode === 'sliding'}
                <div class="flex items-center gap-2 px-1">
                  <span class="text-xs text-slate-500">Janela (s):</span>
                  <input type="number" class="w-16 h-6 text-xs bg-slate-50 dark:bg-black/20 border border-slate-200 dark:border-white/10 rounded px-1" value={chartState.windowSize} oninput={(e: Event) => chartState.windowSize = Number((e.target as HTMLInputElement).value)} />
                </div>
              {/if}
            </div>
            <div class="border-t border-slate-100 dark:border-white/5"></div>
            <!-- Variables -->
            <div>
              <div class="px-1 text-[10px] font-bold uppercase text-slate-400 tracking-wider mb-1 flex items-center gap-2">
                <Palette size={12} /> Variáveis
              </div>
              <div class="space-y-1">
                <!-- PV -->
                <div class="flex items-center justify-between px-2 py-1.5 rounded hover:bg-slate-100 dark:hover:bg-white/5 group">
                  <div class="flex items-center gap-2">
                    <button onclick={() => chartState.visible.pv = !chartState.visible.pv} class="text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 transition-colors">
                      {#if chartState.visible.pv}<Eye size={14} />{:else}<EyeOff size={14} />{/if}
                    </button>
                    <span class="text-xs text-slate-500 dark:text-slate-400 font-medium">PV (Process)</span>
                  </div>
                  <div class="flex items-center gap-2">
                    <input type="text" value={lineColors.pv} oninput={(e: Event) => lineColors.pv = (e.target as HTMLInputElement).value} class="w-16 h-5 text-[10px] font-mono bg-transparent border border-slate-200 dark:border-white/10 rounded px-1 text-slate-600 dark:text-slate-300 focus:outline-none focus:border-blue-500 text-right uppercase" />
                    <div class="relative w-5 h-5 rounded-full overflow-hidden border border-slate-200 dark:border-white/20 shadow-sm cursor-pointer hover:scale-110 transition-transform" style="background-color: {lineColors.pv}">
                      <input type="color" value={lineColors.pv} oninput={(e: Event) => lineColors.pv = (e.target as HTMLInputElement).value} class="absolute -top-1/2 -left-1/2 w-[200%] h-[200%] cursor-pointer p-0 m-0 border-0 opacity-0" />
                    </div>
                  </div>
                </div>
                <!-- SP -->
                <div class="flex items-center justify-between px-2 py-1.5 rounded hover:bg-slate-100 dark:hover:bg-white/5 group">
                  <div class="flex items-center gap-2">
                    <button onclick={() => chartState.visible.sp = !chartState.visible.sp} class="text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 transition-colors">
                      {#if chartState.visible.sp}<Eye size={14} />{:else}<EyeOff size={14} />{/if}
                    </button>
                    <span class="text-xs text-slate-500 dark:text-slate-400 font-medium">SP (Setpoint)</span>
                  </div>
                  <div class="flex items-center gap-2">
                    <input type="text" value={lineColors.sp} oninput={(e: Event) => lineColors.sp = (e.target as HTMLInputElement).value} class="w-16 h-5 text-[10px] font-mono bg-transparent border border-slate-200 dark:border-white/10 rounded px-1 text-slate-600 dark:text-slate-300 focus:outline-none focus:border-blue-500 text-right uppercase" />
                    <div class="relative w-5 h-5 rounded-full overflow-hidden border border-slate-200 dark:border-white/20 shadow-sm cursor-pointer hover:scale-110 transition-transform" style="background-color: {lineColors.sp}">
                      <input type="color" value={lineColors.sp} oninput={(e: Event) => lineColors.sp = (e.target as HTMLInputElement).value} class="absolute -top-1/2 -left-1/2 w-[200%] h-[200%] cursor-pointer p-0 m-0 border-0 opacity-0" />
                    </div>
                  </div>
                </div>
                <!-- MV -->
                <div class="flex items-center justify-between px-2 py-1.5 rounded hover:bg-slate-100 dark:hover:bg-white/5 group">
                  <div class="flex items-center gap-2">
                    <button onclick={() => chartState.visible.mv = !chartState.visible.mv} class="text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 transition-colors">
                      {#if chartState.visible.mv}<Eye size={14} />{:else}<EyeOff size={14} />{/if}
                    </button>
                    <span class="text-xs text-slate-500 dark:text-slate-400 font-medium">MV (Output)</span>
                  </div>
                  <div class="flex items-center gap-2">
                    <input type="text" value={lineColors.mv} oninput={(e: Event) => lineColors.mv = (e.target as HTMLInputElement).value} class="w-16 h-5 text-[10px] font-mono bg-transparent border border-slate-200 dark:border-white/10 rounded px-1 text-slate-600 dark:text-slate-300 focus:outline-none focus:border-blue-500 text-right uppercase" />
                    <div class="relative w-5 h-5 rounded-full overflow-hidden border border-slate-200 dark:border-white/20 shadow-sm cursor-pointer hover:scale-110 transition-transform" style="background-color: {lineColors.mv}">
                      <input type="color" value={lineColors.mv} oninput={(e: Event) => lineColors.mv = (e.target as HTMLInputElement).value} class="absolute -top-1/2 -left-1/2 w-[200%] h-[200%] cursor-pointer p-0 m-0 border-0 opacity-0" />
                    </div>
                  </div>
                </div>
              </div>
            </div>
            <div class="border-t border-slate-100 dark:border-white/5"></div>
            <!-- Y Axis -->
            <div>
              <div class="px-1 text-[10px] font-bold uppercase text-slate-400 tracking-wider mb-1 flex justify-between items-center">
                Eixo Y <span class="text-[9px] bg-slate-100 dark:bg-white/5 px-1 rounded">{chartState.yMode}</span>
              </div>
              <div class="flex gap-1 mb-2">
                <button onclick={() => chartState.yMode = 'auto'} class={`flex-1 text-[10px] font-bold py-1 px-2 rounded border transition-colors ${chartState.yMode === 'auto' ? 'bg-blue-600 text-white border-blue-600' : 'bg-slate-50 dark:bg-white/5 text-slate-500 border-slate-200 dark:border-white/10 hover:bg-slate-100 dark:hover:bg-white/10'}`}>Auto</button>
                <button onclick={() => chartState.yMode = 'manual'} class={`flex-1 text-[10px] font-bold py-1 px-2 rounded border transition-colors ${chartState.yMode === 'manual' ? 'bg-blue-600 text-white border-blue-600' : 'bg-slate-50 dark:bg-white/5 text-slate-500 border-slate-200 dark:border-white/10 hover:bg-slate-100 dark:hover:bg-white/10'}`}>Manual</button>
              </div>
              {#if chartState.yMode === 'manual'}
                <div class="flex gap-2 px-1">
                  <input type="number" placeholder="Min" class="w-full h-6 text-xs bg-slate-50 dark:bg-black/20 border border-slate-200 dark:border-white/10 rounded px-1" value={chartState.yMin} oninput={(e: Event) => chartState.yMin = Number((e.target as HTMLInputElement).value)} />
                  <input type="number" placeholder="Max" class="w-full h-6 text-xs bg-slate-50 dark:bg-black/20 border border-slate-200 dark:border-white/10 rounded px-1" value={chartState.yMax} oninput={(e: Event) => chartState.yMax = Number((e.target as HTMLInputElement).value)} />
                </div>
              {/if}
            </div>
          </div>
        {/if}

        <!-- Chart 1: PV/SP -->
        <div class={`flex-[2] rounded-xl border relative shadow-sm overflow-hidden transition-all duration-500 group select-none ${alarmState !== 'NORMAL' ? 'bg-red-50/50 dark:bg-red-900/10 border-red-500/50' : 'bg-white dark:bg-[#0c0c0e] border-slate-200 dark:border-white/10'}`}>
          <div class="absolute top-4 left-4 z-20 pointer-events-none flex flex-col gap-1">
            <div class="flex items-center gap-2 text-slate-400 uppercase text-[10px] font-bold tracking-widest">
              <Target size={12} /> Process Variable
            </div>
            <div class="flex items-baseline gap-2">
              <span class="text-3xl font-mono font-bold tracking-tight" style="color: {lineColors.pv}">{currentPV.toFixed(2)}</span>
              <span class="text-xs font-medium text-slate-400">%</span>
            </div>
          </div>
          <PlotlyChart 
            series={pvSpSeries} 
            config={pvSpConfig} 
            theme={theme}
            onRangeChange={handleRangeChange}
          />
        </div>

        <!-- Chart 2: MV -->
        <div class="flex-1 bg-white dark:bg-[#0c0c0e] rounded-xl border border-slate-200 dark:border-white/10 relative shadow-sm overflow-hidden group">
          <div class="absolute top-3 left-4 z-20 pointer-events-none">
            <div class="flex items-center gap-2 text-[10px] font-bold uppercase tracking-widest mb-0.5" style="color: {lineColors.mv}">
              <Zap size={12} /> Output (MV)
            </div>
            <div class="text-xl font-mono font-bold" style="color: {lineColors.mv}">
              {currentMV.toFixed(1)}<span class="text-xs opacity-60 ml-1">%</span>
            </div>
          </div>
          <PlotlyChart 
            series={mvSeries} 
            config={mvConfig} 
            theme={theme}
            onRangeChange={handleRangeChange}
          />
        </div>
      </div>
    </div>

    <!-- Controller Panel -->
    <div class={`${showControllerPanel ? 'w-80 translate-x-0' : 'w-0 translate-x-full'} bg-white dark:bg-[#0c0c0e] border-l border-slate-200 dark:border-white/5 flex flex-col transition-all duration-300 ease-in-out shadow-xl relative z-30 print:hidden`}>
      <div class="h-14 border-b border-slate-100 dark:border-white/5 flex justify-between items-center px-5 bg-slate-50 dark:bg-white/[0.02]">
        <h3 class="font-bold text-slate-700 dark:text-white text-sm">Malhas de Controle</h3>
        <button onclick={() => showControllerPanel = false} class="text-slate-400 hover:text-slate-600 dark:hover:text-white" title="Recolher Painel">
          <ChevronsRight size={20} />
        </button>
      </div>
      <div class="flex-1 overflow-y-auto p-5 space-y-6 min-w-[320px]">
        {#if activePlant}
          <!-- Quick Setpoint -->
          <div class="bg-slate-50 dark:bg-[#121215] rounded-xl p-4 border border-slate-200 dark:border-white/5 shadow-sm">
            <div class="flex justify-between items-end mb-2">
              <span class="text-xs font-bold text-slate-500 uppercase tracking-wide">Setpoint</span>
              <span class="text-xl font-mono font-bold text-blue-600 dark:text-blue-400">{activePlant.setpoint.toFixed(1)}%</span>
            </div>
            <input
              type="range"
              min="0"
              max="100"
              step="0.5"
              value={activePlant.setpoint}
              onchange={(e: Event) => updateSetpoint(Number((e.target as HTMLInputElement).value))}
              class="w-full h-1.5 bg-slate-300 dark:bg-zinc-700 rounded-lg appearance-none cursor-pointer accent-blue-600"
            />
          </div>
          
          <!-- Alarm Limits -->
          <div class="bg-red-50/50 dark:bg-red-900/10 rounded-xl p-4 border border-red-100 dark:border-red-900/20 shadow-sm">
            <div class="text-xs font-bold text-red-500 uppercase tracking-wide mb-3 flex items-center gap-2">
              <AlertTriangle size={12} /> Limites de Alarme
            </div>
            <div class="flex gap-4">
              <div class="flex-1">
                <label for="limit-high" class="text-[10px] text-slate-500 uppercase">High (HI)</label>
                <input
                  id="limit-high"
                  type="number"
                  value={activePlant.limits.high}
                  oninput={(e: Event) => updateLimits('high', parseFloat((e.target as HTMLInputElement).value))}
                  class="w-full bg-white dark:bg-zinc-800 border border-slate-200 dark:border-white/10 rounded px-2 py-1 text-xs text-right font-mono"
                />
              </div>
              <div class="flex-1">
                <label for="limit-low" class="text-[10px] text-slate-500 uppercase">Low (LO)</label>
                <input
                  id="limit-low"
                  type="number"
                  value={activePlant.limits.low}
                  oninput={(e: Event) => updateLimits('low', parseFloat((e.target as HTMLInputElement).value))}
                  class="w-full bg-white dark:bg-zinc-800 border border-slate-200 dark:border-white/10 rounded px-2 py-1 text-xs text-right font-mono"
                />
              </div>
            </div>
          </div>
          
          <div class="border-t border-slate-100 dark:border-white/5"></div>
          
          <!-- Controllers -->
          <div>
            <div class="flex justify-between items-center mb-4">
              <span class="text-xs font-bold text-slate-500 uppercase">Controladores</span>
              <button onclick={addController} class="text-xs font-medium bg-blue-50 text-blue-600 hover:bg-blue-100 dark:bg-blue-900/20 dark:text-blue-400 dark:hover:bg-blue-900/30 px-3 py-1.5 rounded-full transition-colors">
                + Adicionar
              </button>
            </div>
            <div class="space-y-4">
              {#each activePlant.controllers as ctrl (ctrl.id)}
                <div class="border border-slate-200 dark:border-white/10 rounded-xl overflow-hidden shadow-sm bg-white dark:bg-[#0c0c0e]">
                  <div class="bg-slate-50 dark:bg-white/[0.02] p-3 border-b border-slate-100 dark:border-white/5 flex items-center justify-between">
                    <div class="flex items-center gap-3">
                      <SimpleToggle checked={ctrl.active} ariaLabel="Toggle controller" onchange={() => updateControllerMeta(ctrl.id, 'active', !ctrl.active)} />
                      <input
                        value={ctrl.name}
                        oninput={(e: Event) => updateControllerMeta(ctrl.id, 'name', (e.target as HTMLInputElement).value)}
                        class="bg-transparent text-sm font-semibold text-slate-700 dark:text-zinc-200 w-32 focus:text-blue-600 dark:focus:text-blue-400 transition-colors"
                        style="border: none; outline: none; box-shadow: none;"
                      />
                    </div>
                    <button onclick={() => deleteController(ctrl.id)} class="text-slate-400 hover:text-red-500 p-1">
                      <Trash2 size={14} />
                    </button>
                  </div>
                  <div class={`p-4 space-y-3 ${ctrl.active ? '' : 'opacity-40 pointer-events-none'}`}>
                    {#each Object.entries(ctrl.params) as [key, param]}
                      <DynamicParamInput
                        label={(param as ControllerParam).label || key}
                        type={(param as ControllerParam).type}
                        value={(param as ControllerParam).value}
                        onChange={(newValue: any) => updateControllerParam(ctrl.id, key, newValue)}
                      />
                    {/each}
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>

