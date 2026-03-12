import type {
  PlantDataPoint,
  PlantSeriesCatalog,
  PlantSeriesDescriptor,
  PlantStats,
  PlantVariable,
  VariableStats,
} from '$lib/types/plant';
import type { Controller } from '$lib/types/controller';
import type { PluginInstance } from '$lib/types/plugin';

export interface CreatePlantVariableDto {
  name: string;
  type: 'sensor' | 'atuador';
  unit: string;
  setpoint: number;
  pv_min: number;
  pv_max: number;
  linked_sensor_ids?: string[];
}

export interface CreatePlantDto {
  name: string;
  sample_time_ms: number;
  variables: CreatePlantVariableDto[];
  driver_id?: string | null;
  controller_ids?: string[] | null;
}

export interface PlantStatsDto {
  dt: number;
  uptime: number;
}

export interface PlantDto {
  id: string;
  name: string;
  sample_time_ms?: number;
  connected: boolean;
  paused: boolean;
  variables: CreatePlantVariableDto[];
  stats: PlantStatsDto;
  driver_id?: string | null;
  controller_ids?: string[] | null;
}

export interface CreatePlantRequest {
  name: string;
  sampleTimeMs: number;
  driver: PluginInstance | null;
  variables: PlantVariable[];
  controllers: Controller[];
}

export interface UpdatePlantRequest {
  id: string;
  name: string;
  sampleTimeMs: number;
  driver: PluginInstance | null;
  variables: PlantVariable[];
  controllers: Controller[];
  source?: 'backend' | 'workspace';
}

export interface CreatePlantResponse {
  success: boolean;
  plant?: import('$lib/types/plant').Plant;
  error?: string;
}

export interface OpenPlantRequest {
  filePath: string;
  file?: File;
}

export interface OpenPlantResponse {
  success: boolean;
  plant?: import('$lib/types/plant').Plant;
  data?: PlantDataPoint[];
  stats?: PlantStats;
  variableStats?: VariableStats[];
  seriesCatalog?: PlantSeriesCatalog;
  error?: string;
}

export interface PlantTelemetryPacket {
  plantId: string;
  points: PlantDataPoint[];
  stats?: PlantStats;
  variableStats?: VariableStats[];
  series?: PlantSeriesDescriptor[];
}

export interface GetPlantRequest {
  id: string;
}

export interface RemovePlantRequest {
  id: string;
}

export interface PlantActionRequest {
  id: string;
}

export interface PlantActionResponse {
  success: boolean;
  plant?: import('$lib/types/plant').Plant;
  error?: string;
}

export interface ListPlantsResponse {
  plants: import('$lib/types/plant').Plant[];
}
