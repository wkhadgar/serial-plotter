import type { PlantDataPoint, PlantStats, VariableStats } from '$lib/types/plant';

const _data = new Map<string, PlantDataPoint[]>();
const _stats = new Map<string, PlantStats>();
const _variableStats = new Map<string, VariableStats>();

const DEFAULT_STATS: Readonly<PlantStats> = Object.freeze({
  dt: 0,
  uptime: 0,
});

const DEFAULT_VAR_STATS: Readonly<VariableStats> = Object.freeze({
  errorAvg: 0,
  stability: 100,
  ripple: 0,
});

export function getPlantData(plantId: string): PlantDataPoint[] {
  let arr = _data.get(plantId);
  if (!arr) {
    arr = [];
    _data.set(plantId, arr);
  }
  return arr;
}

export function getPlantStats(plantId: string): PlantStats {
  return _stats.get(plantId) ?? { ...DEFAULT_STATS };
}

export function setPlantStats(plantId: string, stats: PlantStats): void {
  _stats.set(plantId, stats);
}

export function setPlantData(plantId: string, data: PlantDataPoint[]): void {
  _data.set(plantId, data);
}

export function getVariableStats(plantId: string, varIndex: number): VariableStats {
  const key = `${plantId}_var_${varIndex}`;
  return _variableStats.get(key) ?? { ...DEFAULT_VAR_STATS };
}

export function setVariableStats(plantId: string, varIndex: number, stats: VariableStats): void {
  const key = `${plantId}_var_${varIndex}`;
  _variableStats.set(key, stats);
}

export function clearVariableStats(plantId: string): void {
  for (const key of _variableStats.keys()) {
    if (key.startsWith(`${plantId}_var_`)) {
      _variableStats.delete(key);
    }
  }
}

export function clearPlant(plantId: string): void {
  _data.set(plantId, []);
  _stats.set(plantId, { ...DEFAULT_STATS });
  clearVariableStats(plantId);
}
