<script lang="ts">
  import { onMount } from 'svelte';
  import PlotlyChart from './PlotlyChart.svelte';
  import type { ChartDataPoint, ChartConfig, ChartSeries } from '$lib/types/chart';
  import type { VariableStats } from '$lib/types/plant';

  interface LinkedActuator {
    id: string;
    name: string;
    dataKey: string;
    color: string;
  }

  interface Props {
    title: string;
    unit?: string;
    pvData: ChartDataPoint[];
    mvData: ChartDataPoint[];
    pvKey: string;
    spKey: string;
    actuators?: LinkedActuator[];
    pvConfig: ChartConfig;
    mvConfig: ChartConfig;
    theme: 'dark' | 'light';
    colors?: { pv: string; sp: string };
    visible?: { pv: boolean; sp: boolean };
    lineStyles?: Record<string, { color: string; visible: boolean; label?: string }>;
    showLegend?: boolean;
    stats?: VariableStats;
    onRangeChange?: (xMin: number, xMax: number) => void;
  }

  let {
    title,
    unit = '',
    pvData,
    mvData,
    pvKey,
    spKey,
    actuators = [],
    pvConfig,
    mvConfig,
    theme,
    colors = { pv: '#3b82f6', sp: '#f59e0b' },
    visible = { pv: true, sp: true },
    lineStyles = {},
    showLegend = true,
    stats,
    onRangeChange,
  }: Props = $props();

  type CardLayoutMode = 'regular' | 'compact' | 'tight' | 'collapsed';

  let cardEl: HTMLDivElement | undefined = $state();
  let layoutMode = $state<CardLayoutMode>('regular');
  let actuatorExpanded = $state(false);

  const pvStyle = $derived(lineStyles[pvKey]);
  const spStyle = $derived(lineStyles[spKey]);
  const pvLabel = $derived(pvStyle?.label ?? `${title}`);
  const spLabel = $derived(spStyle?.label ?? 'Setpoint');

  const pvSpSeries = $derived<ChartSeries[]>([
    {
      key: 'pv',
      label: pvLabel,
      color: pvStyle?.color ?? colors.pv,
      visible: pvStyle?.visible ?? visible.pv,
      data: pvData,
      dataKey: pvKey,
      type: 'line',
      strokeWidth: 2,
    },
    {
      key: 'sp',
      label: spLabel,
      color: spStyle?.color ?? colors.sp,
      visible: spStyle?.visible ?? visible.sp,
      data: pvData,
      dataKey: spKey,
      type: 'step',
      strokeWidth: 1.5,
      dashed: true,
    },
  ]);

  const actuatorColors = ['#10b981', '#06b6d4', '#8b5cf6', '#f97316', '#ec4899', '#14b8a6'];

  const mvSeries = $derived<ChartSeries[]>(
    actuators.length > 0
      ? actuators.map((act, idx) => ({
          key: act.id,
          label: lineStyles[act.dataKey]?.label ?? act.name,
          color: lineStyles[act.dataKey]?.color || act.color || actuatorColors[idx % actuatorColors.length],
          visible: lineStyles[act.dataKey]?.visible ?? true,
          data: mvData,
          dataKey: act.dataKey,
          type: 'line' as const,
          strokeWidth: 2,
        }))
      : []
  );

  const hasActuatorChart = $derived(mvSeries.length > 0);
  const showStats = $derived(layoutMode === 'regular');
  const showLegendNow = $derived(showLegend && layoutMode !== 'tight' && layoutMode !== 'collapsed');
  const isCollapsedActuator = $derived(hasActuatorChart && layoutMode === 'collapsed');
  const showActuatorChart = $derived(hasActuatorChart && (!isCollapsedActuator || actuatorExpanded));
  const sensorXAxisMode = $derived(showActuatorChart ? 'grid-only' : 'full');
  const resolvedStats = $derived.by(() => {
    if (!stats) return null;

    const errorAvg = Number.isFinite(stats.errorAvg) ? stats.errorAvg : 0;
    const stability = Number.isFinite(stats.stability) ? stats.stability : 0;
    const ripple = Number.isFinite(stats.ripple) ? stats.ripple : 0;

    return {
      errorAvg,
      stability,
      ripple,
    };
  });

  function resolveLayoutMode(height: number, withActuator: boolean): CardLayoutMode {
    if (withActuator && height < 500) return 'collapsed';
    if (withActuator && height < 620) return 'tight';
    if (withActuator && height < 760) return 'compact';
    if (!withActuator && height < 340) return 'tight';
    if (!withActuator && height < 500) return 'compact';
    return 'regular';
  }

  function updateLayoutMode() {
    if (!cardEl) return;

    const nextMode = resolveLayoutMode(cardEl.getBoundingClientRect().height, hasActuatorChart);
    layoutMode = nextMode;

    if (nextMode !== 'collapsed') {
      actuatorExpanded = false;
    }
  }

  onMount(() => {
    updateLayoutMode();

    const observer = new ResizeObserver(() => {
      updateLayoutMode();
    });

    if (cardEl) {
      observer.observe(cardEl);
    }

    return () => {
      observer.disconnect();
    };
  });

  $effect(() => {
    hasActuatorChart;
    updateLayoutMode();
  });
