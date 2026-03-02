/**
 * ============================================================================
 * ANALYZER STORE - Estado Persistente do Módulo Analyzer
 * ============================================================================
 * 
 * Store reativa que persiste o estado das abas do Analyzer.
 * Permite que o usuário troque de módulo e volte sem perder dados.
 * 
 * Arquitetura similar ao plantData.ts mas com reatividade Svelte 5.
 */

import type { ProcessedVariableData } from '$lib/types/analyzer';
import { type ChartStateType, defaultChartState, nextViewState, resetToGridView } from '$lib/types/chart';
import { generateId } from '$lib/utils/format';

/**
 * Estado de uma aba do Analyzer
 */
export interface AnalyzerTab {
  id: string;
  name: string;
  processedVariables: ProcessedVariableData[];
  selectedVariablesIndexes: number[];
}

/**
 * Estado global do Analyzer (singleton)
 */
class AnalyzerStore {
  // Estado reativo
  tabs = $state<AnalyzerTab[]>([]);
  activeTabId = $state<string>('');
  chartStates = $state<Record<string, ChartStateType>>({});
  showVariablePanel = $state<boolean>(false);

  constructor() {
    // Inicializar com uma aba vazia
    this.createEmptyTab();
  }

  /**
   * Getter para aba ativa
   */
  get activeTab(): AnalyzerTab | undefined {
    return this.tabs.find(t => t.id === this.activeTabId);
  }

  /**
   * Getter para chartState da aba ativa
   */
  get chartState(): ChartStateType {
    return this.chartStates[this.activeTabId] ?? defaultChartState();
  }

  /**
   * Getter para variáveis selecionadas
   */
  get selectedVariables(): ProcessedVariableData[] {
    const tab = this.activeTab;
    if (!tab) return [];
    return tab.processedVariables.filter(pv => 
      tab.selectedVariablesIndexes.includes(pv.variable.index)
    );
  }

  /**
   * Adicionar nova aba
   */
  addTab(id: string, name: string, processedVariables: ProcessedVariableData[]): void {
    this.tabs = [...this.tabs, {
      id,
      name,
      processedVariables,
      selectedVariablesIndexes: [],
    }];
    this.chartStates[id] = defaultChartState();
    this.activeTabId = id;
    this.showVariablePanel = processedVariables.length > 0;
  }

  /**
   * Criar aba vazia "Unnamed"
   */
  createEmptyTab(): void {
    const id = this.generateTabId();
    this.tabs = [...this.tabs, {
      id,
      name: 'Unnamed',
      processedVariables: [],
      selectedVariablesIndexes: [],
    }];
    this.chartStates[id] = defaultChartState();
    this.activeTabId = id;
    this.showVariablePanel = false;
  }

  /**
   * Carregar dados CSV na aba ativa
   */
  loadFileToActiveTab(fileName: string, processedVariables: ProcessedVariableData[]): void {
    const tabIndex = this.tabs.findIndex(t => t.id === this.activeTabId);
    if (tabIndex === -1) return;

    // Atualiza a aba ativa com os dados
    this.tabs[tabIndex].name = fileName;
    this.tabs[tabIndex].processedVariables = processedVariables;
    this.tabs[tabIndex].selectedVariablesIndexes = [];
    
    // Reset chart state e mostra painel
    this.chartStates[this.activeTabId] = defaultChartState();
    this.showVariablePanel = true;
  }

  /**
   * Verifica se a aba ativa está vazia (sem dados)
   */
  get isActiveTabEmpty(): boolean {
    const tab = this.activeTab;
    return !tab || tab.processedVariables.length === 0;
  }

  /**
   * Remover aba (sempre mantém pelo menos uma)
   */
  removeTab(tabId: string): void {
    // Se for a última aba, apenas limpar os dados ao invés de remover
    if (this.tabs.length === 1) {
      this.tabs[0].name = 'Unnamed';
      this.tabs[0].processedVariables = [];
      this.tabs[0].selectedVariablesIndexes = [];
      this.chartStates[this.tabs[0].id] = defaultChartState();
      this.showVariablePanel = false;
      return;
    }

    this.tabs = this.tabs.filter(t => t.id !== tabId);
    delete this.chartStates[tabId];
    
    if (this.activeTabId === tabId && this.tabs.length > 0) {
      this.activeTabId = this.tabs[0].id;
    }
  }

  /**
   * Selecionar aba
   */
  selectTab(tabId: string): void {
    this.activeTabId = tabId;
  }

  /**
   * Toggle variável na seleção
   */
  toggleVariable(index: number): void {
    const tabIndex = this.tabs.findIndex(t => t.id === this.activeTabId);
    if (tabIndex === -1) return;
    
    const currentIndexes = this.tabs[tabIndex].selectedVariablesIndexes;
    if (currentIndexes.includes(index)) {
      this.tabs[tabIndex].selectedVariablesIndexes = currentIndexes.filter(i => i !== index);
    } else {
      this.tabs[tabIndex].selectedVariablesIndexes = [...currentIndexes, index];
    }
  }

  /**
   * Toggle painel de variáveis
   */
  toggleVariablePanel(): void {
    this.showVariablePanel = !this.showVariablePanel;
  }

  /**
   * Atualizar range (zoom)
   */
  setRange(xMin: number, xMax: number): void {
    if (!this.activeTabId) return;
    const state = this.chartStates[this.activeTabId];
    if (state) {
      state.xMin = xMin;
      state.xMax = xMax;
      state.xMode = 'manual';
    }
  }

  /**
   * Resetar zoom
   */
  resetZoom(): void {
    if (!this.activeTabId) return;
    const state = this.chartStates[this.activeTabId];
    if (state) {
      state.xMode = 'auto';
      state.xMin = null;
      state.xMax = null;
    }
  }

  /**
   * Avança para próxima variável (cicla: grid → var0 → var1 → ... → grid)
   */
  nextView(): void {
    if (!this.activeTabId) return;
    const state = this.chartStates[this.activeTabId];
    if (state) {
      // Atualiza variableCount para número de variáveis selecionadas
      state.variableCount = this.selectedVariables.length;
      nextViewState(state);
    }
  }

  /**
   * Volta para visão grid
   */
  resetView(): void {
    if (!this.activeTabId) return;
    const state = this.chartStates[this.activeTabId];
    if (state) {
      resetToGridView(state);
    }
  }

  /**
   * Gerar ID único para aba
   */
  generateTabId(): string {
    return 'tab-' + generateId();
  }
}

// Singleton exportado
export const analyzerStore = new AnalyzerStore();
