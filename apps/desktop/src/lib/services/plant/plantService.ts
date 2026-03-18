import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { ingestPlantTelemetry } from '$lib/stores/plantData';
import {
  type Plant,
  type PlantDataPoint,
  type PlantSeriesCatalog,
  type PlantStats,
  type PlantVariable,
  type VariableStats,
} from '$lib/types/plant';
import type { Controller, ControllerParam } from '$lib/types/controller';
import type { PluginInstance, SchemaFieldValue } from '$lib/types/plugin';
import { listPluginsByType } from '$lib/services/plugin';
import type {
  ControllerParamDto,
  CreatePlantDto,
  CreatePlantRequest,
  CreatePlantResponse,
  OpenPlantRequest,
  OpenPlantResponse,
  OpenPlantFileCommandResponse,
  PlantActionResponse,
  PlantControllerDto,
  PlantDriverDto,
  PlantRuntimeErrorEvent,
  PlantRuntimeStatusEvent,
  PlantRuntimeTelemetryEvent,
  PlantTelemetryPacket,
  PlantDto,
  SaveControllerInstanceConfigRequest,
  SaveControllerInstanceConfigResponse,
  UpdatePlantDto,
  UpdatePlantRequest,
} from './types';

const DEFAULT_SAMPLE_TIME_MS = 100;

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function resolveSerializedErrorMessage(value: unknown): string | null {
  if (typeof value !== 'string') return null;

  const trimmed = value.trim();
  if (!trimmed || trimmed === '[object Object]') return null;

  try {
    return extractErrorMessage(JSON.parse(trimmed), trimmed);
  } catch {
    return trimmed;
  }
}

function extractErrorMessage(error: unknown, fallback: string): string {
  if (typeof error === 'string') {
    return resolveSerializedErrorMessage(error) ?? fallback;
  }

  if (error instanceof Error) {
    return resolveSerializedErrorMessage(error.message) ?? fallback;
  }

  if (isRecord(error)) {
    const message = resolveSerializedErrorMessage(error.message) ?? resolveSerializedErrorMessage(error.error);
    if (message) return message;
  }

  return fallback;
}

function toFiniteNumber(value: unknown, fallback = 0): number {
  const resolved = Number(value);
  return Number.isFinite(resolved) ? resolved : fallback;
}

function isSchemaFieldValue(value: unknown): value is SchemaFieldValue {
  if (
    typeof value === 'boolean' ||
    typeof value === 'number' ||
    typeof value === 'string'
  ) {
    return true;
  }

  if (!Array.isArray(value)) {
    return false;
  }

  return value.every((entry) => isSchemaFieldValue(entry));
}

function normalizeImportedDriverConfig(value: unknown): Record<string, SchemaFieldValue> {
  if (!isRecord(value)) {
    return {};
  }

  const config: Record<string, SchemaFieldValue> = {};
  for (const [key, entry] of Object.entries(value)) {
    if (!key.trim()) continue;
    if (!isSchemaFieldValue(entry)) continue;
    config[key] = entry;
  }

  return config;
}

function normalizeImportedDriver(
  payload: unknown
): { pluginId: string; pluginName: string; config: Record<string, SchemaFieldValue> } | null {
  if (!isRecord(payload)) {
    return null;
  }

  const rawPluginId = typeof payload.plugin_id === 'string'
    ? payload.plugin_id
    : typeof payload.pluginId === 'string'
      ? payload.pluginId
      : '';
  const pluginId = rawPluginId.trim();
  if (!pluginId) {
    return null;
  }

  const rawPluginName = typeof payload.plugin_name === 'string'
    ? payload.plugin_name
    : typeof payload.pluginName === 'string'
      ? payload.pluginName
      : '';
  const pluginName = rawPluginName.trim() || pluginId;

  return {
    pluginId,
    pluginName,
    config: normalizeImportedDriverConfig(payload.config),
  };
}

function normalizeSampleTimeMs(sampleTimeMs: number | null | undefined, fallback = DEFAULT_SAMPLE_TIME_MS): number {
  const resolved = Number(sampleTimeMs);
  if (!Number.isFinite(resolved)) return fallback;
  return Math.max(1, Math.round(resolved));
}

function mapVariableDtoToFrontend(variable: PlantDto['variables'][number], index: number): PlantVariable {
  return {
    id: `var_${index}`,
    name: variable.name,
    type: variable.type,
    unit: variable.unit,
    setpoint: variable.setpoint,
    pvMin: variable.pv_min,
    pvMax: variable.pv_max,
    linkedSensorIds: variable.linked_sensor_ids ?? [],
  };
}

