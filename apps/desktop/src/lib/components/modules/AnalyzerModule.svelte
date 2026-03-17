<script lang="ts">
  import { untrack } from 'svelte';
  import { Upload, Sliders } from 'lucide-svelte';
  import { processJSONFile } from '$lib/services/analyzerBackend';
  import { analyzerStore } from '$lib/stores/analyzerStore.svelte';
  import AnalyzerTabs from '$lib/components/analyzer/AnalyzerTabs.svelte';
  import VariableSelectorPanel from '$lib/components/analyzer/VariableSelectorPanel.svelte';
  import VariableChart from '$lib/components/analyzer/VariableChart.svelte';
  import GenericModal from '$lib/components/modals/GenericModal.svelte';
  import ChartContextMenu from '$lib/components/plotter/ChartContextMenu.svelte';

  let { theme, active = true }: { theme: 'dark' | 'light'; active?: boolean } = $props();

  type SeriesStyle = { color: string; visible: boolean; label: string };

  const SENSOR_COLOR = '#3b82f6';
  const SETPOINT_COLOR = '#f59e0b';
  const ACTUATOR_PALETTE = ['#10b981', '#06b6d4', '#8b5cf6', '#f97316', '#ec4899', '#14b8a6'];

  let fileInput: HTMLInputElement;
  let isProcessing = $state(false);
  let dragOverlay = $state(false);
  let dragDepth = $state(0);
  let graphContainerRef: HTMLDivElement;
  let contextMenu = $state({ visible: false, x: 0, y: 0 });
  let contextVarIndex = $state(0);
  let seriesStyles = $state<Record<string, SeriesStyle>>({});

  let showErrorModal = $state(false);
  let errorMessage = $state('');

  const activeTab = $derived(analyzerStore.activeTab);
  const activeProcessedVariables = $derived(activeTab?.processedVariables ?? []);
  const selectedVariables = $derived(analyzerStore.selectedVariables);
  const hasProcessedVariables = $derived(activeProcessedVariables.length > 0);
  const isSingleView = $derived(analyzerStore.chartState.viewMode === 'single');
  const visibleVariables = $derived.by(() => {
    if (isSingleView) {
      const focused = selectedVariables[analyzerStore.chartState.focusedVariableIndex];
      return focused ? [focused] : [];
    }
    return selectedVariables;
  });
  const selectorVariables = $derived.by(() => {
    if (!activeTab) return [];

    const selectedIndexes = new Set(activeTab.selectedVariablesIndexes);
    return activeTab.processedVariables.map((pv) => ({
      ...pv.variable,
      selected: selectedIndexes.has(pv.variable.index),
    }));
  });

  function sKey(varIndex: number, seriesKey: string) {
    return `${varIndex}_${seriesKey}`;
  }

  $effect(() => {
    if (!activeTab || activeProcessedVariables.length === 0) return;

    const current = untrack(() => seriesStyles);
    const next: Record<string, SeriesStyle> = {};

    activeProcessedVariables.forEach((pv) => {
      const vi = pv.variable.index;
      next[sKey(vi, 'sensor')] = current[sKey(vi, 'sensor')] ?? {
        color: SENSOR_COLOR,
        visible: true,
        label: `${pv.variable.sensorName} (Sensor)`,
      };
      next[sKey(vi, 'setpoint')] = current[sKey(vi, 'setpoint')] ?? {
        color: SETPOINT_COLOR,
        visible: true,
        label: 'Setpoint',
      };
      pv.variable.actuators.forEach((act, ai) => {
        next[sKey(vi, act.id)] = current[sKey(vi, act.id)] ?? {
          color: ACTUATOR_PALETTE[ai % ACTUATOR_PALETTE.length],
          visible: true,
          label: act.name,
        };
      });
    });

    seriesStyles = next;
  });

  const contextVariable = $derived.by(() => {
    if (selectedVariables.length === 0) return null;
    const safe = Math.max(0, Math.min(contextVarIndex, selectedVariables.length - 1));
    return selectedVariables[safe];
  });

  const contextSeriesControls = $derived.by(() => {
    if (!contextVariable) return [];
    const vi = contextVariable.variable.index;
    const controls: { key: string; label: string; color: string; visible: boolean }[] = [];

    const sensorStyle = seriesStyles[sKey(vi, 'sensor')];
    controls.push({
      key: sKey(vi, 'sensor'),
      label: `${contextVariable.variable.sensorName} (Sensor)`,
      color: sensorStyle?.color ?? SENSOR_COLOR,
      visible: sensorStyle?.visible ?? true,
    });

    const spStyle = seriesStyles[sKey(vi, 'setpoint')];
    controls.push({
      key: sKey(vi, 'setpoint'),
      label: 'Setpoint',
      color: spStyle?.color ?? SETPOINT_COLOR,
      visible: spStyle?.visible ?? true,
    });

    contextVariable.variable.actuators.forEach((act, ai) => {
      const actStyle = seriesStyles[sKey(vi, act.id)];
      controls.push({
        key: sKey(vi, act.id),
        label: `${act.name} (Atuador)`,
        color: actStyle?.color ?? ACTUATOR_PALETTE[ai % ACTUATOR_PALETTE.length],
        visible: actStyle?.visible ?? true,
      });
    });

    return controls;
  });

  const contextSeriesTitle = $derived(
    contextVariable ? `Linhas - ${contextVariable.variable.sensorName}` : 'Linhas'
  );

  const variableSeriesStyles = $derived.by(() => {
    const byVariable = new Map<number, Record<string, SeriesStyle>>();

    for (const processedVar of selectedVariables) {
      const variableIndex = processedVar.variable.index;
      const styles: Record<string, SeriesStyle> = {
        sensor: seriesStyles[sKey(variableIndex, 'sensor')] ?? {
          color: SENSOR_COLOR,
          visible: true,
          label: 'Sensor',
        },
        setpoint: seriesStyles[sKey(variableIndex, 'setpoint')] ?? {
          color: SETPOINT_COLOR,
          visible: true,
          label: 'Setpoint',
        },
      };

      for (const [actuatorIndex, actuator] of processedVar.variable.actuators.entries()) {
        styles[actuator.id] = seriesStyles[sKey(variableIndex, actuator.id)] ?? {
          color: ACTUATOR_PALETTE[actuatorIndex % ACTUATOR_PALETTE.length],
          visible: true,
          label: actuator.name,
        };
      }

      byVariable.set(variableIndex, styles);
    }

    return byVariable;
  });

  function toggleSeriesVisibility(key: string) {
    const cur = seriesStyles[key];
    if (!cur) return;
    seriesStyles = { ...seriesStyles, [key]: { ...cur, visible: !cur.visible } };
  }

  function updateSeriesColor(key: string, color: string) {
    const cur = seriesStyles[key];
    if (!cur) return;
    seriesStyles = { ...seriesStyles, [key]: { ...cur, color } };
  }

  function handleAddTab() {
    analyzerStore.createEmptyTab();
  }

  function handleRemoveTab(tabId: string) {
    analyzerStore.removeTab(tabId);
  }

  function handleSelectTab(tabId: string) {
    analyzerStore.selectTab(tabId);
  }

  async function processAnalyzerFile(file: File) {
    isProcessing = true;

    try {
      const response = await processJSONFile(file);

      if (!response.success || !response.variables) {
        errorMessage = response.error || 'Erro ao processar arquivo JSON';
        showErrorModal = true;
        return;
      }

      const fileName = response.plantName || file.name.replace(/\.[^/.]+$/, '');
      
      analyzerStore.loadFileToActiveTab(fileName, response.variables);

    } catch (error) {
      errorMessage = `Erro ao carregar JSON: ${error instanceof Error ? error.message : 'Erro desconhecido'}`;
      showErrorModal = true;
    } finally {
      isProcessing = false;
    }
  }

  async function handleFileUpload(event: Event) {
    const target = event.target as HTMLInputElement;
    const file = target.files?.[0];
    if (!file) return;
    await processAnalyzerFile(file);
    target.value = '';
  }

  function handleRangeChange(xMin: number, xMax: number) {
    analyzerStore.setRange(xMin, xMax);
  }

  function resetZoom() {
    analyzerStore.resetZoom();
  }

  function handleContextMenu(e: MouseEvent) {
    e.preventDefault();
    if (!graphContainerRef) return;

    let target = e.target as HTMLElement | null;
    while (target && target !== graphContainerRef) {
      const idx = target.dataset?.varIndex;
      if (idx !== undefined) {
        contextVarIndex = parseInt(idx, 10);
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

  function handleDragEnter(e: DragEvent) {
    if (!active || !hasDraggedFiles(e)) return;
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = 'copy';
    dragDepth += 1;
    dragOverlay = true;
  }

  function handleDragOver(e: DragEvent) {
    if (!active || !hasDraggedFiles(e)) return;
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = 'copy';
    dragOverlay = true;
  }

  function handleDragLeave(e: DragEvent) {
    if (!active || !hasDraggedFiles(e)) return;
    e.preventDefault();
    dragDepth = Math.max(0, dragDepth - 1);
    if (dragDepth === 0) {
      dragOverlay = false;
    }
  }

  async function handleDrop(e: DragEvent) {
    if (!active || !hasDraggedFiles(e)) return;
    e.preventDefault();
    dragDepth = 0;
    dragOverlay = false;

    const file = e.dataTransfer?.files?.[0];
    if (!file) return;

    if (file.type === 'application/json' || file.name.endsWith('.json')) {
      await processAnalyzerFile(file);
    } else {
      errorMessage = 'Por favor, envie um arquivo JSON válido (exportado pelo Plotter)';
      showErrorModal = true;
    }
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement) {
      return;
    }
    
    if (event.code === 'Space') {
      event.preventDefault();
      analyzerStore.nextView();
    } else if (event.code === 'KeyH') {
      event.preventDefault();
      analyzerStore.resetView();
    }
  }

  $effect(() => {
    if (!active) return;
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  });
</script>

<svelte:window
  ondragenter={handleDragEnter}
  ondragover={handleDragOver}
  ondragleave={handleDragLeave}
  ondrop={handleDrop}
/>

<div
  class="flex flex-col h-full w-full bg-white dark:bg-[#09090b] text-slate-900 dark:text-white"
  role="presentation"
>
  <AnalyzerTabs
    tabs={analyzerStore.tabs}
    activeTabId={analyzerStore.activeTabId}
    onSelect={handleSelectTab}
    onAdd={handleAddTab}
    onRemove={handleRemoveTab}
  />

  <div class="flex-1 flex overflow-hidden bg-slate-50 dark:bg-[#09090b] relative">
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      bind:this={graphContainerRef}
      class="flex-1 flex flex-col min-w-0 relative cursor-crosshair"
      oncontextmenu={handleContextMenu}
      ondblclick={resetZoom}
      role="application"
      aria-label="Área de gráficos"
    >
      {#if dragOverlay}
        <div class="absolute inset-0 bg-blue-500/20 border-2 border-dashed border-blue-500 flex items-center justify-center z-40 pointer-events-none">
          <div class="text-center">
            <Upload size={48} class="mx-auto mb-2 text-blue-600 dark:text-blue-400" />
            <p class="text-lg font-bold text-blue-600 dark:text-blue-400">Solte o JSON aqui</p>
          </div>
        </div>
      {/if}

      <ChartContextMenu
        bind:visible={contextMenu.visible}
        x={contextMenu.x}
        y={contextMenu.y}
        chartState={analyzerStore.chartState}
        seriesControls={contextSeriesControls}
        seriesTitle={contextSeriesTitle}
        onToggleSeries={toggleSeriesVisibility}
        onChangeSeriesColor={updateSeriesColor}
        onClose={closeContextMenu}
      />

      <div class="h-14 shrink-0 border-b border-slate-200 bg-white/90 px-5 backdrop-blur dark:border-white/5 dark:bg-[#0c0c0e]/90">
        <div class="flex h-full items-center justify-between gap-3">
          <div class="flex min-w-0 items-center gap-2">
          {#if activeTab}
            <span class="truncate text-sm font-medium text-slate-700 dark:text-zinc-300">
              {activeTab.name}
            </span>
            {#if hasProcessedVariables}
              <span class="shrink-0 rounded-full bg-slate-100 px-2 py-1 text-[11px] font-medium text-slate-500 dark:bg-zinc-800 dark:text-zinc-400">
                {activeProcessedVariables.length} variáveis
              </span>
            {/if}
          {/if}
          </div>
          <div class="flex items-center gap-2">
            <button
              onclick={() => fileInput?.click()}
              disabled={isProcessing}
              class="flex items-center gap-2 rounded-xl bg-blue-600 px-3.5 py-2 text-xs font-semibold text-white transition-colors hover:bg-blue-700 disabled:cursor-not-allowed disabled:bg-blue-400"
            >
              <Upload size={14} />
              {isProcessing ? 'Processando...' : 'Carregar JSON'}
            </button>
            {#if hasProcessedVariables}
              <button
                onclick={() => analyzerStore.toggleVariablePanel()}
                class={`p-2 rounded-xl border transition-all ${analyzerStore.showVariablePanel ? 'bg-blue-600 text-white border-blue-600' : 'bg-white dark:bg-[#18181b] text-slate-500 border-slate-200 dark:border-white/10 hover:bg-slate-50 dark:hover:bg-white/5'}`}
                title={analyzerStore.showVariablePanel ? 'Ocultar Variáveis' : 'Mostrar Variáveis'}
              >
                <Sliders size={16} />
              </button>
            {/if}
          </div>
        </div>
      </div>

      <input
        bind:this={fileInput}
        type="file"
        accept=".json"
        onchange={handleFileUpload}
        class="hidden"
      />

      <div class="flex-1 overflow-auto p-3">
        {#if isProcessing}
          <div class="h-full flex items-center justify-center">
            <div class="text-center">
              <div class="animate-spin rounded-full h-16 w-16 border-4 border-blue-500 border-t-transparent mx-auto mb-4"></div>
              <h2 class="text-lg font-bold text-slate-600 dark:text-zinc-400 mb-2">
                Processando arquivo...
              </h2>
              <p class="text-sm text-slate-500 dark:text-zinc-500">
                Aguarde enquanto processamos os dados
              </p>
            </div>
          </div>
        {:else if analyzerStore.isActiveTabEmpty}
          <div class="h-full flex items-center justify-center p-8">
            <div class="flex flex-col items-center justify-center gap-6 p-12 border-2 border-dashed border-slate-300 dark:border-zinc-700 rounded-2xl bg-slate-50/50 dark:bg-zinc-900/50 max-w-lg w-full transition-colors hover:border-blue-400 dark:hover:border-blue-500 hover:bg-blue-50/30 dark:hover:bg-blue-900/10">
              <div class="w-20 h-20 rounded-2xl bg-slate-100 dark:bg-zinc-800 flex items-center justify-center">
                <Upload size={40} class="text-slate-400 dark:text-zinc-500" />
              </div>
              <div class="text-center space-y-2">
                <h2 class="text-xl font-semibold text-slate-700 dark:text-zinc-200">
                  Arraste um arquivo JSON aqui
                </h2>
                <p class="text-sm text-slate-500 dark:text-zinc-400 max-w-xs">
                  Ou clique no botão para selecionar um arquivo de dados do experimento
                </p>
              </div>
              <button
                onclick={() => fileInput?.click()}
                class="flex items-center gap-2 px-5 py-2.5 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-lg transition-colors shadow-sm"
              >
                <Upload size={16} />
                Carregar JSON
              </button>
            </div>
          </div>
        {:else if selectedVariables.length === 0}
          <div class="h-full flex items-center justify-center">
            <div class="text-center">
              <div class="text-6xl mb-4">📊</div>
              <h2 class="text-lg font-bold text-slate-600 dark:text-zinc-400 mb-2">
                Nenhuma variável selecionada
              </h2>
              <p class="text-sm text-slate-500 dark:text-zinc-500">
                Selecione variáveis no painel lateral para visualizar
              </p>
            </div>
          </div>
        {:else}
          <div class="h-full relative {isSingleView ? 'flex items-stretch' : 'grid gap-3'}" style={isSingleView ? '' : 'grid-template-columns: repeat(auto-fit, minmax(500px, 1fr)); grid-auto-rows: 1fr;'}>
            {#each visibleVariables as processedVar, vi (processedVar.variable.index)}
              {@const currentSeriesStyles = variableSeriesStyles.get(processedVar.variable.index) ?? {}}
              <div class="bg-white dark:bg-[#0c0c0e] rounded-xl border border-slate-200 dark:border-white/10 overflow-hidden shadow-sm flex flex-col {isSingleView ? 'w-full h-full' : ''}" data-var-index={isSingleView ? analyzerStore.chartState.focusedVariableIndex : vi}>
                <div class="px-3 py-2 border-b border-slate-200 dark:border-white/5 bg-slate-50 dark:bg-zinc-900/50 flex items-center justify-between shrink-0">
                  <h3 class="text-sm font-bold text-slate-700 dark:text-zinc-300">
                    {processedVar.variable.sensorName}
                    <span class="text-[10px] font-normal text-slate-400 dark:text-zinc-500 ml-1">({processedVar.variable.sensorUnit})</span>
                  </h3>
                  <div class="flex items-center gap-3 text-[10px] font-medium">
                    <div class="flex items-center gap-1">
                      <div class="w-2 h-2 rounded-full bg-blue-500"></div>
                      <span class="text-slate-500 dark:text-zinc-400">Sensor</span>
                    </div>
                    <div class="flex items-center gap-1">
                      <div class="w-2 h-2 rounded-full bg-amber-500"></div>
                      <span class="text-slate-500 dark:text-zinc-400">Setpoint</span>
                    </div>
                    {#each processedVar.variable.actuators as act}
                      <div class="flex items-center gap-1">
                        <div class="w-2 h-2 rounded-full bg-emerald-500"></div>
                        <span class="text-slate-500 dark:text-zinc-400">{act.name}</span>
                      </div>
                    {/each}
                  </div>
                </div>
                <div class="flex-1 min-h-0">
                  <VariableChart
                    processedData={processedVar}
                    {theme}
                    chartState={analyzerStore.chartState}
                    seriesStyles={currentSeriesStyles}
                    onRangeChange={handleRangeChange}
                  />
                </div>
              </div>
            {/each}
            
            {#if isSingleView && selectedVariables.length > 1}
              <div class="absolute bottom-4 left-1/2 -translate-x-1/2 flex items-center gap-2 bg-black/60 dark:bg-white/10 backdrop-blur-sm rounded-full px-4 py-2 z-20">
                <span class="text-xs text-white/80 font-medium">
                  {selectedVariables[analyzerStore.chartState.focusedVariableIndex]?.variable.sensorName ?? 'Variável'} ({analyzerStore.chartState.focusedVariableIndex + 1}/{selectedVariables.length})
                </span>
                <span class="text-[10px] text-white/50">
                  [Space] próxima • [H] grid
                </span>
              </div>
            {/if}
          </div>
        {/if}
      </div>
    </div>

    {#if hasProcessedVariables}
      <VariableSelectorPanel
        visible={analyzerStore.showVariablePanel}
        onVisibleChange={(v) => analyzerStore.showVariablePanel = v}
        variables={selectorVariables}
        onToggleVariable={(index) => analyzerStore.toggleVariable(index)}
      />
    {/if}
  </div>
</div>

<GenericModal
  visible={showErrorModal}
  type="error"
  title="Erro ao processar JSON"
  message={errorMessage}
  confirmLabel="Entendi"
  onConfirm={() => showErrorModal = false}
  onClose={() => showErrorModal = false}
/>
