import type { Controller } from './controller';

export interface VariableDataPoint {
  time: number;
  pv: number;
  sp: number;
  mv: number;
}

export interface VariableStats {
  errorAvg: number;
  stability: number;
  ripple: number;
}

export type VariableType = 'sensor' | 'atuador';

export const VARIABLE_TYPE_LABELS: Record<VariableType, string> = {
  sensor: 'Sensor',
  atuador: 'Atuador',
};

export interface PlantVariable {
  id: string;
  name: string;
  type: VariableType;
  unit: string;
  setpoint: number;
  pvMin: number;
  pvMax: number;
  mvMin: number;
  mvMax: number;
  linkedSensorIds?: string[];
}

export interface PlantDataPoint {
  time: number;
  [key: string]: number;
}

export function getVariableKeys(varIndex: number) {
  return {
    pv: `var_${varIndex}_pv`,
    sp: `var_${varIndex}_sp`,
    mv: `var_${varIndex}_mv`,
  };
}

export function extractVariableData(
  data: PlantDataPoint[],
  varIndex: number
): VariableDataPoint[] {
  const keys = getVariableKeys(varIndex);
  return data.map(point => ({
    time: point.time,
    pv: point[keys.pv] ?? 0,
    sp: point[keys.sp] ?? 0,
    mv: point[keys.mv] ?? 0,
  }));
}

export interface PlantStats {
  errorAvg: number;
  stability: number;
  uptime: number;
}

export interface Plant {
  id: string;
  name: string;
  connected: boolean;
  paused: boolean;
  variables: PlantVariable[];
  stats: PlantStats;
  controllers: Controller[];
}

export function createDefaultVariable(index: number, name?: string, type: VariableType = 'sensor'): PlantVariable {
  return {
    id: `var_${index}`,
    name: name ?? `Variável ${index + 1}`,
    type,
    unit: '%',
    setpoint: type === 'sensor' ? 50 : 0,
    pvMin: 0,
    pvMax: 100,
    mvMin: 0,
    mvMax: 100,
    ...(type === 'atuador' ? { linkedSensorIds: [] } : {}),
  };
}

export type { Controller } from './controller';