function mapDriverDtoToFrontend(driver: PlantDriverDto): PluginInstance {
  return {
    pluginId: driver.plugin_id,
    pluginName: driver.plugin_name,
    pluginKind: 'driver',
    config: driver.config ?? {},
  };
}

function mapControllerParamDtoToFrontend(param: ControllerParamDto): ControllerParam {
  return {
    type: param.type,
    value: param.value as ControllerParam['value'],
    label: param.label,
  };
}

function mapControllerDtoToFrontend(controller: PlantControllerDto): Controller {
  return {
    id: controller.id,
    pluginId: controller.plugin_id,
    name: controller.name,
    type: controller.controller_type,
    active: controller.active,
    inputVariableIds: controller.input_variable_ids ?? [],
    outputVariableIds: controller.output_variable_ids ?? [],
    params: Object.fromEntries(
      Object.entries(controller.params ?? {}).map(([key, param]) => [key, mapControllerParamDtoToFrontend(param)])
    ),
  };
}

function mapDtoToPlant(dto: PlantDto): Plant {
  const sampleTimeMs = normalizeSampleTimeMs(
    dto.sample_time_ms,
    dto.stats.dt > 0 ? dto.stats.dt * 1000 : undefined
  );

  return {
    id: dto.id,
    name: dto.name,
    sampleTimeMs,
    connected: dto.connected,
    paused: dto.paused,
    variables: dto.variables.map(mapVariableDtoToFrontend),
    stats: {
      dt: dto.stats.dt > 0 ? dto.stats.dt : sampleTimeMs / 1000,
      uptime: dto.stats.uptime,
    },
    controllers: (dto.controllers ?? []).map(mapControllerDtoToFrontend),
    driverId: dto.driver.plugin_id,
    driver: mapDriverDtoToFrontend(dto.driver),
    source: 'backend',
  };
}

function mapVariableToDto(variable: PlantVariable): CreatePlantDto['variables'][number] {
  return {
    name: variable.name,
    type: variable.type,
    unit: variable.unit,
    setpoint: variable.setpoint,
    pv_min: variable.pvMin,
    pv_max: variable.pvMax,
    linked_sensor_ids: variable.linkedSensorIds,
  };
}

function mapControllerParamToDto(param: ControllerParam): ControllerParamDto {
  return {
    type: param.type,
    value: param.value,
    label: param.label,
  };
}

function mapControllerToDto(controller: Controller): CreatePlantDto['controllers'][number] {
  return {
    id: controller.id,
    plugin_id: controller.pluginId ?? controller.id,
    name: controller.name,
    controller_type: controller.type,
    active: controller.active,
    input_variable_ids: controller.inputVariableIds ?? [],
    output_variable_ids: controller.outputVariableIds ?? [],
    params: Object.fromEntries(
      Object.entries(controller.params ?? {}).map(([key, param]) => [key, mapControllerParamToDto(param)])
    ),
  };
}

function buildCreatePlantDto(request: CreatePlantRequest): CreatePlantDto {
  const sampleTimeMs = normalizeSampleTimeMs(request.sampleTimeMs);

  return {
    name: request.name.trim(),
    sample_time_ms: sampleTimeMs,
    variables: request.variables.map(mapVariableToDto),
    driver: {
      plugin_id: request.driver!.pluginId,
      config: request.driver!.config ?? {},
    },
    controllers: request.controllers.map(mapControllerToDto),
  };
}

function computePlantStats(data: PlantDataPoint[]): PlantStats {
  if (data.length <= 1) {
    return {
      dt: 0,
      uptime: data[0]?.time ?? 0,
    };
  }

  const deltas: number[] = [];
  for (let index = 1; index < data.length; index += 1) {
    deltas.push(Math.max(0, data[index].time - data[index - 1].time));
  }

  const avgDelta = deltas.reduce((sum, delta) => sum + delta, 0) / deltas.length;
  return {
    dt: Number(avgDelta.toFixed(4)),
    uptime: Math.max(0, data[data.length - 1].time - data[0].time),
  };
}

