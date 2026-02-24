<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { appStore } from '$lib/stores/data.svelte';
  import { getPlantData, getPlantStats } from '$lib/stores/plantData';
  import PlotlyChart from '../charts/PlotlyChart.svelte';
  import PlantTabs from '../plotter/PlantTabs.svelte';
  import PlotterToolbar from '../plotter/PlotterToolbar.svelte';
  import ChartContextMenu from '../plotter/ChartContextMenu.svelte';
  import ControllerPanel from '../plotter/ControllerPanel.svelte';
  import RemovalModal from '../plotter/RemovalModal.svelte';
  import type { Plant } from '$lib/types/plant';

  let { plants, activePlantId, theme, showControllerPanel = $bindable(false) } = $props();

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

  const chartState = $derived(chartStates[activePlantId] ?? defaultChartState());

  let lineColors = $state({
    pv: '#3b82f6',
    sp: '#f59e0b',
    mv: '#10b981'
  });

  let contextMenu = $state({ visible: false, x: 0, y: 0 });
  let graphContainerRef: HTMLDivElement;

  let removeModal = $state({
    visible: false,
    plantId: '',
    plantName: '',
    reason: '' as 'confirm' | 'min-units'
  });

  const activePlant = $derived(plants.find((p: Plant) => p.id === activePlantId));

  function formatTime(seconds: number): string {
    if (!Number.isFinite(seconds)) return '--:--';
    const m = Math.floor(seconds / 60);
    const s = Math.floor(seconds % 60);
    return `${m}:${s.toString().padStart(2, '0')}`;
  }

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
      removeModal = {
        visible: true,
        plantId,
        plantName: plants.find((p: Plant) => p.id === plantId)?.name || '',
        reason: 'min-units'
      };
      return;
    }
    removeModal = {
      visible: true,
      plantId,
      plantName: plants.find((p: Plant) => p.id === plantId)?.name || '',
      reason: 'confirm'
    };
  }

  function confirmRemovePlant() {
    if (removeModal.reason === 'confirm') {
      appStore.removePlant(removeModal.plantId);
    }
    removeModal.visible = false;
  }

  function cancelRemovePlant() {
    removeModal.visible = false;
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

  let _displayTick = $state(0);
  let _displayTimer: ReturnType<typeof setInterval>;
  onMount(() => { _displayTimer = setInterval(() => _displayTick++, 200); });
  onDestroy(() => clearInterval(_displayTimer));

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

  const plantData = $derived(getPlantData(activePlantId));

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
    showHover: true
  });

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
    showHover: true
  });

  function handleRangeChange(xMin: number, xMax: number) {
    chartState.xMin = xMin;
    chartState.xMax = xMax;
    chartState.xMode = 'manual';
  }
</script>

<div class="flex flex-col h-full w-full bg-white dark:bg-[#09090b] text-slate-900 dark:text-white">
  <PlantTabs
    {plants}
    {activePlantId}
    {alarmState}
    onSelect={(id) => appStore.setActivePlantId(id)}
    onAdd={handleAddPlant}
    onRemove={handleRemovePlant}
  />

  <div class="flex-1 flex overflow-hidden bg-slate-50 dark:bg-[#09090b] relative">
    <div class="flex-1 flex flex-col min-w-0 relative">
      <PlotterToolbar
        plant={activePlant}
        {alarmState}
        {currentStats}
        bind:showControllerPanel
        onToggleConnect={handleToggleConnect}
        onTogglePause={handleTogglePause}
        onResetZoom={resetZoom}
        onExport={handleExportData}
        onPrint={handlePrint}
        {formatTime}
      />

      <div
        bind:this={graphContainerRef}
        class="flex-1 flex flex-col p-3 gap-3 overflow-hidden relative cursor-crosshair"
        oncontextmenu={handleContextMenu}
        ondblclick={resetZoom}
        role="application"
        aria-label="Área de gráficos"
      >
        <ChartContextMenu
          bind:visible={contextMenu.visible}
          x={contextMenu.x}
          y={contextMenu.y}
          {chartState}
          {lineColors}
          onClose={closeContextMenu}
        />

        <div class={`flex-[2] rounded-xl border relative shadow-sm overflow-hidden transition-all duration-500 group select-none ${alarmState !== 'NORMAL' ? 'bg-red-50/50 dark:bg-red-900/10 border-red-500/50' : 'bg-white dark:bg-[#0c0c0e] border-slate-200 dark:border-white/10'}`}>
          <div class="absolute top-3 right-3 z-20 pointer-events-none flex items-center gap-3 bg-white/70 dark:bg-black/50 backdrop-blur-md rounded-lg px-3.5 py-2 border border-slate-200/50 dark:border-white/10 shadow-sm">
            <div class="flex flex-col items-end">
              <span class="text-[9px] font-bold text-slate-400 dark:text-zinc-500 uppercase tracking-wider">PV</span>
              <span class="text-lg font-mono font-bold leading-tight" style="color: {lineColors.pv}">{currentPV.toFixed(2)}<span class="text-[10px] font-medium text-slate-400 ml-0.5">%</span></span>
            </div>
            <div class="w-px h-8 bg-slate-200 dark:bg-white/10"></div>
            <div class="flex flex-col items-end">
              <span class="text-[9px] font-bold text-slate-400 dark:text-zinc-500 uppercase tracking-wider">SP</span>
              <span class="text-lg font-mono font-bold leading-tight" style="color: {lineColors.sp}">{activePlant?.setpoint.toFixed(1) ?? '--'}<span class="text-[10px] font-medium text-slate-400 ml-0.5">%</span></span>
            </div>
          </div>
          <PlotlyChart 
            series={pvSpSeries} 
            config={pvSpConfig} 
            theme={theme}
            onRangeChange={handleRangeChange}
          />
        </div>

        <div class="flex-1 bg-white dark:bg-[#0c0c0e] rounded-xl border border-slate-200 dark:border-white/10 relative shadow-sm overflow-hidden group">
          <div class="absolute top-3 right-3 z-20 pointer-events-none flex items-center gap-2 bg-white/70 dark:bg-black/50 backdrop-blur-md rounded-lg px-3.5 py-2 border border-slate-200/50 dark:border-white/10 shadow-sm">
            <div class="flex flex-col items-end">
              <span class="text-[9px] font-bold text-slate-400 dark:text-zinc-500 uppercase tracking-wider">MV</span>
              <span class="text-lg font-mono font-bold leading-tight" style="color: {lineColors.mv}">{currentMV.toFixed(1)}<span class="text-[10px] font-medium text-slate-400 ml-0.5">%</span></span>
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

    <ControllerPanel
      bind:visible={showControllerPanel}
      plant={activePlant}
      onAddController={addController}
      onDeleteController={deleteController}
      onUpdateControllerMeta={updateControllerMeta}
      onUpdateControllerParam={updateControllerParam}
      onUpdateSetpoint={updateSetpoint}
      onUpdateLimits={updateLimits}
    />
  </div>

  <RemovalModal
    bind:visible={removeModal.visible}
    plantName={removeModal.plantName}
    reason={removeModal.reason}
    onConfirm={confirmRemovePlant}
    onCancel={cancelRemovePlant}
  />
</div>

