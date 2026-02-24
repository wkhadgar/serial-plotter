import type { Controller } from './controller';

export interface PlantDataPoint {
  time: number;
  sp: number;
  pv: number;
  mv: number;
  [key: string]: number;
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
  data: PlantDataPoint[];
  setpoint: number;
  stats: PlantStats;
  controllers: Controller[];
}

export type { Controller } from './controller';