function computeVariableStats(data: PlantDataPoint[], variableIndex: number, variable: PlantVariable): VariableStats {
  const pvKey = `var_${variableIndex}_pv`;
  const spKey = `var_${variableIndex}_sp`;
  const values = data.map((point) => point[pvKey] ?? 0);

  if (values.length === 0) {
    return { errorAvg: 0, stability: 100, ripple: 0 };
  }

  const min = Math.min(...values);
  const max = Math.max(...values);
  const ripple = Number((max - min).toFixed(3));

  if (variable.type === 'atuador') {
    return { errorAvg: 0, stability: Math.max(0, 100 - ripple), ripple };
  }

  const errorAvg = Number(
    (
      data.reduce((sum, point) => sum + Math.abs((point[pvKey] ?? 0) - (point[spKey] ?? 0)), 0) /
      values.length
    ).toFixed(3)
  );

  return {
    errorAvg,
    stability: Math.max(0, Number((100 - ripple).toFixed(2))),
    ripple,
  };
}

function normalizeImportedVariableStats(payload: unknown): VariableStats {
  const source = isRecord(payload) ? payload : {};
  const errorAvg = toFiniteNumber(source.errorAvg ?? source.error_avg, 0);
  const stability = toFiniteNumber(source.stability, 100);
  const ripple = toFiniteNumber(source.ripple, 0);

  return {
    errorAvg,
    stability,
    ripple,
  };
}

function normalizeImportedSeriesCatalog(
  payload: unknown,
  fallbackPlantId: string
): PlantSeriesCatalog {
  const source = isRecord(payload) ? payload : {};
  const rawSeries = Array.isArray(source.series) ? source.series : [];
  const series = rawSeries
    .map((entry) => {
      const item = isRecord(entry) ? entry : {};
      const key = typeof item.key === 'string' ? item.key.trim() : '';
      const label = typeof item.label === 'string' ? item.label.trim() : '';
      const role = item.role;

      if (!key || (role !== 'pv' && role !== 'sp' && role !== 'mv')) {
        return null;
      }

      return {
        key,
        label: label || key,
        role,
      };
    })
    .filter((entry): entry is PlantSeriesCatalog['series'][number] => entry !== null);

  const plantId = typeof source.plantId === 'string' && source.plantId.trim()
    ? source.plantId
    : typeof source.plant_id === 'string' && source.plant_id.trim()
      ? source.plant_id
      : fallbackPlantId;

  return {
    plantId,
    series,
  };
}

export async function subscribePlantRuntimeEvents(handlers: {
  onTelemetry?: (event: PlantRuntimeTelemetryEvent) => void;
  onStatus?: (event: PlantRuntimeStatusEvent) => void;
  onError?: (event: PlantRuntimeErrorEvent) => void;
}): Promise<() => void> {
  const unlisteners: UnlistenFn[] = [];

  if (handlers.onTelemetry) {
    unlisteners.push(
      await listen<PlantRuntimeTelemetryEvent>('plant://telemetry', (event) => {
        handlers.onTelemetry?.(event.payload);
      })
    );
  }

  if (handlers.onStatus) {
    unlisteners.push(
      await listen<PlantRuntimeStatusEvent>('plant://status', (event) => {
        handlers.onStatus?.(event.payload);
      })
    );
  }

  if (handlers.onError) {
    unlisteners.push(
      await listen<PlantRuntimeErrorEvent>('plant://error', (event) => {
        handlers.onError?.(event.payload);
      })
    );
  }

  return () => {
    for (const unlisten of unlisteners) {
      unlisten();
    }
  };
}

export function buildTelemetryPacketFromRuntimeEvent(
  plant: Plant,
  event: PlantRuntimeTelemetryEvent
): PlantTelemetryPacket {
  const runtimePayload = event.payload ?? {};
  const point: PlantDataPoint = {
    time: Math.max(0, toFiniteNumber(runtimePayload.uptime_s, 0)),
  };

  for (const [index, variable] of plant.variables.entries()) {
    const pvKey = `var_${index}_pv`;
    const spKey = `var_${index}_sp`;
    const sensorValue = toFiniteNumber(runtimePayload.sensors?.[variable.id], 0);
    const actuatorValue = toFiniteNumber(
      runtimePayload.actuators?.[variable.id] ?? runtimePayload.sensors?.[variable.id],
      0
    );

    if (variable.type === 'sensor') {
      point[pvKey] = sensorValue;
      point[spKey] = toFiniteNumber(runtimePayload.setpoints?.[variable.id], variable.setpoint);
      continue;
    }

    point[pvKey] = actuatorValue;
  }

  return {
    plantId: plant.id,
    points: [point],
    stats: {
      dt: Math.max(0, toFiniteNumber(event.effective_dt_ms, plant.sampleTimeMs) / 1000),
      uptime: point.time,
    },
  };
}

