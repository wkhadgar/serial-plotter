/**
 * ChartBuilder - Classe utilitária para construir configurações de gráficos facilmente
 * 
 * Esta classe fornece uma interface fluente para criar gráficos com todas as funcionalidades:
 * - Zoom (mouse wheel)
 * - Pan (Shift+drag ou botão do meio)
 * - Seleção de área para zoom
 * - Tooltip hover
 * - Auto-resize
 * - Configuração de eixos (auto/manual)
 * 
 * @example
 * ```typescript
 * const builder = new ChartBuilder()
 *   .addLineSeries('pv', data, 'pv', 'PV', '#3b82f6')
 *   .addStepSeries('sp', data, 'sp', 'SP', '#f59e0b', { dashed: true })
 *   .setYAxis('manual', 0, 100)
 *   .setXAxis('sliding', 30)
 *   .enableGrid()
 *   .build();
 * 
 * // Use em um componente Svelte:
 * <PlotlyChart series={builder.series} config={builder.config} theme={theme} />
 * ```
 */

import type { ChartSeries, ChartConfig, ChartDataPoint } from '$lib/types/chart';

export interface SeriesOptions {
  strokeWidth?: number;
  dashed?: boolean;
  visible?: boolean;
}

export class ChartBuilder {
  private _series: ChartSeries[] = [];
  private _config: ChartConfig = {
    yMin: 0,
    yMax: 100,
    yMode: 'auto',
    xMode: 'auto',
    windowSize: 30,
    showGrid: true,
    showHover: true,
  };

  /**
   * Adiciona uma série de linha ao gráfico
   * @param key Chave única da série
   * @param data Array de dados
   * @param dataKey Chave do valor no objeto de dados
   * @param label Label para exibição
   * @param color Cor da linha (hex)
   * @param options Opções adicionais (strokeWidth, dashed, visible)
   */
  addLineSeries(
    key: string,
    data: ChartDataPoint[],
    dataKey: string,
    label: string,
    color: string,
    options: SeriesOptions = {}
  ): this {
    this._series.push({
      key,
      label,
      color,
      visible: options.visible ?? true,
      data,
      dataKey,
      type: 'line',
      strokeWidth: options.strokeWidth ?? 2,
      dashed: options.dashed ?? false,
    });
    return this;
  }

  /**
   * Adiciona uma série step (degrau) ao gráfico
   * Útil para setpoints que mudam instantaneamente
   */
  addStepSeries(
    key: string,
    data: ChartDataPoint[],
    dataKey: string,
    label: string,
    color: string,
    options: SeriesOptions = {}
  ): this {
    this._series.push({
      key,
      label,
      color,
      visible: options.visible ?? true,
      data,
      dataKey,
      type: 'step',
      strokeWidth: options.strokeWidth ?? 1.5,
      dashed: options.dashed ?? false,
    });
    return this;
  }

  /**
   * Adiciona uma série de área ao gráfico
   * Útil para MV (output) de controladores
   */
  addAreaSeries(
    key: string,
    data: ChartDataPoint[],
    dataKey: string,
    label: string,
    color: string,
    options: SeriesOptions = {}
  ): this {
    this._series.push({
      key,
      label,
      color,
      visible: options.visible ?? true,
      data,
      dataKey,
      type: 'area',
      strokeWidth: options.strokeWidth ?? 1.5,
      dashed: options.dashed ?? false,
    });
    return this;
  }

  /**
   * Configura o eixo Y
   * @param mode 'auto' para escala automática, 'manual' para valores fixos
   * @param min Valor mínimo (usado em modo manual)
   * @param max Valor máximo (usado em modo manual)
   */
  setYAxis(mode: 'auto' | 'manual', min: number = 0, max: number = 100): this {
    this._config.yMode = mode;
    this._config.yMin = min;
    this._config.yMax = max;
    return this;
  }

  /**
   * Configura o eixo X
   * @param mode 'auto' (mostra todos os dados), 'sliding' (janela móvel), 'manual' (range fixo)
   * @param windowSize Tamanho da janela em segundos (para modo sliding)
   * @param min Valor mínimo (para modo manual)
   * @param max Valor máximo (para modo manual)
   */
  setXAxis(
    mode: 'auto' | 'sliding' | 'manual',
    windowSize: number = 30,
    min?: number,
    max?: number
  ): this {
    this._config.xMode = mode;
    this._config.windowSize = windowSize;
    this._config.xMin = min;
    this._config.xMax = max;
    return this;
  }

  /**
   * Habilita a grid do gráfico
   */
  enableGrid(): this {
    this._config.showGrid = true;
    return this;
  }

  /**
   * Desabilita a grid do gráfico
   */
  disableGrid(): this {
    this._config.showGrid = false;
    return this;
  }

  /**
   * Habilita o tooltip hover
   */
  enableHover(): this {
    this._config.showHover = true;
    return this;
  }

  /**
   * Desabilita o tooltip hover
   */
  disableHover(): this {
    this._config.showHover = false;
    return this;
  }

  /**
   * Alterna a visibilidade de uma série
   * @param key Chave da série
   * @param visible Novo estado de visibilidade
   */
  toggleSeries(key: string, visible: boolean): this {
    const series = this._series.find(s => s.key === key);
    if (series) {
      series.visible = visible;
    }
    return this;
  }

  /**
   * Atualiza a cor de uma série
   * @param key Chave da série
   * @param color Nova cor (hex)
   */
  setSeriesColor(key: string, color: string): this {
    const series = this._series.find(s => s.key === key);
    if (series) {
      series.color = color;
    }
    return this;
  }

