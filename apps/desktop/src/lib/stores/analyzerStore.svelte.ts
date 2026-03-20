import type { ProcessedVariableData } from '$lib/types/analyzer';
import { type ChartStateType, defaultChartState, nextViewState, resetToGridView } from '$lib/types/chart';
import { generateId } from '$lib/utils/format';

export type AnalysisMethod = 'open_loop' | 'closed_loop';

export interface AnalyzerTab {
  id: string;
  name: string;
  processedVariables: ProcessedVariableData[];
  selectedVariablesIndexes: number[];
  selectedAnalysisMethod: AnalysisMethod | null;
}

class AnalyzerStore {
  tabs = $state<AnalyzerTab[]>([]);
  activeTabId = $state<string>('');
  chartStates = $state<Record<string, ChartStateType>>({});
  showVariablePanel = $state<boolean>(false);

  constructor() {
    this.createEmptyTab();
  }

  get activeTab(): AnalyzerTab | undefined {
    return this.tabs.find(t => t.id === this.activeTabId);
  }

  get chartState(): ChartStateType {
    return this.chartStates[this.activeTabId] ?? defaultChartState();
  }

  get selectedVariables(): ProcessedVariableData[] {
    const tab = this.activeTab;
    if (!tab) return [];

    if (tab.selectedVariablesIndexes.length === 0) return [];

    const selectedIndexes = new Set(tab.selectedVariablesIndexes);
    const selected: ProcessedVariableData[] = [];

    for (const variable of tab.processedVariables) {
      if (selectedIndexes.has(variable.variable.index)) {
        selected.push(variable);
      }
    }

    return selected;
  }

  get selectedAnalysisMethod(): AnalysisMethod | null {
    return this.activeTab?.selectedAnalysisMethod ?? null;
  }

  addTab(id: string, name: string, processedVariables: ProcessedVariableData[]): void {
    this.tabs = [...this.tabs, {
      id,
      name,
      processedVariables,
      selectedVariablesIndexes: [],
      selectedAnalysisMethod: null,
    }];
    this.chartStates[id] = defaultChartState();
    this.activeTabId = id;
    this.showVariablePanel = processedVariables.length > 0;
  }

  createEmptyTab(): void {
    const id = this.generateTabId();
    this.tabs = [...this.tabs, {
      id,
      name: 'Unnamed',
      processedVariables: [],
      selectedVariablesIndexes: [],
      selectedAnalysisMethod: null,
    }];
    this.chartStates[id] = defaultChartState();
    this.activeTabId = id;
    this.showVariablePanel = false;
  }

  loadFileToActiveTab(fileName: string, processedVariables: ProcessedVariableData[]): void {
    const tabIndex = this.tabs.findIndex(t => t.id === this.activeTabId);
    if (tabIndex === -1) return;

    this.tabs[tabIndex].name = fileName;
    this.tabs[tabIndex].processedVariables = processedVariables;
    this.tabs[tabIndex].selectedVariablesIndexes = [];
    this.tabs[tabIndex].selectedAnalysisMethod = null;
    
    this.chartStates[this.activeTabId] = defaultChartState();
    this.showVariablePanel = true;
  }

  get isActiveTabEmpty(): boolean {
    const tab = this.activeTab;
    return !tab || tab.processedVariables.length === 0;
  }

  removeTab(tabId: string): void {
    if (this.tabs.length === 1) {
      this.tabs[0].name = 'Unnamed';
      this.tabs[0].processedVariables = [];
      this.tabs[0].selectedVariablesIndexes = [];
      this.tabs[0].selectedAnalysisMethod = null;
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

  selectTab(tabId: string): void {
    this.activeTabId = tabId;
  }

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

  toggleVariablePanel(): void {
    this.showVariablePanel = !this.showVariablePanel;
  }

  toggleAnalysisMethod(method: AnalysisMethod): void {
    const tabIndex = this.tabs.findIndex(t => t.id === this.activeTabId);
    if (tabIndex === -1) return;

    const currentMethod = this.tabs[tabIndex].selectedAnalysisMethod;
    this.tabs[tabIndex].selectedAnalysisMethod = currentMethod === method ? null : method;
  }

  setRange(xMin: number, xMax: number): void {
    if (!this.activeTabId) return;
    const state = this.chartStates[this.activeTabId];
    if (state) {
      state.xMin = xMin;
      state.xMax = xMax;
      state.xMode = 'manual';
    }
  }

  resetZoom(): void {
    if (!this.activeTabId) return;
    const state = this.chartStates[this.activeTabId];
    if (state) {
      state.xMode = 'auto';
      state.xMin = null;
      state.xMax = null;
    }
  }

  nextView(): void {
    if (!this.activeTabId) return;
    const state = this.chartStates[this.activeTabId];
    if (state) {
      state.variableCount = this.selectedVariables.length;
      nextViewState(state);
    }
  }

  resetView(): void {
    if (!this.activeTabId) return;
    const state = this.chartStates[this.activeTabId];
    if (state) {
      resetToGridView(state);
    }
  }

  generateTabId(): string {
    return 'tab-' + generateId();
  }
}

// Singleton exportado
export const analyzerStore = new AnalyzerStore();