async function listBackendPlants(): Promise<Plant[]> {
  const response = await invoke<PlantDto[]>('list_plants');
  return response.map(mapDtoToPlant);
}

async function getBackendPlant(id: string): Promise<Plant | null> {
  try {
    const response = await invoke<PlantDto>('get_plant', { id });
    return mapDtoToPlant(response);
  } catch {
    return null;
  }
}

export async function createPlant(request: CreatePlantRequest): Promise<CreatePlantResponse> {
  if (!request.name.trim()) {
    return { success: false, error: 'Nome da planta é obrigatório' };
  }

  if (!request.driver?.pluginId) {
    return { success: false, error: 'Configure um driver de comunicação para a planta' };
  }

  if (request.variables.length === 0) {
    return { success: false, error: 'Pelo menos uma variável deve ser definida' };
  }

  try {
    const response = await invoke<PlantDto>('create_plant', { request: buildCreatePlantDto(request) });
    const plant = mapDtoToPlant(response);

    return { success: true, plant };
  } catch (error) {
    const message = extractErrorMessage(error, 'Erro ao criar planta no backend');
    return { success: false, error: message };
  }
}

export async function updatePlant(request: UpdatePlantRequest): Promise<PlantActionResponse> {
  const current = await getPlant(request.id);
  if (!current) {
    return { success: false, error: 'Planta não encontrada' };
  }

  const sampleTimeMs = normalizeSampleTimeMs(request.sampleTimeMs, current.sampleTimeMs);
  try {
    const response = await invoke<PlantDto>('update_plant', {
      request: {
        id: request.id,
        name: request.name.trim(),
        sample_time_ms: sampleTimeMs,
        variables: request.variables.map(mapVariableToDto),
        driver: {
          plugin_id: request.driver?.pluginId ?? current.driver?.pluginId ?? current.driverId ?? '',
          config: request.driver?.config ?? current.driver?.config ?? {},
        },
        controllers: request.controllers.map(mapControllerToDto),
      } satisfies UpdatePlantDto,
    });

    return {
      success: true,
      plant: mapDtoToPlant(response),
    };
  } catch (error) {
    const message = extractErrorMessage(error, 'Erro ao atualizar planta no backend');
    return { success: false, error: message };
  }
}

export async function saveControllerInstanceConfig(
  request: SaveControllerInstanceConfigRequest
): Promise<SaveControllerInstanceConfigResponse> {
  if (!request.controller.id) {
    return { success: false, error: 'Controlador não encontrado' };
  }

  if (request.source === 'backend') {
    try {
      await invoke('save_controller_instance_config', {
        request: {
          plant_id: request.plantId,
          controller_id: request.controller.id,
          plugin_id: request.controller.pluginId ?? null,
          name: request.controller.name,
          active: request.controller.active,
          input_variable_ids: request.controller.inputVariableIds ?? [],
          output_variable_ids: request.controller.outputVariableIds ?? [],
          params: Object.entries(request.controller.params ?? {}).map(([key, param]) => ({
            key,
            type: param.type,
            value: param.value,
            label: param.label,
          })),
        },
      });

      return { success: true };
    } catch (error) {
      console.warn(
        'Persistencia de instancia de controlador ainda nao esta disponivel no backend. Mantendo alteracoes localmente.',
        error
      );

      return {
        success: true,
        deferred: true,
      };
    }
  }

  return { success: true };
}

export async function listPlants(): Promise<Plant[]> {
  try {
    return await listBackendPlants();
  } catch (error) {
    console.error('Falha ao listar plantas do backend:', error);
    return [];
  }
}

export async function getPlant(id: string): Promise<Plant | null> {
  return getBackendPlant(id);
}

export async function removePlant(id: string): Promise<PlantActionResponse> {
  try {
    const response = await invoke<PlantDto>('remove_plant', { id });
    return { success: true, plant: mapDtoToPlant(response) };
  } catch (error) {
    const message = extractErrorMessage(error, 'Erro ao remover planta no backend');
    return { success: false, error: message };
  }
}

async function invokePlantAction(command: string, id: string, merge: (current: Plant, backend: Plant) => Plant): Promise<PlantActionResponse> {
  const current = await getPlant(id);
  if (!current) {
    return { success: false, error: 'Planta não encontrada' };
  }

  try {
    const response = await invoke<PlantDto>(command, { id });
    const merged = merge(current, mapDtoToPlant(response));
    return { success: true, plant: merged };
  } catch (error) {
    const message = extractErrorMessage(error, 'Erro ao sincronizar ação da planta');
    return { success: false, error: message };
  }
}

