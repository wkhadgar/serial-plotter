import type { PlantDataPoint, PlantStats } from '$lib/types/plant';

const _data = new Map<string, PlantDataPoint[]>();
const _stats = new Map<string, PlantStats>();

const DEFAULT_STATS: Readonly<PlantStats> = Object.freeze({
  errorAvg: 0,
  stability: 100,
  uptime: 0,
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

export function clearPlant(plantId: string): void {
  _data.set(plantId, []);
  _stats.set(plantId, { ...DEFAULT_STATS });
}
