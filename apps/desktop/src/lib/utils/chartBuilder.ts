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

  setYAxis(mode: 'auto' | 'manual', min: number = 0, max: number = 100): this {
    this._config.yMode = mode;
    this._config.yMin = min;
    this._config.yMax = max;
    return this;
  }

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

  enableGrid(): this {
    this._config.showGrid = true;
    return this;
  }

  disableGrid(): this {
    this._config.showGrid = false;
    return this;
  }

  enableHover(): this {
    this._config.showHover = true;
    return this;
  }

  disableHover(): this {
    this._config.showHover = false;
    return this;
  }

  toggleSeries(key: string, visible: boolean): this {
    const series = this._series.find(s => s.key === key);
    if (series) {
      series.visible = visible;
    }
    return this;
  }

  setSeriesColor(key: string, color: string): this {
    const series = this._series.find(s => s.key === key);
    if (series) {
      series.color = color;
    }
    return this;
  }

  build(): { series: ChartSeries[]; config: ChartConfig } {
    return {
      series: this._series,
      config: this._config,
    };
  }

  get series(): ChartSeries[] {
    return this._series;
  }

  get config(): ChartConfig {
    return this._config;
  }

  static from(series: ChartSeries[], config: ChartConfig): ChartBuilder {
    const builder = new ChartBuilder();
    builder._series = [...series];
    builder._config = { ...config };
    return builder;
  }

  static createRealtimeConfig(
    data: ChartDataPoint[],
    colors: { pv: string; sp: string; mv: string },
    visible: { pv: boolean; sp: boolean; mv: boolean } = { pv: true, sp: true, mv: true }
  ): { pvsp: { series: ChartSeries[]; config: ChartConfig }; mv: { series: ChartSeries[]; config: ChartConfig } } {
    const pvspBuilder = new ChartBuilder()
      .addLineSeries('pv', data, 'pv', 'PV (Process Variable)', colors.pv, { visible: visible.pv })
      .addStepSeries('sp', data, 'sp', 'SP (Setpoint)', colors.sp, { dashed: true, visible: visible.sp })
      .setYAxis('manual', 0, 100)
      .setXAxis('auto', 30)
      .enableGrid()
      .enableHover();

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

  static createAnalyzerConfig(
    sensorData: ChartDataPoint[],
    setpointData: ChartDataPoint[],
    actuatorsData: Array<{ id: string; name: string; data: ChartDataPoint[] }>,
    sensorRange: { min: number; max: number },
    actuatorRange: { min: number; max: number }
  ): { sensor: { series: ChartSeries[]; config: ChartConfig }; actuator: { series: ChartSeries[]; config: ChartConfig } } {
    const actuatorColors = ['#10b981', '#06b6d4', '#8b5cf6', '#f97316', '#ec4899', '#14b8a6'];

    const sensorBuilder = new ChartBuilder()
      .addLineSeries('sensor', sensorData, 'sensor', 'Sensor', '#3b82f6')
      .addLineSeries('setpoint', setpointData, 'setpoint', 'Setpoint', '#f59e0b', { dashed: true, strokeWidth: 1.5 })
      .setYAxis('manual', sensorRange.min, sensorRange.max)
      .setXAxis('auto')
      .enableGrid()
      .enableHover();

    const actuatorBuilder = new ChartBuilder()
      .setYAxis('manual', actuatorRange.min, actuatorRange.max)
      .setXAxis('auto')
      .enableGrid()
      .enableHover();

    actuatorsData.forEach((act, i) => {
      const color = actuatorColors[i % actuatorColors.length];
      actuatorBuilder.addLineSeries(act.id, act.data, act.id, act.name, color, { strokeWidth: 1.5 });
    });

    if (actuatorsData.length === 0) {
      actuatorBuilder.addLineSeries('empty', [], 'empty', 'Sem atuador', '#666');
    }

    return {
      sensor: sensorBuilder.build(),
      actuator: actuatorBuilder.build(),
    };
  }
}

export function createChart(): ChartBuilder {
  return new ChartBuilder();
}

export const ChartPresets = {
  realtime: ChartBuilder.createRealtimeConfig,
  analyzer: ChartBuilder.createAnalyzerConfig,
} as const;
