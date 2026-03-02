<script lang="ts">
  import { onMount, onDestroy, untrack } from 'svelte';
  import { appStore } from '$lib/stores/data.svelte';
  import { getPlantData, getPlantStats, getVariableStats } from '$lib/stores/plantData';
  import { exportPlantDataCSV, exportPlantDataJSON } from '$lib/services/export';
  import { formatTime } from '$lib/utils/format';
  import VariableGrid from '../charts/VariableGrid.svelte';
  import PlantTabs from '../plotter/PlantTabs.svelte';
  import PlotterToolbar from '../plotter/PlotterToolbar.svelte';
  import ChartContextMenu from '../plotter/ChartContextMenu.svelte';
  import ControllerPanel from '../plotter/ControllerPanel.svelte';
  import PlantRemovalModal from '../modals/PlantRemovalModal.svelte';
  import CreatePlantModal from '../modals/CreatePlantModal.svelte';
  import type { Plant, PlantVariable, VariableStats } from '$lib/types/plant';
  import { createDefaultVariable } from '$lib/types/plant';
  import { getVariableKeys } from '$lib/types/plant';
  import { type ChartStateType, defaultChartState, DEFAULT_LINE_COLORS, nextViewState, resetToGridView } from '$lib/types/chart';
  import { generateId } from '$lib/utils/format';
  import { openPlant } from '$lib/services/plantBackend';
  import { openFileDialog, readFileAsJSON, FILE_FILTERS } from '$lib/services/fileDialog';

  let { plants, activePlantId, theme, showControllerPanel = $bindable(false) } = $props();

  let chartStates: Record<string, ChartStateType> = $state({});

  // Inicializa chartStates para novas plantas
  // Conta apenas sensores para navegação (atuadores são plotados nos sensores)
  $effect(() => {
    for (const plant of plants) {
      const sensorCount = plant.variables.filter((v: any) => v.type === 'sensor').length;
      if (!(plant.id in chartStates)) {
        chartStates[plant.id] = defaultChartState(sensorCount);
      } else if (chartStates[plant.id].variableCount !== sensorCount) {
        // Atualiza se o número de sensores mudou
        chartStates[plant.id].variableCount = sensorCount;
      }
    }
  });

  const chartState = $derived(chartStates[activePlantId] ?? defaultChartState());

  type SeriesStyle = {
    color: string;
    visible: boolean;
    label: string;
  };

  let seriesStyles = $state<Record<string, SeriesStyle>>({});
  const actuatorPalette = ['#10b981', '#06b6d4', '#8b5cf6', '#f97316', '#ec4899', '#14b8a6'];

  let contextMenu = $state({ visible: false, x: 0, y: 0 });
  let contextSensorIndex = $state(0);
  let graphContainerRef: HTMLDivElement;

  let removeModal = $state({
    visible: false,
    plantId: '',
    plantName: '',
    reason: '' as 'confirm' | 'min-units'
  });

  let createPlantModal = $state(false);
  let openPlantLoading = $state(false);

  const activePlant = $derived(plants.find((p: Plant) => p.id === activePlantId));

  const sensorVariables = $derived(
    activePlant?.variables
      .map((variable: PlantVariable, index: number) => ({ variable, index }))
      .filter(({ variable }: { variable: PlantVariable; index: number }) => variable.type === 'sensor') ?? []
  );

  const focusedSensor = $derived.by(() => {
    if (sensorVariables.length === 0) return null;
    const safeIndex = Math.max(0, Math.min(chartState.focusedVariableIndex, sensorVariables.length - 1));
    return sensorVariables[safeIndex];
  });

  // Sensor selecionado pelo clique direito (para o menu contextual)
  const contextSensor = $derived.by(() => {
    if (sensorVariables.length === 0) return null;
    const safeIndex = Math.max(0, Math.min(contextSensorIndex, sensorVariables.length - 1));
    return sensorVariables[safeIndex];
  });

  $effect(() => {
    if (!activePlant) return;

    // untrack para evitar dependência circular (lê seriesStyles sem rastrear)
    const current = untrack(() => seriesStyles);
    const next: Record<string, SeriesStyle> = {};
    let actuatorColorIndex = 0;

    activePlant.variables.forEach((variable: PlantVariable, index: number) => {
      const keys = getVariableKeys(index);

      if (variable.type === 'sensor') {
        next[keys.pv] = current[keys.pv] ?? {
          color: DEFAULT_LINE_COLORS.pv,
          visible: true,
          label: `${variable.name} PV`,
        };
        next[keys.sp] = current[keys.sp] ?? {
          color: DEFAULT_LINE_COLORS.sp,
          visible: true,
          label: `${variable.name} SP`,
        };
      } else {
        next[keys.pv] = current[keys.pv] ?? {
          color: actuatorPalette[actuatorColorIndex % actuatorPalette.length],
          visible: true,
          label: variable.name,
        };
        actuatorColorIndex += 1;
      }
    });

    seriesStyles = next;
  });

  const contextSeriesControls = $derived.by(() => {
    if (!activePlant || !contextSensor) return [];

    const controls: { key: string; label: string; color: string; visible: boolean }[] = [];
    const sensorKeys = getVariableKeys(contextSensor.index);

    const pvStyle = seriesStyles[sensorKeys.pv];
    const spStyle = seriesStyles[sensorKeys.sp];

    controls.push({
      key: sensorKeys.pv,
      label: 'PV (Sensor)',
      color: pvStyle?.color ?? DEFAULT_LINE_COLORS.pv,
      visible: pvStyle?.visible ?? true,
    });

    controls.push({
      key: sensorKeys.sp,
      label: 'SP (Setpoint)',
      color: spStyle?.color ?? DEFAULT_LINE_COLORS.sp,
      visible: spStyle?.visible ?? true,
    });

    activePlant.variables.forEach((variable: PlantVariable, index: number) => {
      if (variable.type !== 'atuador' || !variable.linkedSensorIds?.includes(contextSensor.variable.id)) {
        return;
      }

      const actuatorKey = getVariableKeys(index).pv;
      const actuatorStyle = seriesStyles[actuatorKey];

      controls.push({
        key: actuatorKey,
        label: `${variable.name} (Atuador)`,
        color: actuatorStyle?.color ?? DEFAULT_LINE_COLORS.mv,
        visible: actuatorStyle?.visible ?? true,
      });
    });

    return controls;
  });

  const contextSeriesTitle = $derived(
    contextSensor ? `Linhas - ${contextSensor.variable.name}` : 'Linhas'
  );

  const mockDt = $derived.by(() => {
    _displayTick;
    return activePlant?.connected ? 0.1 : 0;
  });

  function toggleSeriesVisibility(key: string) {
    const current = seriesStyles[key];
    if (!current) return;
    seriesStyles = {
      ...seriesStyles,
      [key]: {
        ...current,
        visible: !current.visible,
      },
    };
  }

  function updateSeriesColor(key: string, color: string) {
    const current = seriesStyles[key];
    if (!current) return;
    seriesStyles = {
      ...seriesStyles,
      [key]: {
        ...current,
        color,
      },
    };
  }

  async function handleOpenFile() {
    openPlantLoading = true;
    try {
      // Abre seletor de arquivo usando serviço modular
      const result = await openFileDialog({
        title: 'Abrir Planta',
        filters: FILE_FILTERS.plant,
      });

      if (!result) {
        openPlantLoading = false;
        return;
      }

      // Lê e processa o arquivo
      const plantResult = await openPlant({ filePath: result.name });
      if (plantResult.success && plantResult.plant) {
        appStore.addPlant(plantResult.plant);
        appStore.setActivePlantId(plantResult.plant.id);
      } else {
        alert(plantResult.error || 'Erro ao abrir planta');
      }
    } catch (e) {
      console.error('Erro ao abrir planta:', e);
      alert('Erro ao abrir planta');
    } finally {
      openPlantLoading = false;
    }
  }

  function handleCreateNew() {
    createPlantModal = true;
  }

  function handlePlantCreated(plant: Plant) {
    appStore.addPlant(plant);
    appStore.setActivePlantId(plant.id);
    createPlantModal = false;
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

  function handleExportCSV() {
    const data = getPlantData(activePlantId);
    if (!activePlant || !exportPlantDataCSV(activePlant, data)) {
      alert('Sem dados para exportar.');
    }
  }

  function handleExportJSON() {
    const data = getPlantData(activePlantId);
    if (!activePlant || !exportPlantDataJSON(activePlant, data)) {
      alert('Sem dados para exportar.');
    }
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

  let _displayTick = $state(0);
  let _displayTimer: ReturnType<typeof setInterval>;

  function handleContextMenu(e: MouseEvent) {
    e.preventDefault();
    if (!graphContainerRef) return;

    // Detecta qual card de sensor foi clicado via data-sensor-index
    let target = e.target as HTMLElement | null;
    while (target && target !== graphContainerRef) {
      const idx = target.dataset?.sensorIndex;
      if (idx !== undefined) {
        contextSensorIndex = parseInt(idx, 10);
        break;
      }
      target = target.parentElement;
    }

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

  function updateSetpoint(varIndex: number, value: number) {
    if (!activePlant) return;
    appStore.updateVariableSetpoint(activePlant.id, varIndex, value);
  }

  // Keyboard navigation para modos de visualização
  function handleKeyDown(event: KeyboardEvent) {
    // Ignora se estiver em um input
    if (event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement) {
      return;
    }
    
    const state = chartStates[activePlantId];
    if (!state) return;
    
    if (event.code === 'Space') {
      event.preventDefault();
      nextViewState(state);
    } else if (event.code === 'KeyH') {
      event.preventDefault();
      resetToGridView(state);
    }
  }

  onMount(() => {
    _displayTimer = setInterval(() => _displayTick++, 33);
    window.addEventListener('keydown', handleKeyDown);
  });
  
  onDestroy(() => {
    clearInterval(_displayTimer);
    window.removeEventListener('keydown', handleKeyDown);
  });

  const currentStats = $derived.by(() => {
    _displayTick;
    return getPlantStats(activePlantId);
  });
  const plantData = $derived(getPlantData(activePlantId));
  
  // Stats por variável (atualiza com displayTick)
  const variableStatsArray = $derived.by(() => {
    _displayTick;
    if (!activePlant) return [];
    return activePlant.variables.map((_: unknown, idx: number) => getVariableStats(activePlantId, idx));
  });

  const pvSpConfig = $derived({
    yMin: chartState.yMin,
    yMax: chartState.yMax,
    yMode: chartState.yMode,
    xMode: chartState.xMode,
    windowSize: chartState.windowSize,
    xMin: chartState.xMin,
    xMax: chartState.xMax,
    showGrid: true,
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
    onSelect={(id) => appStore.setActivePlantId(id)}
    onOpenFile={handleOpenFile}
    onCreateNew={handleCreateNew}
    onRemove={handleRemovePlant}
  />

  <div class="flex-1 flex overflow-hidden bg-slate-50 dark:bg-[#09090b] relative">
    <div class="flex-1 flex flex-col min-w-0 relative">
      <PlotterToolbar
        plant={activePlant}
        {currentStats}
        dt={mockDt}
        bind:showControllerPanel
        onToggleConnect={handleToggleConnect}
        onTogglePause={handleTogglePause}
        onResetZoom={resetZoom}
        onExportCSV={handleExportCSV}
        onExportJSON={handleExportJSON}
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
          seriesControls={contextSeriesControls}
          seriesTitle={contextSeriesTitle}
          onToggleSeries={toggleSeriesVisibility}
          onChangeSeriesColor={updateSeriesColor}
          onClose={closeContextMenu}
        />

        {#if activePlant}
          <VariableGrid
            variables={activePlant.variables}
            data={plantData}
            pvConfig={pvSpConfig}
            {mvConfig}
            {theme}
            viewMode={chartState.viewMode}
            focusedIndex={chartState.focusedVariableIndex}
            lineStyles={seriesStyles}
            variableStats={variableStatsArray}
            onRangeChange={handleRangeChange}
          />
        {/if}
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
    />
  </div>

  <PlantRemovalModal
    bind:visible={removeModal.visible}
    plantName={removeModal.plantName}
    reason={removeModal.reason}
    onConfirm={confirmRemovePlant}
    onCancel={cancelRemovePlant}
  />

  <CreatePlantModal
    bind:visible={createPlantModal}
    onPlantCreated={handlePlantCreated}
    onClose={() => createPlantModal = false}
  />
</div>