</script>

<div
  bind:this={cardEl}
  class={`variable-card variable-card--${layoutMode} flex h-full min-h-0 flex-col rounded-xl border border-slate-200 bg-white shadow-sm dark:border-white/10 dark:bg-[#0c0c0e]`}
>
  <div class="variable-card__header shrink-0 border-b border-slate-200 bg-slate-50 px-3 py-2 dark:border-white/5 dark:bg-zinc-900/50">
    <div class="flex items-center justify-between gap-2">
    <div class="flex min-w-0 items-center gap-2 sm:gap-3">
      <h3 class="truncate text-sm font-bold text-slate-700 dark:text-zinc-300">
        {title}
        {#if unit}
          <span class="text-xs font-normal text-slate-400 dark:text-zinc-500">({unit})</span>
        {/if}
      </h3>
      {#if resolvedStats && showStats}
        <div class="variable-card__stats flex items-center gap-2 text-[10px] font-medium">
          <div class="flex shrink-0 items-center gap-1 px-1.5 py-0.5 rounded bg-slate-100 dark:bg-white/5">
            <span class="text-slate-400 dark:text-zinc-500">Erro:</span>
            <span class={resolvedStats.errorAvg < 3 ? 'text-emerald-600 dark:text-emerald-400' : resolvedStats.errorAvg < 10 ? 'text-amber-600 dark:text-amber-400' : 'text-red-600 dark:text-red-400'}>
              {resolvedStats.errorAvg.toFixed(2)}
            </span>
          </div>
          <div class="flex shrink-0 items-center gap-1 px-1.5 py-0.5 rounded bg-slate-100 dark:bg-white/5">
            <span class="text-slate-400 dark:text-zinc-500">Estab:</span>
            <span class={resolvedStats.stability > 90 ? 'text-emerald-600 dark:text-emerald-400' : resolvedStats.stability > 70 ? 'text-amber-600 dark:text-amber-400' : 'text-red-600 dark:text-red-400'}>
              {resolvedStats.stability.toFixed(0)}%
            </span>
          </div>
        </div>
      {/if}
    </div>
    {#if showLegendNow}
      <div class="variable-card__legend flex max-w-full items-center gap-3 overflow-x-auto text-[10px] font-medium">
        <div class="flex shrink-0 items-center gap-1">
          <div class="w-2 h-2 rounded-full" style="background-color: {pvStyle?.color ?? colors.pv}"></div>
          <span class="text-slate-500 dark:text-zinc-400">{pvLabel}</span>
        </div>
        <div class="flex shrink-0 items-center gap-1">
          <div class="w-2 h-2 rounded-full" style="background-color: {spStyle?.color ?? colors.sp}"></div>
          <span class="text-slate-500 dark:text-zinc-400">{spLabel}</span>
        </div>
        {#each actuators as act, idx}
          <div class="flex shrink-0 items-center gap-1">
            <div class="w-2 h-2 rounded-full" style="background-color: {lineStyles[act.dataKey]?.color || act.color || actuatorColors[idx % actuatorColors.length]}"></div>
            <span class="text-slate-500 dark:text-zinc-400">{lineStyles[act.dataKey]?.label ?? act.name}</span>
          </div>
        {/each}
      </div>
    {/if}
    </div>
    {#if isCollapsedActuator}
      <div class="mt-2 flex items-center justify-between gap-2 rounded-lg border border-slate-200 bg-white/80 px-2.5 py-1.5 text-[11px] text-slate-500 dark:border-white/10 dark:bg-white/[0.03] dark:text-zinc-400">
        <span>Saída de controle recolhida para caber na altura disponível.</span>
        <button
          type="button"
          onclick={() => actuatorExpanded = !actuatorExpanded}
          class="shrink-0 rounded-md bg-slate-100 px-2 py-1 font-medium text-slate-700 transition-colors hover:bg-slate-200 dark:bg-white/10 dark:text-zinc-200 dark:hover:bg-white/15"
        >
          {actuatorExpanded ? 'Fechar' : 'Abrir'}
        </button>
      </div>
    {/if}
  </div>

  <div class={`variable-card__body flex-1 min-h-0 ${isCollapsedActuator && actuatorExpanded ? 'overflow-y-auto' : 'overflow-hidden'}`}>
    <div class={`flex min-h-0 flex-col ${showActuatorChart ? 'h-full' : 'h-full'}`}>
    <div class={showActuatorChart
      ? isCollapsedActuator && actuatorExpanded
        ? 'variable-card__sensor-chart shrink-0 min-h-[190px]'
        : 'variable-card__sensor-chart flex-[3] min-h-0'
      : 'flex-1 min-h-0'}>
      <PlotlyChart
        series={pvSpSeries}
        config={pvConfig}
        {theme}
        xAxisMode={sensorXAxisMode}
        {onRangeChange}
      />
    </div>
    {#if showActuatorChart}
      <div class={isCollapsedActuator && actuatorExpanded
        ? 'variable-card__actuator-chart variable-card__actuator-chart--expanded shrink-0 min-h-[170px] border-t border-slate-100 dark:border-white/5'
        : 'variable-card__actuator-chart flex-[2] min-h-0 border-t border-slate-100 dark:border-white/5'}>
        <PlotlyChart
          series={mvSeries}
          config={mvConfig}
          {theme}
          xAxisMode="full"
          {onRangeChange}
        />
      </div>
    {/if}
    </div>
  </div>
</div>

<style>
  .variable-card {
    overflow: hidden;
  }

  .variable-card__body {
    overscroll-behavior: contain;
  }

  .variable-card--compact .variable-card__header {
    padding: 0.5rem 0.75rem;
  }

  .variable-card--compact .variable-card__header h3 {
    font-size: 0.875rem;
  }

  .variable-card--tight .variable-card__header,
  .variable-card--collapsed .variable-card__header {
    padding: 0.4rem 0.625rem;
  }

  .variable-card--tight .variable-card__legend,
  .variable-card--collapsed .variable-card__legend {
    gap: 0.5rem;
  }

  .variable-card--regular .variable-card__sensor-chart {
    flex: 3 1 0%;
  }

  .variable-card--regular .variable-card__actuator-chart {
    flex: 2 1 0%;
  }

  .variable-card--compact .variable-card__sensor-chart {
    flex: 11 1 0%;
  }

  .variable-card--compact .variable-card__actuator-chart {
    flex: 9 1 0%;
  }

  .variable-card--tight .variable-card__sensor-chart {
    flex: 1 1 50%;
  }

  .variable-card--tight .variable-card__actuator-chart {
    flex: 1 1 50%;
  }

  .variable-card__actuator-chart--expanded {
    overflow: hidden;
  }

  .variable-card--collapsed .variable-card__header h3,
  .variable-card--tight .variable-card__header h3 {
    font-size: 0.8125rem;
  }

  .variable-card--collapsed .variable-card__legend,
  .variable-card--tight .variable-card__legend {
    display: none;
  }
</style>