  /**
   * Constrói e retorna a configuração final
   * @returns Objeto com series e config prontos para usar no PlotlyChart
   */
  build(): { series: ChartSeries[]; config: ChartConfig } {
    return {
      series: this._series,
      config: this._config,
    };
  }

  /**
   * Retorna apenas as séries
   */
  get series(): ChartSeries[] {
    return this._series;
  }

  /**
   * Retorna apenas a configuração
   */
  get config(): ChartConfig {
    return this._config;
  }

  /**
   * Cria um novo builder a partir de uma configuração existente
   * Útil para clonar configurações
   */
  static from(series: ChartSeries[], config: ChartConfig): ChartBuilder {
    const builder = new ChartBuilder();
    builder._series = [...series];
    builder._config = { ...config };
    return builder;
  }

  /**
   * Cria uma configuração padrão para gráfico de tempo real (PV/SP/MV)
   * @param data Dados da planta
   * @param colors Cores das linhas
   * @param visible Visibilidade das linhas
   */
  static createRealtimeConfig(
    data: ChartDataPoint[],
    colors: { pv: string; sp: string; mv: string },
    visible: { pv: boolean; sp: boolean; mv: boolean } = { pv: true, sp: true, mv: true }
  ): { pvsp: { series: ChartSeries[]; config: ChartConfig }; mv: { series: ChartSeries[]; config: ChartConfig } } {
    // Gráfico PV/SP
    const pvspBuilder = new ChartBuilder()
      .addLineSeries('pv', data, 'pv', 'PV (Process Variable)', colors.pv, { visible: visible.pv })
      .addStepSeries('sp', data, 'sp', 'SP (Setpoint)', colors.sp, { dashed: true, visible: visible.sp })
      .setYAxis('manual', 0, 100)
      .setXAxis('auto', 30)
      .enableGrid()
      .enableHover();

    // Gráfico MV
    const mvBuilder = new ChartBuilder()
      .addAreaSeries('mv', data, 'mv', 'MV (Output)', colors.mv, { visible: visible.mv })
      .setYAxis('manual', 0, 100)
      .setXAxis('auto', 30)
      .enableGrid()
      .enableHover();

    return {
      pvsp: pvspBuilder.build(),
      mv: mvBuilder.build(),
    };
  }

  /**
   * Cria uma configuração padrão para análise de variável (Sensor/Setpoint + Atuadores)
   * Suporta múltiplos atuadores vinculados a um sensor.
   * @param sensorData Dados do sensor
   * @param setpointData Dados do setpoint
   * @param actuatorsData Array de atuadores com seus dados
   * @param sensorRange Range do eixo Y do sensor
   * @param actuatorRange Range do eixo Y do actuator
   */
  static createAnalyzerConfig(
    sensorData: ChartDataPoint[],
    setpointData: ChartDataPoint[],
    actuatorsData: Array<{ id: string; name: string; data: ChartDataPoint[] }>,
    sensorRange: { min: number; max: number },
    actuatorRange: { min: number; max: number }
  ): { sensor: { series: ChartSeries[]; config: ChartConfig }; actuator: { series: ChartSeries[]; config: ChartConfig } } {
    const actuatorColors = ['#10b981', '#06b6d4', '#8b5cf6', '#f97316', '#ec4899', '#14b8a6'];

    // Gráfico Sensor + Setpoint
    const sensorBuilder = new ChartBuilder()
      .addLineSeries('sensor', sensorData, 'sensor', 'Sensor', '#3b82f6')
      .addLineSeries('setpoint', setpointData, 'setpoint', 'Setpoint', '#f59e0b', { dashed: true, strokeWidth: 1.5 })
      .setYAxis('manual', sensorRange.min, sensorRange.max)
      .setXAxis('auto')
      .enableGrid()
      .enableHover();

    // Gráfico Atuadores (múltiplos)
    const actuatorBuilder = new ChartBuilder()
      .setYAxis('manual', actuatorRange.min, actuatorRange.max)
      .setXAxis('auto')
      .enableGrid()
      .enableHover();

    actuatorsData.forEach((act, i) => {
      const color = actuatorColors[i % actuatorColors.length];
      actuatorBuilder.addLineSeries(act.id, act.data, act.id, act.name, color, { strokeWidth: 1.5 });
    });

    // Se não há atuadores, adiciona série vazia para não quebrar o chart
    if (actuatorsData.length === 0) {
      actuatorBuilder.addLineSeries('empty', [], 'empty', 'Sem atuador', '#666');
    }

    return {
      sensor: sensorBuilder.build(),
      actuator: actuatorBuilder.build(),
    };
  }
}

/**
 * Função auxiliar para criar configurações de gráfico rapidamente
 * @example
 * ```typescript
 * const { series, config } = createChart()
 *   .addLineSeries('data', myData, 'value', 'My Data', '#3b82f6')
 *   .setXAxis('sliding', 30)
 *   .build();
 * ```
 */
export function createChart(): ChartBuilder {
  return new ChartBuilder();
}

/**
 * Tipos de preset disponíveis
 */
export const ChartPresets = {
  /**
   * Cria gráfico de tempo real (PV/SP/MV)
   */
  realtime: ChartBuilder.createRealtimeConfig,

  /**
   * Cria gráficos de análise (Sensor/Target/Actuator)
   */
  analyzer: ChartBuilder.createAnalyzerConfig,
} as const;
