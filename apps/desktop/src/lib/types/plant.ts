import type { Controller } from './controller';

/**
 * Ponto de dado do histórico da planta
 */
export interface PlantDataPoint {
  time: number;
  sp: number; // Setpoint
  pv: number; // Process Variable
  mv: number; // Manipulated Variable
  [key: string]: number; // index signature for generic chart access
}

/**
 * Limites de alarme
 */
export interface AlarmLimits {
  high: number;
  low: number;
}

/**
 * Estatísticas (KPIs) da planta
 */
export interface PlantStats {
  errorAvg: number;
  stability: number;
  uptime: number;
}

/**
 * Representa uma planta/processo no sistema
 */
export interface Plant {
  id: string;
  name: string;
  connected: boolean;
  paused: boolean;
  data: PlantDataPoint[];
  setpoint: number;
  limits: AlarmLimits;
  stats: PlantStats;
  controllers: Controller[];
}

/**
 * Dados simulados da planta
 */
export interface PlantSimulationData {
  plantId: string;
  output: number;
  input: number;
  timeConstant: number;
}

// Re-export Controller for convenience
export type { Controller } from './controller';

