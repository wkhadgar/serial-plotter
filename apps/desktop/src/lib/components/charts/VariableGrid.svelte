<script lang="ts">
  import VariableCard from './VariableCard.svelte';
  import type { PlantVariable, PlantDataPoint, VariableStats } from '$lib/types/plant';
  import type { ChartConfig, ChartStateType, ViewMode } from '$lib/types/chart';
  import { getVariableKeys } from '$lib/types/plant';

  interface Props {
    variables: PlantVariable[];
    data: PlantDataPoint[];
    pvConfig: ChartConfig;
    mvConfig: ChartConfig;
    theme: 'dark' | 'light';
    viewMode: ViewMode;
    focusedIndex: number;
    lineStyles?: Record<string, { color: string; visible: boolean; label?: string }>;
    variableStats?: VariableStats[];
    xRangeByVariableIndex?: Record<number, { xMin: number; xMax: number }>;
    onRangeChange?: (variableIndex: number, xMin: number, xMax: number) => void;
  }

  interface LinkedActuatorEntry {
    id: string;
    name: string;
    dataKey: string;
    color: string;
    unit?: string;
  }

  interface SensorEntry {
    variable: PlantVariable;
    originalIndex: number;
    pvKey: string;
    spKey: string;
    actuators: LinkedActuatorEntry[];
  }

  const ACTUATOR_COLORS = ['#10b981', '#06b6d4', '#8b5cf6', '#f97316', '#ec4899', '#14b8a6'];

  let {
    variables,
    data,
    pvConfig,
    mvConfig,
    theme,
    viewMode,
    focusedIndex,
    lineStyles = {},
    variableStats = [],
    xRangeByVariableIndex = {},
    onRangeChange,
  }: Props = $props();

  const sensorEntries = $derived.by<SensorEntry[]>(() => {
    const entries: SensorEntry[] = [];
    const bySensorId = new Map<string, LinkedActuatorEntry[]>();

    for (const [index, variable] of variables.entries()) {
      if (variable.type !== 'sensor') continue;

      const keys = getVariableKeys(index);
      const actuators: LinkedActuatorEntry[] = [];
      entries.push({
        variable,
        originalIndex: index,
        pvKey: keys.pv,
        spKey: keys.sp,
        actuators,
      });
      bySensorId.set(variable.id, actuators);
    }

    for (const [index, variable] of variables.entries()) {
      if (variable.type !== 'atuador' || !variable.linkedSensorIds?.length) continue;

      for (const sensorId of variable.linkedSensorIds) {
        const actuatorList = bySensorId.get(sensorId);
        if (!actuatorList) continue;
        actuatorList.push({
          id: variable.id,
          name: variable.name,
          dataKey: getVariableKeys(index).pv,
          color: ACTUATOR_COLORS[actuatorList.length % ACTUATOR_COLORS.length],
          unit: variable.unit,
        });
      }
    }

    return entries;
  });

  const gridCols = $derived.by(() => {
    const count = sensorEntries.length;
    if (count <= 1) return 'grid-cols-1';
    if (count <= 2) return 'grid-cols-1 lg:grid-cols-2';
    if (count <= 4) return 'grid-cols-1 md:grid-cols-2';
    return 'grid-cols-1 md:grid-cols-2 xl:grid-cols-3';
  });

  const visibleSensors = $derived.by(() => {
    if (viewMode === 'single' && focusedIndex >= 0 && focusedIndex < sensorEntries.length) {
      return [sensorEntries[focusedIndex]];
    }
    return sensorEntries;
  });

  const variableColors = [
    { pv: '#3b82f6', sp: '#f59e0b' },
    { pv: '#8b5cf6', sp: '#ec4899' },
    { pv: '#f97316', sp: '#84cc16' },
    { pv: '#14b8a6', sp: '#f43f5e' },
  ];

  function getColorSet(index: number) {
    return variableColors[index % variableColors.length];
  }

  function getVariableChartConfig(baseConfig: ChartConfig, variableIndex: number): ChartConfig {
    const range = xRangeByVariableIndex[variableIndex];
    if (range) {
      return {
        ...baseConfig,
        xMode: 'manual',
        xMin: range.xMin,
        xMax: range.xMax,
      };
    }

    if (baseConfig.xMode !== 'manual') return baseConfig;

    return {
      ...baseConfig,
      xMin: baseConfig.xMin ?? null,
      xMax: baseConfig.xMax ?? null,
    };
  }
</script>

<div
  class="variable-grid-container box-border w-full min-h-0 flex-1 overflow-y-auto p-2 {viewMode === 'single' ? 'flex flex-col' : 'grid gap-2 ' + gridCols}"
  class:variable-grid-container--single={viewMode === 'single'}
  class:variable-grid-container--grid={viewMode !== 'single'}
>
  {#each visibleSensors as sensorEntry, displayIdx (sensorEntry.variable.id)}
    {@const cardPvConfig = getVariableChartConfig(pvConfig, sensorEntry.originalIndex)}
    {@const cardMvConfig = getVariableChartConfig(mvConfig, sensorEntry.originalIndex)}
    <div class={viewMode === 'single' ? 'single-sensor-shell w-full' : 'sensor-card-shell'} data-sensor-index={displayIdx}>
      <VariableCard
        title={sensorEntry.variable.name}
        unit={sensorEntry.variable.unit}
        pvData={data}
        mvData={data}
        pvKey={sensorEntry.pvKey}
        spKey={sensorEntry.spKey}
        actuators={sensorEntry.actuators}
        pvConfig={cardPvConfig}
        mvConfig={cardMvConfig}
        {theme}
        colors={getColorSet(displayIdx)}
        {lineStyles}
        stats={variableStats[sensorEntry.originalIndex]}
        onRangeChange={onRangeChange
          ? (xMin: number, xMax: number) => onRangeChange(sensorEntry.originalIndex, xMin, xMax)
          : undefined}
      />
    </div>
  {/each}
</div>

{#if viewMode === 'single' && sensorEntries.length > 1}
  <div class="sensor-view-hint absolute bottom-4 left-1/2 -translate-x-1/2 flex items-center gap-2 bg-black/60 dark:bg-white/10 backdrop-blur-sm rounded-full px-4 py-2">
    <span class="text-xs text-white/80 font-medium">
      {sensorEntries[focusedIndex]?.variable.name ?? 'Sensor'} ({focusedIndex + 1}/{sensorEntries.length})
    </span>
    <span class="text-[10px] text-white/50">
      [Space] próxima • [H] grid
    </span>
  </div>
{/if}

<style>
  .variable-grid-container {
    overflow-x: hidden;
  }

  .variable-grid-container--single {
    display: flex;
    flex-direction: column;
    min-height: 0;
    align-content: stretch;
  }

  .sensor-card-shell {
    min-height: clamp(130px, 23vh, 240px);
  }

  .single-sensor-shell {
    flex: 1 1 clamp(160px, 52vh, 560px);
    height: 100%;
    min-height: 0;
  }

  @media (max-height: 900px) {
    .variable-grid-container {
      padding: 0.375rem;
      gap: 0.375rem;
    }
  }

  @media (max-height: 760px) {
    .variable-grid-container {
      padding: 0.25rem;
      gap: 0.25rem;
    }
    .sensor-view-hint {
      bottom: 0.5rem;
      padding: 0.4rem 0.75rem;
    }
  }
</style>
