export interface ChartDataPoint {
  time: number;
  [key: string]: number;
}

export interface ChartSeries {
  key: string;
  label: string;
  color: string;
  visible: boolean;
  data: ChartDataPoint[];
  dataKey: string;
  type?: 'line' | 'step' | 'area';
  strokeWidth?: number;
  dashed?: boolean;
}

export interface ChartConfig {
  yMin: number;
  yMax: number;
  yMode: 'auto' | 'manual';
  xMode: 'auto' | 'sliding' | 'manual';
  windowSize: number;
  xMin?: number | null;
  xMax?: number | null;
  showGrid: boolean;
  showHover?: boolean;
}

export type XAxisMode = 'auto' | 'sliding' | 'manual';
export type YAxisMode = 'auto' | 'manual';

export type ViewMode = 'grid' | 'single';

export interface VariableVisibility {
  pv: boolean;
  sp: boolean;
  mv: boolean;
}

export interface ChartStateType {
  xMode: XAxisMode;
  yMode: YAxisMode;
  xMin: number | null;
  xMax: number | null;
  yMin: number;
  yMax: number;
  windowSize: number;
  visible: VariableVisibility;
  viewMode: ViewMode;
  focusedVariableIndex: number;
  variableCount: number;
}

export function defaultChartState(variableCount: number = 1): ChartStateType {
  return {
    xMode: 'auto',
    yMode: 'manual',
    xMin: null,
    xMax: null,
    yMin: 0,
    yMax: 100,
    windowSize: 30,
    visible: { pv: true, sp: true, mv: true },
    viewMode: 'grid',
    focusedVariableIndex: 0,
    variableCount,
  };
}

export function nextViewState(state: ChartStateType): void {
  if (state.viewMode === 'grid') {
    state.viewMode = 'single';
    state.focusedVariableIndex = 0;
  } else {
    const nextIndex = state.focusedVariableIndex + 1;
    if (nextIndex >= state.variableCount) {
      state.viewMode = 'grid';
      state.focusedVariableIndex = 0;
    } else {
      state.focusedVariableIndex = nextIndex;
    }
  }
}

export function resetToGridView(state: ChartStateType): void {
  state.viewMode = 'grid';
  state.focusedVariableIndex = 0;
}

export interface LineColors {
  pv: string;
  sp: string;
  mv: string;
}

export const DEFAULT_LINE_COLORS: Readonly<LineColors> = Object.freeze({
  pv: '#3b82f6',
  sp: '#f59e0b',
  mv: '#10b981',
});
