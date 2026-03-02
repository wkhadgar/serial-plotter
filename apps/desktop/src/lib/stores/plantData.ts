import type { PlantDataPoint, PlantStats, VariableStats } from '$lib/types/plant';

const _data = new Map<string, PlantDataPoint[]>();
const _stats = new Map<string, PlantStats>();
const _variableStats = new Map<string, VariableStats>();

const DEFAULT_STATS: Readonly<PlantStats> = Object.freeze({
  errorAvg: 0,
  stability: 100,
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

/**
 * Retorna estatísticas de uma variável específica
 */
export function getVariableStats(plantId: string, varIndex: number): VariableStats {
  const key = `${plantId}_var_${varIndex}`;
  return _variableStats.get(key) ?? { ...DEFAULT_VAR_STATS };
}

/**
 * Define estatísticas de uma variável específica
 */
export function setVariableStats(plantId: string, varIndex: number, stats: VariableStats): void {
  const key = `${plantId}_var_${varIndex}`;
  _variableStats.set(key, stats);
}

export function clearPlant(plantId: string): void {
  _data.set(plantId, []);
  _stats.set(plantId, { ...DEFAULT_STATS });
  // Limpa stats de variáveis (até 10 por segurança)
  for (let i = 0; i < 10; i++) {
    _variableStats.delete(`${plantId}_var_${i}`);
  }
}
