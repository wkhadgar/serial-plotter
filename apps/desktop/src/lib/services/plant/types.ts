import type {
  PlantDataPoint,
  PlantSeriesCatalog,
  PlantSeriesDescriptor,
  PlantStats,
  PlantVariable,
  VariableStats,
} from '$lib/types/plant';
import type { Controller, ControllerParam, ParamType } from '$lib/types/controller';
import type { PluginInstance, PluginRuntime, SchemaFieldValue } from '$lib/types/plugin';

export interface CreatePlantVariableDto {
  name: string;
  type: 'sensor' | 'atuador';
  unit: string;
  setpoint: number;
  pv_min: number;
  pv_max: number;
  linked_sensor_ids?: string[];
}

export interface ControllerParamDto {
  type: ParamType;
  value: SchemaFieldValue;
  label: string;
}

export interface CreatePlantDriverDto {
  plugin_id: string;
  config: Record<string, SchemaFieldValue>;
}

export interface CreatePlantControllerDto {
  id?: string | null;
  plugin_id: string;
  name: string;
  controller_type: string;
  active: boolean;
  input_variable_ids: string[];
  output_variable_ids: string[];
  params: Record<string, ControllerParamDto>;
}

export interface CreatePlantDto {
  name: string;
  sample_time_ms: number;
  variables: CreatePlantVariableDto[];
  driver: CreatePlantDriverDto;
  controllers: CreatePlantControllerDto[];
}

export interface UpdatePlantDto extends CreatePlantDto {
  id: string;
}

export interface PlantStatsDto {
  dt: number;
  uptime: number;
}

export interface PlantDriverDto {
  plugin_id: string;
  plugin_name: string;
  runtime: PluginRuntime;
  source_file?: string | null;
  source_code?: string | null;
  config: Record<string, SchemaFieldValue>;
}

export interface PlantControllerDto {
  id: string;
  plugin_id: string;
  plugin_name?: string;
  name: string;
  controller_type: string;
  active: boolean;
  input_variable_ids: string[];
  output_variable_ids: string[];
  params: Record<string, ControllerParamDto>;
  runtime_status?: 'synced' | 'pending_restart';
}

export interface PlantDto {
  id: string;
  name: string;
  sample_time_ms?: number;
  connected: boolean;
  paused: boolean;
  variables: CreatePlantVariableDto[];
  stats: PlantStatsDto;
  driver: PlantDriverDto;
  controllers: PlantControllerDto[];
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
  warning?: string;
  error?: string;
}

export interface OpenPlantFileCommandRequest {
  fileName: string;
  content: string;
}

export interface OpenPlantFileCommandResponse {
  plant: {
    id: string;
    name: string;
    sample_time_ms: number;
    connected: boolean;
    paused: boolean;
    variables: CreatePlantVariableDto[];
    stats: PlantStatsDto;
    driver?: {
      plugin_id: string;
      plugin_name: string;
      config: Record<string, SchemaFieldValue>;
    } | null;
  };
  data: PlantDataPoint[];
  stats: PlantStats;
  variable_stats: Array<{
    error_avg?: number;
    errorAvg?: number;
    stability?: number;
    ripple?: number;
  }>;
  series_catalog: {
    plant_id?: string;
    plantId?: string;
    series: PlantSeriesCatalog['series'];
  };
}

export interface ImportPlantFileCommandResponse {
  plant: PlantDto;
  data: PlantDataPoint[];
  stats: PlantStats;
  variable_stats: Array<{
    error_avg?: number;
    errorAvg?: number;
    stability?: number;
    ripple?: number;
  }>;
  series_catalog: {
    plant_id?: string;
    plantId?: string;
    series: PlantSeriesCatalog['series'];
  };
}

export interface PlantTelemetryPacket {
  plantId: string;
  points: PlantDataPoint[];
  stats?: PlantStats;
  variableStats?: VariableStats[];
  series?: PlantSeriesDescriptor[];
}

export type PlantRuntimeLifecycleState =
  | 'created'
  | 'bootstrapping'
  | 'ready'
  | 'connecting'
  | 'running'
  | 'stopping'
  | 'stopped'
  | 'faulted';

export type PlantRuntimeCyclePhase =
  | 'cycle_started'
  | 'read_inputs'
  | 'compute_controllers'
  | 'write_outputs'
  | 'publish_telemetry'
  | 'sleep_until_deadline';

export interface PlantRuntimeTelemetryEvent {
  plant_id: string;
  runtime_id: string;
  lifecycle_state: PlantRuntimeLifecycleState;
  cycle_phase: PlantRuntimeCyclePhase;
  timestamp: number;
  cycle_id: number;
  configured_sample_time_ms: number;
  effective_dt_ms: number;
  cycle_duration_ms: number;
  read_duration_ms: number;
  control_duration_ms: number;
  write_duration_ms: number;
  publish_duration_ms: number;
  cycle_late: boolean;
  late_by_ms: number;
  phase: string;
  uptime_s: number;
  sensors: Record<string, number>;
  actuators: Record<string, number>;
  actuators_read: Record<string, number>;
  setpoints: Record<string, number>;
  controller_outputs: Record<string, number>;
  written_outputs: Record<string, number>;
  controller_durations_ms: Record<string, number>;
}

export interface PlantRuntimeStatusEvent {
  plant_id: string;
  runtime_id: string;
  lifecycle_state: PlantRuntimeLifecycleState;
  cycle_phase: PlantRuntimeCyclePhase;
  configured_sample_time_ms: number;
  effective_dt_ms: number;
  cycle_late: boolean;
}

export interface PlantRuntimeErrorEvent {
  plant_id: string;
  runtime_id: string;
  message: string;
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

export interface SaveControllerInstanceConfigRequest {
  plantId: string;
  controller: Controller;
  source?: 'backend' | 'workspace';
}

export interface SaveControllerInstanceConfigResponse {
  success: boolean;
  plant?: import('$lib/types/plant').Plant;
  deferred?: boolean;
  error?: string;
}

export interface RemoveControllerInstanceRequest {
  plantId: string;
  controllerId: string;
}

export interface SavePlantSetpointRequest {
  plantId: string;
  variableId: string;
  setpoint: number;
}

export interface ListPlantsResponse {
  plants: import('$lib/types/plant').Plant[];
}

export type { ControllerParam };
