import type { Controller } from './controller';
import type { PluginInstance } from './plugin';

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

export type PlantSeriesRole = 'pv' | 'sp' | 'mv';

export interface PlantSeriesDescriptor {
  key: string;
  label: string;
  role: PlantSeriesRole;
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
  linkedSensorIds?: string[];
}

export interface PlantDataPoint {
  time: number;
  [key: string]: number;
}

export interface PlantSeriesCatalog {
  plantId: string;
  series: PlantSeriesDescriptor[];
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
  dt: number;    // Delta time / amostragem
  uptime: number;  // Segundos desde que iniciou
}

export interface PlantTelemetryBufferConfig {
  maxPoints: number;
  trimTo: number;
}

export interface Plant {
  id: string;
  name: string;
  sampleTimeMs: number;
  connected: boolean;
  paused: boolean;
  variables: PlantVariable[];
  stats: PlantStats;
  controllers: Controller[];
  driver?: PluginInstance | null;
  driverId?: string | null;
  source?: 'backend' | 'workspace';
}

export function buildPlantSeriesCatalog(plantId: string, variables: PlantVariable[]): PlantSeriesCatalog {
  const series: PlantSeriesDescriptor[] = [];

  variables.forEach((variable, index) => {
    const keys = getVariableKeys(index);

    if (variable.type === 'sensor') {
      series.push({
        key: keys.pv,
        label: `${variable.name} PV`,
        role: 'pv',
      });
      series.push({
        key: keys.sp,
        label: `${variable.name} SP`,
        role: 'sp',
      });
      return;
    }

    series.push({
      key: keys.pv,
      label: variable.name,
      role: 'mv',
    });
  });

  return {
    plantId,
    series,
  };
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
    ...(type === 'atuador' ? { linkedSensorIds: [] } : {}),
  };
}

export type { Controller } from './controller';
