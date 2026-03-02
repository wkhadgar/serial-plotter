<script lang="ts">
  /**
   * ============================================================================
   * ANALYZER MODULE - Análise de Dados Históricos
   * ============================================================================
   * 
   * Módulo para análise de dados CSV com todas as funcionalidades do PlotterModule:
   * - Zoom por seleção, scroll e double-click para reset
   * - Pan com Shift+drag ou middle-click
   * - Menu de contexto (botão direito)
   * - Multi-abas com estado persistente
   * - Drag-and-drop de arquivos CSV
   * - Grid/Single view com navegação por teclado (Space/H)
   * 
   * Estado persistido via analyzerStore (singleton reativo)
   */
  import { onMount, onDestroy, untrack } from 'svelte';
  import { Upload, Sliders } from 'lucide-svelte';
  import { processJSONFile } from '$lib/services/analyzerBackend';
  import { analyzerStore } from '$lib/stores/analyzerStore.svelte';
  import AnalyzerTabs from '$lib/components/analyzer/AnalyzerTabs.svelte';
  import VariableSelectorPanel from '$lib/components/analyzer/VariableSelectorPanel.svelte';
  import VariableChart from '$lib/components/analyzer/VariableChart.svelte';
  import GenericModal from '$lib/components/modals/GenericModal.svelte';
  import ChartContextMenu from '$lib/components/plotter/ChartContextMenu.svelte';

  let { theme }: { theme: 'dark' | 'light' } = $props();

  type SeriesStyle = { color: string; visible: boolean; label: string };

  const SENSOR_COLOR = '#3b82f6';
  const SETPOINT_COLOR = '#f59e0b';
  const ACTUATOR_PALETTE = ['#10b981', '#06b6d4', '#8b5cf6', '#f97316', '#ec4899', '#14b8a6'];

  // Estado local (não persistido)
  let fileInput: HTMLInputElement;
  let isProcessing = $state(false);
  let dragOverlay = $state(false);
  let graphContainerRef: HTMLDivElement;
  let contextMenu = $state({ visible: false, x: 0, y: 0 });
  let contextVarIndex = $state(0);
  let seriesStyles = $state<Record<string, SeriesStyle>>({});

  // Modal state
  let showErrorModal = $state(false);
  let errorMessage = $state('');

  // Gera chave de série para o seriesStyles global
  function sKey(varIndex: number, seriesKey: string) {
    return `${varIndex}_${seriesKey}`;
  }

  // Inicializa/atualiza seriesStyles quando variáveis mudam
  $effect(() => {
    const tab = analyzerStore.activeTab;
    if (!tab || tab.processedVariables.length === 0) return;

    const current = untrack(() => seriesStyles);
    const next: Record<string, SeriesStyle> = {};

    tab.processedVariables.forEach((pv) => {
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

  // Variável apontada pelo clique direito
  const contextVariable = $derived.by(() => {
    const vars = analyzerStore.selectedVariables;
    if (vars.length === 0) return null;
    const safe = Math.max(0, Math.min(contextVarIndex, vars.length - 1));
    return vars[safe];
  });

  // Controles de série para o menu contextual da variável clicada
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

  /** Extrai seriesStyles relevantes para uma variável (mapa local: seriesKey → style) */
  function getVarSeriesStyles(varIndex: number, pv: import('$lib/types/analyzer').ProcessedVariableData): Record<string, SeriesStyle> {
    const result: Record<string, SeriesStyle> = {};
    result['sensor'] = seriesStyles[sKey(varIndex, 'sensor')] ?? { color: SENSOR_COLOR, visible: true, label: 'Sensor' };
    result['setpoint'] = seriesStyles[sKey(varIndex, 'setpoint')] ?? { color: SETPOINT_COLOR, visible: true, label: 'Setpoint' };
    pv.variable.actuators.forEach((act, ai) => {
      result[act.id] = seriesStyles[sKey(varIndex, act.id)] ?? { color: ACTUATOR_PALETTE[ai % ACTUATOR_PALETTE.length], visible: true, label: act.name };
    });
    return result;
  }

  // Handlers
  function handleAddTab() {
    analyzerStore.createEmptyTab();
  }

  function handleRemoveTab(tabId: string) {
    analyzerStore.removeTab(tabId);
  }

  function handleSelectTab(tabId: string) {
    analyzerStore.selectTab(tabId);
  }

  async function handleFileUpload(event: Event) {
    const target = event.target as HTMLInputElement;
    const file = target.files?.[0];
    if (!file) return;

    isProcessing = true;

    try {
      const response = await processJSONFile(file);

      if (!response.success || !response.variables) {
        errorMessage = response.error || 'Erro ao processar arquivo JSON';
        showErrorModal = true;
        return;
      }

      const fileName = response.plantName || file.name.replace(/\.[^/.]+$/, '');
      
      // Carrega os dados na aba ativa (renomeia de "Unnamed" para o nome do arquivo)
      analyzerStore.loadFileToActiveTab(fileName, response.variables);

    } catch (error) {
      errorMessage = `Erro ao carregar JSON: ${error instanceof Error ? error.message : 'Erro desconhecido'}`;
      showErrorModal = true;
    } finally {
      isProcessing = false;
      target.value = '';
    }
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

    // Detecta qual card de variável foi clicado via data-var-index
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

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    dragOverlay = true;
  }

  function handleDragLeave(e: DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    dragOverlay = false;
  }

  function handleDrop(e: DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    dragOverlay = false;

    const files = e.dataTransfer?.files;
    if (files && files.length > 0) {
      const file = files[0];
      if (file.type === 'application/json' || file.name.endsWith('.json')) {
        const inputEvent = new Event('change', { bubbles: true });
        Object.defineProperty(fileInput, 'files', {
          value: files,
          configurable: true,
        });
        fileInput.dispatchEvent(inputEvent);
      } else {
        errorMessage = 'Por favor, envie um arquivo JSON válido (exportado pelo Plotter)';
        showErrorModal = true;
      }
    }
  }

  // Keyboard navigation para modos de visualização
  function handleKeyDown(event: KeyboardEvent) {
    // Ignora se estiver em um input
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

  onMount(() => {
    window.addEventListener('keydown', handleKeyDown);
  });
  
  onDestroy(() => {
    window.removeEventListener('keydown', handleKeyDown);
  });
</script>

<div class="flex flex-col h-full w-full bg-white dark:bg-[#09090b] text-slate-900 dark:text-white">
  <!-- Tabs - sempre visível -->
  <AnalyzerTabs
    tabs={analyzerStore.tabs}
    activeTabId={analyzerStore.activeTabId}
    onSelect={handleSelectTab}
    onAdd={handleAddTab}
    onRemove={handleRemoveTab}
  />

  <!-- Main content -->
  <div class="flex-1 flex overflow-hidden bg-slate-50 dark:bg-[#09090b] relative">
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      bind:this={graphContainerRef}
      class="flex-1 flex flex-col min-w-0 relative cursor-crosshair"
      ondragover={handleDragOver}
      ondragleave={handleDragLeave}
      ondrop={handleDrop}
      oncontextmenu={handleContextMenu}
      ondblclick={resetZoom}
      role="application"
      aria-label="Área de gráficos"
    >
      <!-- Drag overlay -->
      {#if dragOverlay}
        <div class="absolute inset-0 bg-blue-500/20 border-2 border-dashed border-blue-500 flex items-center justify-center z-40 pointer-events-none">
          <div class="text-center">
            <Upload size={48} class="mx-auto mb-2 text-blue-600 dark:text-blue-400" />
            <p class="text-lg font-bold text-blue-600 dark:text-blue-400">Solte o JSON aqui</p>
          </div>
        </div>
      {/if}

      <!-- Context Menu -->
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

      <!-- Toolbar -->
      <div class="h-12 flex items-center justify-between px-4 border-b border-slate-200 dark:border-white/5 bg-white dark:bg-[#0c0c0e] shrink-0">
        <div class="flex items-center gap-2">
          {#if analyzerStore.activeTab}
            <span class="text-sm font-medium text-slate-600 dark:text-zinc-400">
              {analyzerStore.activeTab.name}
            </span>
            {#if analyzerStore.activeTab.processedVariables.length > 0}
              <span class="text-xs text-slate-400 dark:text-zinc-500">
                · {analyzerStore.activeTab.processedVariables.length} variáveis
              </span>
            {/if}
          {/if}
        </div>
        <div class="flex items-center gap-2">
          <button
            onclick={() => fileInput?.click()}
            disabled={isProcessing}
            class="flex items-center gap-2 px-3 py-1.5 bg-blue-600 hover:bg-blue-700 disabled:bg-blue-400 disabled:cursor-not-allowed text-white rounded-lg text-xs font-medium transition-colors"
          >
            <Upload size={14} />
            {isProcessing ? 'Processando...' : 'Carregar JSON'}
          </button>
          {#if analyzerStore.activeTab?.processedVariables && analyzerStore.activeTab.processedVariables.length > 0}
            <button
              onclick={() => analyzerStore.toggleVariablePanel()}
              class={`p-1.5 rounded-lg border transition-all ${analyzerStore.showVariablePanel ? 'bg-blue-600 text-white border-blue-600' : 'bg-white dark:bg-[#18181b] text-slate-500 border-slate-200 dark:border-white/10 hover:bg-slate-50 dark:hover:bg-white/5'}`}
              title={analyzerStore.showVariablePanel ? 'Ocultar Variáveis' : 'Mostrar Variáveis'}
            >
              <Sliders size={16} />
            </button>
          {/if}
        </div>
      </div>

      <input
        bind:this={fileInput}
        type="file"
        accept=".json"
        onchange={handleFileUpload}
        class="hidden"
      />

      <!-- Charts area -->
      <div class="flex-1 overflow-auto p-3">
        {#if isProcessing}
          <div class="h-full flex items-center justify-center">
            <div class="text-center">
              <div class="animate-spin rounded-full h-16 w-16 border-4 border-blue-500 border-t-transparent mx-auto mb-4"></div>
              <h2 class="text-lg font-bold text-slate-600 dark:text-zinc-400 mb-2">
                Processando arquivo...
              </h2>
              <p class="text-sm text-slate-500 dark:text-zinc-500">
                Aguarde enquanto o backend processa os dados
              </p>
            </div>
          </div>
        {:else if analyzerStore.isActiveTabEmpty}
          <!-- Aba vazia - mostrar área de drop -->
          <div class="h-full flex items-center justify-center">
            <div class="text-center">
              <Upload size={64} class="mx-auto mb-4 text-slate-300 dark:text-zinc-700" />
              <h2 class="text-lg font-bold text-slate-600 dark:text-zinc-400 mb-2">
                Arraste um arquivo JSON aqui
              </h2>
              <p class="text-sm text-slate-500 dark:text-zinc-500 mb-4">
                Ou clique no botão para selecionar
              </p>
              <button
                onclick={() => fileInput?.click()}
                class="px-6 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg text-sm font-medium transition-colors"
              >
                Carregar JSON
              </button>
            </div>
          </div>
        {:else if analyzerStore.selectedVariables.length === 0}
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
          {@const visibleVariables = analyzerStore.chartState.viewMode === 'single' 
            ? [analyzerStore.selectedVariables[analyzerStore.chartState.focusedVariableIndex]]
            : analyzerStore.selectedVariables}
          {@const isSingleView = analyzerStore.chartState.viewMode === 'single'}
          
          <div class="h-full relative {isSingleView ? 'flex items-stretch' : 'grid gap-3'}" style={isSingleView ? '' : 'grid-template-columns: repeat(auto-fit, minmax(500px, 1fr)); grid-auto-rows: 1fr;'}>
            {#each visibleVariables.filter(Boolean) as processedVar, vi (processedVar.variable.index)}
              <div class="bg-white dark:bg-[#0c0c0e] rounded-xl border border-slate-200 dark:border-white/10 overflow-hidden shadow-sm flex flex-col {isSingleView ? 'w-full h-full' : ''}" data-var-index={vi}>
                <!-- Variable header with legend -->
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
                <!-- Charts -->
                <div class="flex-1 min-h-0">
                  <VariableChart
                    processedData={processedVar}
                    {theme}
                    chartState={analyzerStore.chartState}
                    seriesStyles={getVarSeriesStyles(processedVar.variable.index, processedVar)}
                    onRangeChange={handleRangeChange}
                  />
                </div>
              </div>
            {/each}
            
            <!-- Indicator para modo single view -->
            {#if isSingleView && analyzerStore.selectedVariables.length > 1}
              <div class="absolute bottom-4 left-1/2 -translate-x-1/2 flex items-center gap-2 bg-black/60 dark:bg-white/10 backdrop-blur-sm rounded-full px-4 py-2 z-20">
                <span class="text-xs text-white/80 font-medium">
                  {analyzerStore.selectedVariables[analyzerStore.chartState.focusedVariableIndex]?.variable.sensorName ?? 'Variável'} ({analyzerStore.chartState.focusedVariableIndex + 1}/{analyzerStore.selectedVariables.length})
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

    <!-- Variable selector panel -->
    {#if analyzerStore.activeTab?.processedVariables && analyzerStore.activeTab.processedVariables.length > 0}
      <VariableSelectorPanel
        visible={analyzerStore.showVariablePanel}
        onVisibleChange={(v) => analyzerStore.showVariablePanel = v}
        variables={analyzerStore.activeTab.processedVariables.map(pv => ({
          ...pv.variable,
          selected: analyzerStore.activeTab!.selectedVariablesIndexes.includes(pv.variable.index)
        }))}
        onToggleVariable={(index) => analyzerStore.toggleVariable(index)}
      />
    {/if}
  </div>
</div>

<!-- Error Modal -->
<GenericModal
  visible={showErrorModal}
  type="error"
  title="Erro ao processar JSON"
  message={errorMessage}
  confirmLabel="Entendi"
  onConfirm={() => showErrorModal = false}
  onClose={() => showErrorModal = false}
/>