export async function connectPlant(id: string): Promise<PlantActionResponse> {
  const current = await getPlant(id);
  if (current && !current.connected && !current.driver?.pluginId) {
    const hasMissingLinkedDriver = !!current.driverId;
    return {
      success: false,
      error: hasMissingLinkedDriver
        ? 'O driver desta planta não está carregado. Vincule um novo driver antes de ligar.'
        : 'Configure um driver de comunicação para a planta antes de ligar.',
    };
  }

  return invokePlantAction('connect_plant', id, (currentPlant, backendPlant) => ({
    ...backendPlant,
    driver: currentPlant.driver,
    controllers: currentPlant.controllers,
    source: 'backend',
  }));
}

export async function disconnectPlant(id: string): Promise<PlantActionResponse> {
  return invokePlantAction('disconnect_plant', id, (currentPlant, backendPlant) => ({
    ...backendPlant,
    driver: currentPlant.driver,
    controllers: currentPlant.controllers,
    source: 'backend',
  }));
}

export async function pausePlant(id: string): Promise<PlantActionResponse> {
  return invokePlantAction('pause_plant', id, (currentPlant, backendPlant) => ({
    ...backendPlant,
    driver: currentPlant.driver,
    controllers: currentPlant.controllers,
    source: 'backend',
  }));
}

export async function resumePlant(id: string): Promise<PlantActionResponse> {
  return invokePlantAction('resume_plant', id, (currentPlant, backendPlant) => ({
    ...backendPlant,
    driver: currentPlant.driver,
    controllers: currentPlant.controllers,
    source: 'backend',
  }));
}

export async function openPlant(request: OpenPlantRequest): Promise<OpenPlantResponse> {
  if (!request.file) {
    return {
      success: false,
      error: 'Selecione um arquivo de exportação válido para abrir a planta na UI',
    };
  }

  try {
    const text = await request.file.text();
    const response = await invoke<OpenPlantFileCommandResponse>('open_plant_file', {
      request: {
        fileName: request.file.name,
        content: text,
      },
    });
    const importedDriver = normalizeImportedDriver(response.plant.driver);
    const availableDrivers = importedDriver
      ? await listPluginsByType('driver')
      : [];
    const matchedDriver = importedDriver
      ? availableDrivers.find((entry) => entry.id === importedDriver.pluginId) ??
        availableDrivers.find(
          (entry) => entry.name.trim().toLowerCase() === importedDriver.pluginName.trim().toLowerCase()
        )
      : undefined;
    const resolvedDriver: PluginInstance | null = matchedDriver
      ? {
          pluginId: matchedDriver.id,
          pluginName: matchedDriver.name,
          pluginKind: 'driver',
          config: importedDriver?.config ?? {},
        }
      : null;
    if (!resolvedDriver?.pluginId) {
      const message = importedDriver
        ? `Driver "${importedDriver.pluginName}" não está carregado. Vincule/importe o driver e tente novamente.`
        : 'Arquivo de planta sem driver válido. A planta precisa de um driver para ser criada no backend.';
      return { success: false, error: message };
    }

    const createResponse = await invoke<PlantDto>('create_plant', {
      request: {
        name: response.plant.name.trim(),
        sample_time_ms: normalizeSampleTimeMs(response.plant.sample_time_ms, response.stats.dt * 1000),
        variables: response.plant.variables,
        driver: {
          plugin_id: resolvedDriver.pluginId,
          config: resolvedDriver.config ?? {},
        },
        controllers: [],
      } satisfies CreatePlantDto,
    });
    const plant = mapDtoToPlant(createResponse);

    return {
      success: true,
      plant,
      data: response.data,
      stats: response.stats,
      variableStats: (response.variable_stats ?? []).map(normalizeImportedVariableStats),
      seriesCatalog: normalizeImportedSeriesCatalog(response.series_catalog, plant.id),
    };
  } catch (error) {
    const errorMessage = extractErrorMessage(error, 'Erro ao abrir arquivo');
    return { success: false, error: errorMessage };
  }
}

export function applyPlantTelemetryPacket(packet: PlantTelemetryPacket): PlantDataPoint[] {
  return ingestPlantTelemetry(packet);
}
