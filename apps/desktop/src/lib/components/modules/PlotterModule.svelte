<script lang="ts">
  import { untrack } from 'svelte';
  import { appStore } from '$lib/stores/data.svelte';
  import {
    clearPlant,
    clearVariableStats,
    getPlantData,
    getPlantSeriesCatalog,
    getPlantStats,
    getVariableStats,
    seedPlantSeriesCatalog,
    setPlantData,
    setPlantSeriesCatalog,
    setPlantStats,
    setVariableStats,
  } from '$lib/stores/plantData';
  import { exportPlantDataCSV, exportPlantDataJSON } from '$lib/services/export';
  import { formatTime } from '$lib/utils/format';
  import VariableGrid from '../charts/VariableGrid.svelte';
  import PlantTabs from '../plotter/PlantTabs.svelte';
  import PlotterToolbar from '../plotter/PlotterToolbar.svelte';
  import ChartContextMenu from '../plotter/ChartContextMenu.svelte';
  import ControllerPanel from '../plotter/ControllerPanel.svelte';
  import ControllerLibraryModal from '../modals/ControllerLibraryModal.svelte';
  import ControllerBindingsModal from '../modals/ControllerBindingsModal.svelte';
  import PlantRemovalModal from '../modals/PlantRemovalModal.svelte';
  import CreatePlantModal from '../modals/CreatePlantModal.svelte';
  import type { Plant, PlantVariable } from '$lib/types/plant';
  import { buildPlantSeriesCatalog, type Controller } from '$lib/types/plant';
  import type { PluginDefinition, PluginInstance } from '$lib/types/plugin';
  import { type ChartStateType, defaultChartState, nextViewState, resetToGridView } from '$lib/types/chart';
  import {
    connectPlant,
    disconnectPlant,
    openPlant,
    pausePlant,
    removePlant,
    resumePlant,
    saveControllerInstanceConfig,
  } from '$lib/services/plant';
  import { createConfiguredController } from '$lib/services/plugin';
  import { openFileDialog, FILE_FILTERS } from '$lib/services/fileDialog';
  import { getControllerActivationConflict } from '$lib/utils/controllerAssignments';
  import { buildContextSeriesControls, buildSeriesStyles, type SeriesStyle } from '$lib/utils/plotterSeries';
  import PluginInstanceConfigModal from '../modals/PluginInstanceConfigModal.svelte';

  let { plants, activePlantId, theme, active = true, showControllerPanel = $bindable(false) } = $props();

  interface ActiveVariableGroups {
    sensorEntries: Array<{ variable: PlantVariable; index: number }>;
    sensorVariables: PlantVariable[];
    actuatorVariables: PlantVariable[];
  }

  const EMPTY_VARIABLE_GROUPS: ActiveVariableGroups = {
    sensorEntries: [],
    sensorVariables: [],
    actuatorVariables: [],
  };

  let chartStates: Record<string, ChartStateType> = $state({});
  let manualRangesByPlant: Record<string, Record<number, { xMin: number; xMax: number }>> = $state({});

  function countSensors(variables: PlantVariable[]): number {
    let count = 0;
    for (const variable of variables) {
      if (variable.type === 'sensor') count += 1;
    }
    return count;
  }

  $effect(() => {
    for (const plant of plants) {
      const sensorCount = countSensors(plant.variables);
      if (!(plant.id in chartStates)) {
        chartStates[plant.id] = defaultChartState(sensorCount);
      } else if (chartStates[plant.id].variableCount !== sensorCount) {
        chartStates[plant.id].variableCount = sensorCount;
      }

      if (!(plant.id in manualRangesByPlant)) {
        manualRangesByPlant[plant.id] = {};
      }
    }
  });

  const chartState = $derived(chartStates[activePlantId] ?? defaultChartState());
  const activeManualRanges = $derived(manualRangesByPlant[activePlantId] ?? {});

  let seriesStyles = $state<Record<string, SeriesStyle>>({});

  let contextMenu = $state({ visible: false, x: 0, y: 0 });
  let contextSensorIndex = $state(0);
  let graphContainerRef = $state<HTMLDivElement | undefined>(undefined);

  let removeModal = $state({
    visible: false,
    plantId: '',
    plantName: '',
    reason: '' as 'confirm' | 'min-units'
  });

  let createPlantModal = $state(false);
  let editPlantModal = $state(false);
  let controllerLibraryModal = $state(false);
  let controllerConfigModal = $state(false);
  let controllerPluginToConfig = $state<PluginDefinition | null>(null);
  let controllerBindingsModal = $state(false);
  let controllerToEditBindings = $state<Controller | null>(null);
  let openPlantLoading = $state(false);
  let dragOverlay = $state(false);
  let dragDepth = $state(0);
  let plantActionLoading = $state<'connect' | 'pause' | 'remove' | null>(null);

  const activePlant = $derived(plants.find((p: Plant) => p.id === activePlantId));

  $effect(() => {
    for (const plant of plants) {
      seedPlantSeriesCatalog(buildPlantSeriesCatalog(plant.id, plant.variables));
    }
  });

  const activeVariableGroups = $derived.by<ActiveVariableGroups>(() => {
    if (!activePlant) return EMPTY_VARIABLE_GROUPS;

    const sensorEntries: ActiveVariableGroups['sensorEntries'] = [];
    const sensorVariables: PlantVariable[] = [];
    const actuatorVariables: PlantVariable[] = [];

    for (const [index, variable] of activePlant.variables.entries()) {
      if (variable.type === 'sensor') {
        sensorEntries.push({ variable, index });
        sensorVariables.push(variable);
      } else if (variable.type === 'atuador') {
        actuatorVariables.push(variable);
      }
    }

    return { sensorEntries, sensorVariables, actuatorVariables };
  });

  const sensorVariables = $derived(activeVariableGroups.sensorEntries);
  const controllerSensorVariables = $derived(activeVariableGroups.sensorVariables);
  const controllerActuatorVariables = $derived(activeVariableGroups.actuatorVariables);

  const activeSeriesCatalog = $derived.by(() => {
    _displayTick;
    return activePlant ? getPlantSeriesCatalog(activePlant.id) : [];
  });

  const activeSeriesCatalogByKey = $derived.by(
    () => new Map(activeSeriesCatalog.map((entry) => [entry.key, entry]))
  );

  const focusedSensor = $derived.by(() => {
    if (sensorVariables.length === 0) return null;
    const safeIndex = Math.max(0, Math.min(chartState.focusedVariableIndex, sensorVariables.length - 1));
    return sensorVariables[safeIndex];
  });

  const contextSensor = $derived.by(() => {
    if (sensorVariables.length === 0) return null;
    const safeIndex = Math.max(0, Math.min(contextSensorIndex, sensorVariables.length - 1));
    return sensorVariables[safeIndex];
  });

  $effect(() => {
    if (!activePlant) return;

    seriesStyles = buildSeriesStyles(activePlant, untrack(() => seriesStyles), activeSeriesCatalogByKey);
  });

  const contextSeriesControls = $derived.by(() => {
    if (!activePlant || !contextSensor) return [];

    return buildContextSeriesControls({
      plant: activePlant,
      contextSensor,
      seriesStyles,
      catalogByKey: activeSeriesCatalogByKey,
    });
  });

  const contextSeriesTitle = $derived(
    contextSensor ? `Linhas - ${contextSensor.variable.name}` : 'Linhas'
  );

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

  async function importPlantFile(file: File): Promise<void> {
    const plantResult = await openPlant({ filePath: file.name, file });
    if (plantResult.success && plantResult.plant) {
      appStore.addPlant(plantResult.plant);
      appStore.setActivePlantId(plantResult.plant.id);
      setPlantData(plantResult.plant.id, plantResult.data ?? []);
      setPlantSeriesCatalog(plantResult.seriesCatalog ?? buildPlantSeriesCatalog(plantResult.plant.id, plantResult.plant.variables));
      setPlantStats(plantResult.plant.id, plantResult.stats ?? plantResult.plant.stats);
      clearVariableStats(plantResult.plant.id);
      for (const [index, stats] of (plantResult.variableStats ?? []).entries()) {
        setVariableStats(plantResult.plant.id, index, stats);
      }
      return;
    }

    throw new Error(plantResult.error || 'Erro ao abrir planta');
  }

  async function handleOpenFile() {
    openPlantLoading = true;
    try {
      const result = await openFileDialog({
        title: 'Abrir Planta',
        filters: FILE_FILTERS.plant,
      });

      if (!result) {
        openPlantLoading = false;
        return;
      }

      await importPlantFile(result.file);
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

  function handleEditPlant() {
    if (!activePlant) return;
    editPlantModal = true;
  }

  function handlePlantSaved(plant: Plant) {
    appStore.upsertPlant(plant);
    appStore.setActivePlantId(plant.id);
    setPlantSeriesCatalog(buildPlantSeriesCatalog(plant.id, plant.variables));
    setPlantStats(plant.id, plant.stats);
    createPlantModal = false;
    editPlantModal = false;
  }

  function handleRemovePlant(plantId: string) {
    removeModal = {
      visible: true,
      plantId,
      plantName: plants.find((p: Plant) => p.id === plantId)?.name || '',
      reason: 'confirm'
    };
  }

  async function confirmRemovePlant() {
    if (removeModal.reason === 'confirm') {
      plantActionLoading = 'remove';
      const result = await removePlant(removeModal.plantId);
      if (result.success) {
        appStore.removePlant(removeModal.plantId);
        clearPlant(removeModal.plantId);
        const remainingRanges = { ...manualRangesByPlant };
        delete remainingRanges[removeModal.plantId];
        manualRangesByPlant = remainingRanges;
      } else {
        alert(result.error || 'Erro ao remover planta');
      }
      plantActionLoading = null;
    }
    removeModal.visible = false;
  }

  function cancelRemovePlant() {
    removeModal.visible = false;
  }

  async function handleToggleConnect() {
    if (!activePlant) return;
    plantActionLoading = 'connect';
    const result = activePlant.connected
      ? await disconnectPlant(activePlant.id)
      : await connectPlant(activePlant.id);

    if (result.success && result.plant) {
      appStore.upsertPlant(result.plant);
    } else {
      alert(result.error || 'Erro ao atualizar driver da planta');
    }
    plantActionLoading = null;
  }

  async function handleTogglePause() {
    if (!activePlant) return;
    plantActionLoading = 'pause';
    const result = activePlant.paused
      ? await resumePlant(activePlant.id)
      : await pausePlant(activePlant.id);

    if (result.success && result.plant) {
      appStore.upsertPlant(result.plant);
    } else {
      alert(result.error || 'Erro ao atualizar pausa da planta');
    }
    plantActionLoading = null;
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
    manualRangesByPlant = {
      ...manualRangesByPlant,
      [activePlantId]: {},
    };
  }

  let _displayTick = $state(0);

  function handleContextMenu(e: MouseEvent) {
    e.preventDefault();
    if (!graphContainerRef) return;

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
    controllerLibraryModal = true;
  }

  function handleControllerTemplateSelected(plugin: PluginDefinition) {
    if (!activePlant) return;

    controllerPluginToConfig = plugin;
    controllerConfigModal = true;
  }

  function handleControllerConfigured(
    instance: PluginInstance,
    bindings?: { inputVariableIds: string[]; outputVariableIds: string[] }
  ) {
    if (!activePlant || !controllerPluginToConfig) return;

    const { id: _id, ...controller } = createConfiguredController(
      controllerPluginToConfig,
      instance.config,
      {
        name: `${controllerPluginToConfig.name} ${activePlant.controllers.length + 1}`,
        active: true,
      }
    );

    appStore.addController(activePlant.id, {
      ...controller,
      inputVariableIds: bindings?.inputVariableIds ?? [],
      outputVariableIds: bindings?.outputVariableIds ?? [],
    });
    controllerConfigModal = false;
    controllerPluginToConfig = null;
  }

  function deleteController(controllerId: string) {
    if (!activePlant) return;
    appStore.deleteController(activePlant.id, controllerId);
  }

  function updateControllerMeta(controllerId: string, field: string, value: any) {
    if (!activePlant) return;
    appStore.updateControllerMeta(activePlant.id, controllerId, field, value);
  }

  function openControllerBindingsEditor(controllerId: string) {
    if (!activePlant) return;

    const controller = activePlant.controllers.find((entry: Controller) => entry.id === controllerId);
    if (!controller) return;

    if (activePlant.connected && controller.active) {
      alert('Não é possível editar os vínculos enquanto o controlador estiver em execução.');
      return;
    }

    controllerToEditBindings = controller;
    controllerBindingsModal = true;
  }

  function toggleControllerActive(controllerId: string, nextActive: boolean) {
    if (!activePlant) return;

    const controller = activePlant.controllers.find((entry: Controller) => entry.id === controllerId);
    if (!controller) return;

    if (nextActive) {
      const conflict = getControllerActivationConflict(
        {
          ...controller,
          active: true,
        },
        activePlant.controllers,
        activePlant.variables
      );

      if (conflict) {
        alert(conflict);
        return;
      }
    }

    appStore.updateControllerMeta(activePlant.id, controllerId, 'active', nextActive);
  }

  function updateControllerBindings(
    controllerId: string,
    bindings: { inputVariableIds: string[]; outputVariableIds: string[] }
  ): string | null {
    if (!activePlant) return 'Planta ativa não encontrada';

    const controller = activePlant.controllers.find((entry: Controller) => entry.id === controllerId);
    if (!controller) return 'Controlador não encontrado';

    const nextController: Controller = {
      ...controller,
      inputVariableIds: bindings.inputVariableIds,
      outputVariableIds: bindings.outputVariableIds,
    };

    if (controller.active) {
      const conflict = getControllerActivationConflict(nextController, activePlant.controllers, activePlant.variables);
      if (conflict) {
        return conflict;
      }
    }

    appStore.updateControllerMeta(activePlant.id, controllerId, 'inputVariableIds', bindings.inputVariableIds);
    appStore.updateControllerMeta(activePlant.id, controllerId, 'outputVariableIds', bindings.outputVariableIds);
    return null;
  }

  function updateControllerParam(controllerId: string, paramKey: string, value: any) {
    if (!activePlant) return;
    appStore.updateControllerParam(activePlant.id, controllerId, paramKey, value);
  }

  async function handleSaveControllerConfig(controllerId: string) {
    if (!activePlant) {
      return { success: false, error: 'Planta ativa nao encontrada' };
    }

    const controller = activePlant.controllers.find((entry: Controller) => entry.id === controllerId);
    if (!controller) {
      return { success: false, error: 'Controlador nao encontrado' };
    }

    return saveControllerInstanceConfig({
      plantId: activePlant.id,
      controller,
      source: activePlant.source,
    });
  }

  function updateSetpoint(varIndex: number, value: number) {
    if (!activePlant) return;
    appStore.updateVariableSetpoint(activePlant.id, varIndex, value);
  }

  function handleKeyDown(event: KeyboardEvent) {
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

  $effect(() => {
    if (!active) return;
    const timer = setInterval(() => _displayTick++, 250);
    return () => clearInterval(timer);
  });

  $effect(() => {
    if (!active) return;
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  });

  const currentStats = $derived.by(() => {
    _displayTick;
    return getPlantStats(activePlantId);
  });

  const displayDt = $derived.by(() => {
    _displayTick;
    return activePlant?.connected ? currentStats.dt : 0;
  });
  const plantData = $derived(getPlantData(activePlantId));
  
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

  function handleRangeChange(variableIndex: number, xMin: number, xMax: number) {
    chartState.xMin = xMin;
    chartState.xMax = xMax;
    chartState.xMode = 'manual';
    manualRangesByPlant = {
      ...manualRangesByPlant,
      [activePlantId]: {
        ...(manualRangesByPlant[activePlantId] ?? {}),
        [variableIndex]: { xMin, xMax },
      },
    };
  }

  function hasDraggedFiles(event: DragEvent): boolean {
    const transfer = event.dataTransfer;
    if (!transfer) return false;

    if ((transfer.files?.length ?? 0) > 0) {
      return true;
    }

    for (let index = 0; index < transfer.types.length; index += 1) {
      if (transfer.types[index] === 'Files') {
        return true;
      }
    }

    for (let index = 0; index < transfer.items.length; index += 1) {
      if (transfer.items[index]?.kind === 'file') {
        return true;
      }
    }

    return false;
  }

  function handleDragEnter(event: DragEvent) {
    if (!active || !hasDraggedFiles(event)) return;
    event.preventDefault();
    if (event.dataTransfer) event.dataTransfer.dropEffect = 'copy';
    dragDepth += 1;
    dragOverlay = true;
  }

  function handleDragOver(event: DragEvent) {
    if (!active || !hasDraggedFiles(event)) return;
    event.preventDefault();
    if (event.dataTransfer) event.dataTransfer.dropEffect = 'copy';
    dragOverlay = true;
  }

  function handleDragLeave(event: DragEvent) {
    if (!active || !hasDraggedFiles(event)) return;
    event.preventDefault();
    dragDepth = Math.max(0, dragDepth - 1);
    if (dragDepth === 0) {
      dragOverlay = false;
    }
  }

  async function handleDrop(event: DragEvent) {
    if (!active || !hasDraggedFiles(event)) return;
    event.preventDefault();
    dragDepth = 0;
    dragOverlay = false;

    const file = event.dataTransfer?.files?.[0];
    if (!file) return;

    if (!file.name.toLowerCase().endsWith('.json')) {
      alert('Apenas arquivos JSON exportados podem ser soltos no Plotter.');
      return;
    }

    openPlantLoading = true;
    try {
      await importPlantFile(file);
    } catch (error) {
      console.error('Erro ao abrir arquivo arrastado:', error);
      alert(error instanceof Error ? error.message : 'Erro ao abrir planta');
    } finally {
      openPlantLoading = false;
    }
  }
</script>

<svelte:window
  ondragenter={handleDragEnter}
  ondragover={handleDragOver}
  ondragleave={handleDragLeave}
  ondrop={handleDrop}
/>

<div
  class="flex flex-col h-full w-full bg-white dark:bg-[#09090b] text-slate-900 dark:text-white relative"
  role="presentation"
>
  {#if dragOverlay}
    <div class="pointer-events-none absolute inset-4 z-40 flex items-center justify-center rounded-[28px] border-2 border-dashed border-blue-500 bg-blue-500/10 backdrop-blur-sm">
      <div class="text-center">
        <svg class="mx-auto h-10 w-10 text-blue-600 dark:text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M7 16l5-5 5 5M12 11v10M5 4h14" />
        </svg>
        <p class="mt-3 text-sm font-semibold text-blue-700 dark:text-blue-300">Solte um JSON exportado para abrir a planta no Plotter</p>
      </div>
    </div>
  {/if}

  <PlantTabs
    {plants}
    {activePlantId}
    onSelect={(id) => appStore.setActivePlantId(id)}
    onOpenFile={handleOpenFile}
    onCreateNew={handleCreateNew}
    onRemove={handleRemovePlant}
  />

  {#if plants.length === 0}
    <div class="flex-1 flex items-center justify-center bg-slate-50 p-8 dark:bg-[#09090b]">
      <div
        class="flex w-full max-w-3xl flex-col items-center justify-center gap-6 rounded-[28px] border border-slate-200 bg-white p-12 shadow-sm transition-colors hover:border-blue-300 dark:border-white/10 dark:bg-[#0c0c0e] dark:hover:border-blue-500/40"
        role="region"
        aria-label="Área para criar ou abrir uma planta"
      >
        <div class="w-20 h-20 rounded-2xl bg-slate-100 dark:bg-zinc-800 flex items-center justify-center">
          <svg class="w-10 h-10 text-slate-400 dark:text-zinc-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 17v-2m3 2v-4m3 4v-6m2 10H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
        </div>
        
        <div class="text-center space-y-2">
          <h2 class="text-2xl font-semibold text-slate-700 dark:text-zinc-200">
            Nenhuma planta ativa
          </h2>
          <p class="max-w-md text-sm text-slate-500 dark:text-zinc-400">
            Crie uma nova planta ou abra um arquivo <code class="px-1 py-0.5 bg-slate-200 dark:bg-zinc-700 rounded text-xs">.json</code> para começar a plotar dados
          </p>
        </div>

        <div class="mt-2 flex flex-wrap justify-center gap-3">
          <button
            type="button"
            onclick={handleCreateNew}
            class="flex items-center gap-2 px-5 py-2.5 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-lg transition-colors shadow-sm"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
            Nova Planta
          </button>
          <button
            type="button"
            onclick={handleOpenFile}
            disabled={openPlantLoading}
            class="flex items-center gap-2 px-5 py-2.5 bg-slate-200 dark:bg-zinc-700 hover:bg-slate-300 dark:hover:bg-zinc-600 text-slate-700 dark:text-zinc-200 font-medium rounded-lg transition-colors disabled:opacity-50"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 19a2 2 0 01-2-2V7a2 2 0 012-2h4l2 2h4a2 2 0 012 2v1M5 19h14a2 2 0 002-2v-5a2 2 0 00-2-2H9a2 2 0 00-2 2v5a2 2 0 01-2 2z" />
            </svg>
            {openPlantLoading ? 'Abrindo...' : 'Abrir Arquivo'}
          </button>
        </div>
      </div>
    </div>
  {:else}
  <div class="flex-1 flex flex-col md:flex-row overflow-hidden bg-slate-50 dark:bg-[#09090b] relative">
    <div class="flex-1 flex flex-col min-w-0 relative">
      <PlotterToolbar
        plant={activePlant}
        {currentStats}
        dt={displayDt}
        bind:showControllerPanel
        onToggleConnect={handleToggleConnect}
        onTogglePause={handleTogglePause}
        onEditPlant={handleEditPlant}
        onResetZoom={resetZoom}
        onExportCSV={handleExportCSV}
        onExportJSON={handleExportJSON}
        onPrint={handlePrint}
        {formatTime}
      />

      <div
        bind:this={graphContainerRef}
        class="plotter-graph-area flex-1 flex flex-col p-3 gap-3 overflow-hidden relative cursor-crosshair"
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
            xRangeByVariableIndex={activeManualRanges}
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
      onEditControllerBindings={openControllerBindingsEditor}
      onSaveControllerConfig={handleSaveControllerConfig}
      onToggleControllerActive={toggleControllerActive}
      onUpdateControllerMeta={updateControllerMeta}
      onUpdateControllerParam={updateControllerParam}
      onUpdateSetpoint={updateSetpoint}
    />
  </div>
  {/if}

  <PlantRemovalModal
    bind:visible={removeModal.visible}
    plantName={removeModal.plantName}
    reason={removeModal.reason}
    onConfirm={confirmRemovePlant}
    onCancel={cancelRemovePlant}
  />

  <ControllerLibraryModal
    bind:visible={controllerLibraryModal}
    onClose={() => controllerLibraryModal = false}
    onSelect={handleControllerTemplateSelected}
  />

  <PluginInstanceConfigModal
    visible={controllerConfigModal}
    plugin={controllerPluginToConfig}
    instanceLabel={controllerPluginToConfig?.name}
    showVariableBindings={true}
    sensorVariables={controllerSensorVariables}
    actuatorVariables={controllerActuatorVariables}
    submitLabel="Adicionar controlador"
    onClose={() => {
      controllerConfigModal = false;
      controllerPluginToConfig = null;
    }}
    onConfigured={handleControllerConfigured}
  />

  <ControllerBindingsModal
    visible={controllerBindingsModal}
    controllerName={controllerToEditBindings?.name ?? 'Controlador'}
    sensorVariables={controllerSensorVariables}
    actuatorVariables={controllerActuatorVariables}
    initialInputVariableIds={controllerToEditBindings?.inputVariableIds ?? []}
    initialOutputVariableIds={controllerToEditBindings?.outputVariableIds ?? []}
    onClose={() => {
      controllerBindingsModal = false;
      controllerToEditBindings = null;
    }}
    onSave={(bindings) => {
      if (!controllerToEditBindings) {
        return 'Controlador não encontrado';
      }

      const result = updateControllerBindings(controllerToEditBindings.id, bindings);
      if (!result) {
        controllerBindingsModal = false;
        controllerToEditBindings = null;
      }

      return result;
    }}
  />

  <CreatePlantModal
    bind:visible={createPlantModal}
    onPlantSaved={handlePlantSaved}
    onClose={() => createPlantModal = false}
  />

  <CreatePlantModal
    visible={editPlantModal}
    initialPlant={activePlant ?? null}
    onPlantSaved={handlePlantSaved}
    onClose={() => editPlantModal = false}
  />
</div>

<style>
  @media (max-height: 900px) {
    .plotter-graph-area {
      padding: 0.5rem;
      gap: 0.5rem;
    }
  }

  @media (max-height: 760px) {
    .plotter-graph-area {
      padding: 0.375rem;
      gap: 0.375rem;
    }
  }
</style>
