/**
 * ============================================================================
 * SISTEMA UNIVERSAL DE GRÁFICOS - Tipos e Interfaces
 * ============================================================================
 * 
 * Este arquivo define a interface universal para criar gráficos em qualquer
 * lugar da aplicação. Todos os gráficos têm as seguintes funcionalidades:
 * 
 * - Zoom: Mouse wheel sobre o gráfico
 * - Pan: Shift+Drag ou botão do meio do mouse
 * - Seleção: Arraste para selecionar área e dar zoom
 * - Tooltip: Hover sobre o gráfico mostra valores
 * - Auto-resize: Gráfico se ajusta ao container
 * 
 * Para criar gráficos facilmente, use a classe ChartBuilder em:
 * @see lib/utils/chartBuilder.ts
 * 
 * @example Uso básico
 * ```typescript
 * import { createChart } from '$lib/utils/chartBuilder';
 * 
 * const { series, config } = createChart()
 *   .addLineSeries('data', myData, 'value', 'My Data', '#3b82f6')
 *   .setYAxis('manual', 0, 100)
 *   .setXAxis('sliding', 30)
 *   .build();
 * ```
 */

/**
 * Estrutura de dados de um ponto do gráfico
 * 
 * @property time - Tempo em segundos (eixo X)
 * @property [key: string] - Valores das séries (pv, sp, mv, sensor, etc.)
 */
export interface ChartDataPoint {
  time: number;
  [key: string]: number;
}

/**
 * Configuração de uma série (linha) do gráfico
 * 
 * @property key - Identificador único da série
 * @property label - Nome exibido no tooltip
 * @property color - Cor da linha (hex: '#3b82f6')
 * @property visible - Se a série está visível
 * @property data - Array de pontos de dados
 * @property dataKey - Chave da propriedade nos dados ('pv', 'sp', 'sensor', etc.)
 * @property type - Tipo de visualização: 'line' (padrão), 'step' (degrau), 'area' (preenchida)
 * @property strokeWidth - Espessura da linha em pixels (padrão: 2)
 * @property dashed - Se a linha é tracejada (padrão: false)
 */
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

/**
 * Configuração global do gráfico
 * 
 * @property yMin - Valor mínimo do eixo Y
 * @property yMax - Valor máximo do eixo Y
 * @property yMode - Modo do eixo Y: 'auto' (calcula automaticamente) ou 'manual' (usa yMin/yMax)
 * @property xMode - Modo do eixo X:
 *   - 'auto': Mostra todos os dados
 *   - 'sliding': Janela móvel (últimos N segundos)
 *   - 'manual': Range fixo (usa xMin/xMax)
 * @property windowSize - Tamanho da janela móvel em segundos (usado em xMode='sliding')
 * @property xMin - Valor mínimo do eixo X (usado em xMode='manual')
 * @property xMax - Valor máximo do eixo X (usado em xMode='manual')
 * @property showGrid - Se mostra a grid de fundo
 * @property showHover - Se mostra o tooltip ao passar o mouse
 */
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

/**
 * Modo de visualização do gráfico
 * - 'grid': Visão geral com todas as variáveis em grid
 * - 'single': Visualização individual de uma variável
 */
export type ViewMode = 'grid' | 'single';

/**
 * Estado de visibilidade de uma variável individual
 */
export interface VariableVisibility {
  pv: boolean;
  sp: boolean;
  mv: boolean;
}

/**
 * Estado completo de um gráfico (usado em PlotterModule e AnalyzerModule)
 * Inclui configuração de eixos, visibilidade e modo de visualização
 */
export interface ChartStateType {
  xMode: XAxisMode;
  yMode: YAxisMode;
  xMin: number | null;
  xMax: number | null;
  yMin: number;
  yMax: number;
  windowSize: number;
  visible: VariableVisibility;
  // Multi-variável
  viewMode: ViewMode;             // 'grid' ou 'single'
  focusedVariableIndex: number;   // Índice da variável em foco (quando viewMode === 'single')
  variableCount: number;          // Total de variáveis
}

/**
 * Cria um estado padrão para gráficos
 * @param variableCount - Número de variáveis (padrão: 1)
 * @returns Estado inicial com configuração padrão
 */
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

/**
 * Avança para a próxima variável (cicla: grid -> var0 -> var1 -> ... -> grid)
 */
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

/**
 * Volta para a visão grid
 */
export function resetToGridView(state: ChartStateType): void {
  state.viewMode = 'grid';
  state.focusedVariableIndex = 0;
}

/**
 * Cores padrão para linhas de tempo real (PV/SP/MV)
 */
export interface LineColors {
  pv: string;  // Process Variable (azul)
  sp: string;  // Setpoint (laranja)
  mv: string;  // Manipulated Variable (verde)
}

/**
 * Cores padrão recomendadas para gráficos de tempo real
 */
export const DEFAULT_LINE_COLORS: Readonly<LineColors> = Object.freeze({
  pv: '#3b82f6',  // blue-500
  sp: '#f59e0b',  // amber-500
  mv: '#10b981',  // emerald-500
});
