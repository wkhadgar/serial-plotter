/**
 * Tipos compartilhados para componentes de gráfico (uPlot)
 */

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
  showLimits: boolean;
  limitHigh?: number;
  limitLow?: number;
  showHover?: boolean;
}

export type XAxisMode = 'auto' | 'sliding' | 'manual';
export type YAxisMode = 'auto' | 'manual';

export interface ChartStateType {
  xMode: XAxisMode;
  yMode: YAxisMode;
  xMin: number | null;
  xMax: number | null;
  yMin: number;
  yMax: number;
  windowSize: number;
  visible: { pv: boolean; sp: boolean; mv: boolean };
}

export function defaultChartState(): ChartStateType {
  return {
    xMode: 'auto',
    yMode: 'manual',
    xMin: null,
    xMax: null,
    yMin: 0,
    yMax: 100,
    windowSize: 30,
    visible: { pv: true, sp: true, mv: true },
  };
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
