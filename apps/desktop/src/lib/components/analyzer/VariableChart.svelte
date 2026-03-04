<script lang="ts">
  import type { ProcessedVariableData } from '$lib/types/analyzer';
  import PlotlyChart from '$lib/components/charts/PlotlyChart.svelte';
  import { ChartPresets } from '$lib/utils/chartBuilder';
  import type { ChartStateType } from '$lib/types/chart';
  import { defaultChartState } from '$lib/types/chart';

  type SeriesStyle = { color: string; visible: boolean; label: string };

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

  const sensorData = $derived(
    processedData.sensorData.map(point => ({
      time: point.time,
      sensor: point.value
    }))
  );

  const setpointData = $derived(
    processedData.setpointData.map(point => ({
      time: point.time,
      setpoint: point.value
    }))
  );

  const actuatorsData = $derived(
    processedData.actuatorsData.map(act => ({
      id: act.id,
      name: act.name,
      data: act.data.map(point => ({
        time: point.time,
        [act.id]: point.value
      }))
    }))
  );

  let chartsBase = $derived(
    ChartPresets.analyzer(
      sensorData,
      setpointData,
      actuatorsData,
      processedData.sensorRange,
      processedData.actuatorRange
    )
  );

  const charts = $derived.by(() => {
    function applySS(series: typeof chartsBase.sensor.series) {
      return series.map(s => {
        const style = seriesStyles[s.key];
        if (!style) return s;
        return { ...s, visible: style.visible, color: style.color };
      });
    }
    return {
      sensor: { ...chartsBase.sensor, series: applySS(chartsBase.sensor.series) },
      actuator: { ...chartsBase.actuator, series: applySS(chartsBase.actuator.series) },
    };
  });

  const sensorConfig = $derived({
    ...charts.sensor.config,
    xMode: chartState.xMode,
    xMin: chartState.xMin,
    xMax: chartState.xMax,
    windowSize: chartState.windowSize,
  });

  const actuatorConfig = $derived({
    ...charts.actuator.config,
    xMode: chartState.xMode,
    xMin: chartState.xMin,
    xMax: chartState.xMax,
    windowSize: chartState.windowSize,
  });
</script>

<div class="h-full flex flex-col bg-white dark:bg-zinc-900 border-r border-b border-slate-200 dark:border-white/5">
  <div class="flex-1 flex flex-col min-h-0">
    <div class="flex-[3] min-h-0 bg-slate-50 dark:bg-[#09090b] border-b border-slate-200 dark:border-white/5">
      <PlotlyChart
        series={charts.sensor.series}
        config={sensorConfig}
        {theme}
        onRangeChange={onRangeChange}
      />
    </div>

    <div class="flex-[2] min-h-0 bg-slate-50 dark:bg-[#09090b]">
      <PlotlyChart
        series={charts.actuator.series}
        config={actuatorConfig}
        {theme}
        onRangeChange={onRangeChange}
      />
    </div>
  </div>
</div>
