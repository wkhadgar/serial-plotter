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
    onRangeChange?: (xMin: number, xMax: number) => void;
  }

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
    onRangeChange,
  }: Props = $props();

  const sensorVariables = $derived(
    variables
      .map((v, idx) => ({ variable: v, originalIndex: idx }))
      .filter(({ variable }) => variable.type === 'sensor')
  );

  const variablesById = $derived(
    new Map(variables.map((v, idx) => [v.id, { variable: v, index: idx }]))
  );

  function getLinkedActuators(sensorId: string) {
    const actuators: { id: string; name: string; dataKey: string; color: string }[] = [];
    const actuatorColors = ['#10b981', '#06b6d4', '#8b5cf6', '#f97316', '#ec4899', '#14b8a6'];
    
    variables.forEach((v, idx) => {
      if (v.type === 'atuador' && v.linkedSensorIds?.includes(sensorId)) {
        actuators.push({
          id: v.id,
          name: v.name,
          dataKey: getVariableKeys(idx).pv,
          color: actuatorColors[actuators.length % actuatorColors.length],
        });
      }
    });
    
    return actuators;
  }

  const gridCols = $derived.by(() => {
    const count = sensorVariables.length;
    if (count <= 1) return 'grid-cols-1';
    if (count <= 2) return 'grid-cols-1 lg:grid-cols-2';
    if (count <= 4) return 'grid-cols-1 md:grid-cols-2';
    return 'grid-cols-1 md:grid-cols-2 xl:grid-cols-3';
  });

  const visibleSensors = $derived.by(() => {
    if (viewMode === 'single' && focusedIndex >= 0 && focusedIndex < sensorVariables.length) {
      return [sensorVariables[focusedIndex]];
    }
    return sensorVariables;
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
</script>

<div 
  class="w-full h-full overflow-y-auto p-2 {viewMode === 'single' ? '' : 'grid gap-2 ' + gridCols}"
  class:flex={viewMode === 'single'}
  class:items-stretch={viewMode === 'single'}
>
  {#each visibleSensors as { variable, originalIndex }, displayIdx (variable.id)}
    <div class={viewMode === 'single' ? 'w-full h-full' : 'min-h-[300px]'} data-sensor-index={displayIdx}>
      <VariableCard
        title={variable.name}
        unit={variable.unit}
        pvData={data}
        mvData={data}
        pvKey={getVariableKeys(originalIndex).pv}
        spKey={getVariableKeys(originalIndex).sp}
        actuators={getLinkedActuators(variable.id)}
        {pvConfig}
        {mvConfig}
        {theme}
        colors={getColorSet(displayIdx)}
        {lineStyles}
        stats={variableStats[originalIndex]}
        {onRangeChange}
      />
    </div>
  {/each}
</div>

{#if viewMode === 'single' && sensorVariables.length > 1}
  <div class="absolute bottom-4 left-1/2 -translate-x-1/2 flex items-center gap-2 bg-black/60 dark:bg-white/10 backdrop-blur-sm rounded-full px-4 py-2">
    <span class="text-xs text-white/80 font-medium">
      {sensorVariables[focusedIndex]?.variable.name ?? 'Sensor'} ({focusedIndex + 1}/{sensorVariables.length})
    </span>
    <span class="text-[10px] text-white/50">
      [Space] próxima • [H] grid
    </span>
  </div>
{/if}
