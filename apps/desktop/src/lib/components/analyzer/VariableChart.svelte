<script lang="ts">
  import type { ProcessedVariableData } from '$lib/types/analyzer';
  import PlotlyChart from '$lib/components/charts/PlotlyChart.svelte';
  import type { ChartConfig, ChartSeries, ChartStateType } from '$lib/types/chart';
  import { defaultChartState } from '$lib/types/chart';

  type SeriesStyle = { color: string; visible: boolean; label: string };
  const ACTUATOR_PALETTE = ['#10b981', '#06b6d4', '#8b5cf6', '#f97316', '#ec4899', '#14b8a6'];

  let {
    processedData,
    theme,
    onRangeChange,
    chartState = defaultChartState(),
    seriesStyles = {}
  }: {
    processedData: ProcessedVariableData;
    theme: 'dark' | 'light';
    onRangeChange?: (xMin: number, xMax: number) => void;
    chartState?: ChartStateType;
    seriesStyles?: Record<string, SeriesStyle>;
  } = $props();

  const sensorSeries = $derived.by<ChartSeries[]>(() => {
    const sensorStyle = seriesStyles.sensor;
    const setpointStyle = seriesStyles.setpoint;

    return [
      {
        key: 'sensor',
        label: sensorStyle?.label ?? `${processedData.variable.sensorName} (Sensor)`,
        color: sensorStyle?.color ?? '#3b82f6',
        visible: sensorStyle?.visible ?? true,
        data: processedData.sensorData,
        dataKey: 'value',
        type: 'line',
        strokeWidth: 2,
      },
      {
        key: 'setpoint',
        label: setpointStyle?.label ?? 'Setpoint',
        color: setpointStyle?.color ?? '#f59e0b',
        visible: setpointStyle?.visible ?? true,
        data: processedData.setpointData,
        dataKey: 'value',
        type: 'step',
        strokeWidth: 1.5,
        dashed: true,
      },
    ];
  });

  const actuatorSeries = $derived.by<ChartSeries[]>(() => {
    if (processedData.actuatorsData.length === 0) {
      return [{
        key: 'empty',
        label: 'Sem atuador',
        color: '#666',
        visible: true,
        data: [],
        dataKey: 'value',
        type: 'line',
        strokeWidth: 1.5,
      }];
    }

    const series: ChartSeries[] = [];

    for (const [index, actuator] of processedData.actuatorsData.entries()) {
      const style = seriesStyles[actuator.id];
      series.push({
        key: actuator.id,
        label: style?.label ?? actuator.name,
        color: style?.color ?? ACTUATOR_PALETTE[index % ACTUATOR_PALETTE.length],
        visible: style?.visible ?? true,
        data: actuator.data,
        dataKey: 'value',
        type: 'line',
        strokeWidth: 1.5,
      });
    }

    return series;
  });

  const baseSensorConfig = $derived<ChartConfig>({
    yMin: processedData.sensorRange.min,
    yMax: processedData.sensorRange.max,
    yMode: 'manual',
    xMode: 'auto',
    windowSize: 30,
    showGrid: true,
    showHover: true,
  });

  const baseActuatorConfig = $derived<ChartConfig>({
    yMin: processedData.actuatorRange.min,
    yMax: processedData.actuatorRange.max,
    yMode: 'manual',
    xMode: 'auto',
    windowSize: 30,
    showGrid: true,
    showHover: true,
  });

  const sensorConfig = $derived({
    ...baseSensorConfig,
    xMode: chartState.xMode,
    xMin: chartState.xMin,
    xMax: chartState.xMax,
    windowSize: chartState.windowSize,
  });

  const actuatorConfig = $derived({
    ...baseActuatorConfig,
    xMode: chartState.xMode,
    xMin: chartState.xMin,
    xMax: chartState.xMax,
    windowSize: chartState.windowSize,
  });
</script>

<div class="variable-chart h-full flex flex-col bg-white dark:bg-zinc-900 border-r border-b border-slate-200 dark:border-white/5">
  <div class="flex-1 flex flex-col min-h-0">
    <div class="variable-chart__sensor flex-[3] min-h-0 bg-slate-50 dark:bg-[#09090b] border-b border-slate-200 dark:border-white/5">
      <PlotlyChart
        series={sensorSeries}
        config={sensorConfig}
        {theme}
        onRangeChange={onRangeChange}
      />
    </div>

    <div class="variable-chart__actuator flex-[2] min-h-0 bg-slate-50 dark:bg-[#09090b]">
      <PlotlyChart
        series={actuatorSeries}
        config={actuatorConfig}
        {theme}
        onRangeChange={onRangeChange}
      />
    </div>
  </div>
</div>

<style>
  @media (max-height: 820px) {
    .variable-chart__sensor {
      flex: 1 1 56%;
    }

    .variable-chart__actuator {
      flex: 1 1 44%;
    }
  }

  @media (max-height: 700px) {
    .variable-chart__sensor {
      flex: 1 1 52%;
    }

    .variable-chart__actuator {
      flex: 1 1 48%;
    }
  }
</style>
