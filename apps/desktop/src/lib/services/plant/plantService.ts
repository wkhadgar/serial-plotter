import { invoke } from '@tauri-apps/api/core';
import { ingestPlantTelemetry } from '$lib/stores/plantData';
import {
  buildPlantSeriesCatalog,
  type Plant,
  type PlantDataPoint,
  type PlantStats,
  type PlantVariable,
  type VariableStats,
} from '$lib/types/plant';
import type {
  CreatePlantDto,
  CreatePlantRequest,
  CreatePlantResponse,
  OpenPlantRequest,
  OpenPlantResponse,
  PlantActionResponse,
  PlantTelemetryPacket,
  PlantDto,
  UpdatePlantRequest,
} from './types';
import { generateId } from '$lib/utils/format';
import { validatePlantExportJSON } from '$lib/types/plantExport';

const STORAGE_KEY = 'senamby.desktop.plants.workspace';

type PlantWorkspaceState = {
  workspacePlants: Plant[];
  plantOverrides: Record<string, Plant>;
  deletedBackendPlantIds: string[];
};

const DEFAULT_WORKSPACE_STATE: PlantWorkspaceState = {
  workspacePlants: [],
  plantOverrides: {},
  deletedBackendPlantIds: [],
};

const DEFAULT_SAMPLE_TIME_MS = 100;

function normalizeSampleTimeMs(sampleTimeMs: number | null | undefined, fallback = DEFAULT_SAMPLE_TIME_MS): number {
  const resolved = Number(sampleTimeMs);
  if (!Number.isFinite(resolved)) return fallback;
  return Math.max(1, Math.round(resolved));
}

function canUseStorage(): boolean {
  return typeof window !== 'undefined' && typeof localStorage !== 'undefined';
}

function loadWorkspaceState(): PlantWorkspaceState {
  if (!canUseStorage()) return structuredClone(DEFAULT_WORKSPACE_STATE);

  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return structuredClone(DEFAULT_WORKSPACE_STATE);
    const parsed = JSON.parse(raw) as PlantWorkspaceState;
    return {
      workspacePlants: Array.isArray(parsed.workspacePlants) ? parsed.workspacePlants : [],
      plantOverrides: parsed.plantOverrides ?? {},
      deletedBackendPlantIds: Array.isArray(parsed.deletedBackendPlantIds) ? parsed.deletedBackendPlantIds : [],
    };
  } catch (error) {
    console.error('Erro ao carregar estado local de plantas:', error);
    return structuredClone(DEFAULT_WORKSPACE_STATE);
  }
}

function saveWorkspaceState(state: PlantWorkspaceState): void {
  if (!canUseStorage()) return;
  localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
}

function normalizePlant(plant: Plant): Plant {
  const sampleTimeMs = normalizeSampleTimeMs(
    plant.sampleTimeMs,
    normalizeSampleTimeMs(plant.stats?.dt ? plant.stats.dt * 1000 : undefined)
  );

  return {
    ...plant,
    sampleTimeMs,
    controllers: plant.controllers ?? [],
    driver: plant.driver ?? null,
    driverId: plant.driver?.pluginId ?? plant.driverId ?? null,
    stats: {
      dt: plant.stats?.dt && plant.stats.dt > 0 ? plant.stats.dt : sampleTimeMs / 1000,
      uptime: plant.stats?.uptime ?? 0,
    },
    source: plant.source ?? 'workspace',
  };
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
    controllers: [],
    driverId: dto.driver_id ?? null,
    driver: null,
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

function buildCreatePlantDto(request: CreatePlantRequest): CreatePlantDto {
  const sampleTimeMs = normalizeSampleTimeMs(request.sampleTimeMs);

  return {
    name: request.name.trim(),
    sample_time_ms: sampleTimeMs,
    variables: request.variables.map(mapVariableToDto),
    driver_id: request.driver?.pluginId ?? null,
    controller_ids: null,
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

function upsertWorkspacePlant(plant: Plant): Plant {
  const normalized = normalizePlant({ ...plant, source: 'workspace' });
  const state = loadWorkspaceState();
  const index = state.workspacePlants.findIndex((entry) => entry.id === normalized.id);

  if (index >= 0) {
    state.workspacePlants[index] = normalized;
  } else {
    state.workspacePlants.unshift(normalized);
  }

  saveWorkspaceState(state);
  return normalized;
}

function savePlantOverride(plant: Plant): Plant {
  const normalized = normalizePlant(plant);
  const state = loadWorkspaceState();
  state.plantOverrides[normalized.id] = normalized;
  saveWorkspaceState(state);
  return normalized;
}

function clearPlantLocalState(plantId: string): void {
  const state = loadWorkspaceState();
  state.workspacePlants = state.workspacePlants.filter((plant) => plant.id !== plantId);
  delete state.plantOverrides[plantId];
  state.deletedBackendPlantIds = state.deletedBackendPlantIds.filter((id) => id !== plantId);
  saveWorkspaceState(state);
}

function markBackendPlantDeleted(plantId: string): void {
  const state = loadWorkspaceState();
  if (!state.deletedBackendPlantIds.includes(plantId)) {
    state.deletedBackendPlantIds.push(plantId);
  }
  saveWorkspaceState(state);
}

function mergePlants(base: Plant, override?: Plant): Plant {
  if (!override) return normalizePlant(base);
  return normalizePlant({
    ...base,
    ...override,
    stats: override.stats ?? base.stats,
    connected: override.connected,
    paused: override.paused,
  });
}

function updateWorkspacePlant(id: string, updater: (plant: Plant) => Plant): PlantActionResponse {
  const state = loadWorkspaceState();
  const index = state.workspacePlants.findIndex((plant) => plant.id === id);

  if (index < 0) {
    return { success: false, error: 'Planta não encontrada' };
  }

  state.workspacePlants[index] = normalizePlant(updater(state.workspacePlants[index]));
  saveWorkspaceState(state);
  return { success: true, plant: state.workspacePlants[index] };
}

async function listBackendPlants(): Promise<Plant[]> {
  try {
    const response = await invoke<PlantDto[]>('list_plants');
    return response.map(mapDtoToPlant);
  } catch (error) {
    console.warn('Backend de plantas indisponível, usando somente workspace local:', error);
    return [];
  }
}

export async function createPlant(request: CreatePlantRequest): Promise<CreatePlantResponse> {
  if (!request.name.trim()) {
    return { success: false, error: 'Nome da planta é obrigatório' };
  }

  const sampleTimeMs = normalizeSampleTimeMs(request.sampleTimeMs);

  if (request.variables.length === 0) {
    return { success: false, error: 'Pelo menos uma variável deve ser definida' };
  }

  try {
    const response = await invoke<PlantDto>('create_plant', { request: buildCreatePlantDto(request) });
    const plant = savePlantOverride({
      ...mapDtoToPlant(response),
      sampleTimeMs,
      driver: request.driver,
      driverId: request.driver?.pluginId ?? response.driver_id ?? null,
      controllers: request.controllers ?? [],
      source: 'backend',
    });

    return { success: true, plant };
  } catch (error) {
    console.warn('Falha ao criar planta no backend, salvando no workspace:', error);
  }

  const plant = upsertWorkspacePlant({
    id: generateId(),
    name: request.name.trim(),
    sampleTimeMs,
    connected: false,
    paused: false,
    variables: request.variables,
    controllers: request.controllers ?? [],
    driver: request.driver,
    driverId: request.driver?.pluginId ?? null,
    stats: {
      dt: sampleTimeMs / 1000,
      uptime: 0,
    },
    source: 'workspace',
  });

  return { success: true, plant };
}

export async function updatePlant(request: UpdatePlantRequest): Promise<PlantActionResponse> {
  const current = await getPlant(request.id);
  if (!current) {
    return { success: false, error: 'Planta não encontrada' };
  }

  const sampleTimeMs = normalizeSampleTimeMs(request.sampleTimeMs, current.sampleTimeMs);
  const currentDtMs = current.stats.dt > 0 ? Math.round(current.stats.dt * 1000) : 0;
  const shouldRefreshConfiguredDt = !current.connected || currentDtMs === current.sampleTimeMs;

  const updatedPlant: Plant = normalizePlant({
    id: request.id,
    name: request.name.trim(),
    sampleTimeMs,
    variables: request.variables,
    controllers: request.controllers,
    driver: request.driver,
    driverId: request.driver?.pluginId ?? current.driverId ?? null,
    connected: current.connected,
    paused: current.paused,
    stats: shouldRefreshConfiguredDt
      ? {
          ...current.stats,
          dt: sampleTimeMs / 1000,
        }
      : current.stats,
    source: request.source ?? current.source ?? 'workspace',
  });

  if ((request.source ?? current.source) === 'backend') {
    return { success: true, plant: savePlantOverride({ ...current, ...updatedPlant, source: 'backend' }) };
  }

  return { success: true, plant: upsertWorkspacePlant(updatedPlant) };
}

export async function listPlants(): Promise<Plant[]> {
  const state = loadWorkspaceState();
  const backendPlants = await listBackendPlants();

  const mergedBackend = backendPlants
    .filter((plant) => !state.deletedBackendPlantIds.includes(plant.id))
    .map((plant) => mergePlants(plant, state.plantOverrides[plant.id]));

  return [...state.workspacePlants.map(normalizePlant), ...mergedBackend];
}

export async function getPlant(id: string): Promise<Plant | null> {
  const plants = await listPlants();
  return plants.find((plant) => plant.id === id) ?? null;
}

export async function removePlant(id: string): Promise<PlantActionResponse> {
  const plant = await getPlant(id);

  if (!plant) {
    return { success: false, error: 'Planta não encontrada' };
  }

  if (plant.source === 'backend') {
    try {
      await invoke<PlantDto>('remove_plant', { id });
      clearPlantLocalState(id);
      return { success: true, plant };
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Erro ao remover planta no backend';
      return { success: false, error: message };
    }
  }

  clearPlantLocalState(id);
  return { success: true, plant };
}

async function invokePlantAction(command: string, id: string, merge: (current: Plant, backend: Plant) => Plant): Promise<PlantActionResponse> {
  const current = await getPlant(id);
  if (!current) {
    return { success: false, error: 'Planta não encontrada' };
  }

  if (current.source !== 'backend') {
    return { success: false, error: 'Ação disponível apenas para plantas integradas ao backend' };
  }

  try {
    const response = await invoke<PlantDto>(command, { id });
    const merged = savePlantOverride(merge(current, mapDtoToPlant(response)));
    return { success: true, plant: merged };
  } catch (error) {
    const message = error instanceof Error ? error.message : 'Erro ao sincronizar ação da planta';
    return { success: false, error: message };
  }
}

export async function connectPlant(id: string): Promise<PlantActionResponse> {
  const current = await getPlant(id);
  if (current?.source !== 'backend') {
    return updateWorkspacePlant(id, (plant) => ({ ...plant, connected: true, paused: false }));
  }

  return invokePlantAction('connect_plant', id, (currentPlant, backendPlant) => ({
    ...backendPlant,
    driver: currentPlant.driver,
    controllers: currentPlant.controllers,
    source: 'backend',
  }));
}

export async function disconnectPlant(id: string): Promise<PlantActionResponse> {
  const current = await getPlant(id);
  if (current?.source !== 'backend') {
    return updateWorkspacePlant(id, (plant) => ({ ...plant, connected: false, paused: false }));
  }

  return invokePlantAction('disconnect_plant', id, (currentPlant, backendPlant) => ({
    ...backendPlant,
    driver: currentPlant.driver,
    controllers: currentPlant.controllers,
    source: 'backend',
  }));
}

export async function pausePlant(id: string): Promise<PlantActionResponse> {
  const current = await getPlant(id);
  if (current?.source !== 'backend') {
    return updateWorkspacePlant(id, (plant) => ({ ...plant, paused: true }));
  }

  return invokePlantAction('pause_plant', id, (currentPlant, backendPlant) => ({
    ...backendPlant,
    driver: currentPlant.driver,
    controllers: currentPlant.controllers,
    source: 'backend',
  }));
}

export async function resumePlant(id: string): Promise<PlantActionResponse> {
  const current = await getPlant(id);
  if (current?.source !== 'backend') {
    return updateWorkspacePlant(id, (plant) => ({ ...plant, paused: false }));
  }

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
    const parsed = JSON.parse(text) as Record<string, unknown>;
    const validationError = validatePlantExportJSON(parsed);

    if (validationError) {
      return { success: false, error: validationError };
    }

    const sensors = (parsed.sensors as Array<Record<string, unknown>>).map((sensor, index) => ({
      id: `var_${index}`,
      name: sensor.name as string,
      type: 'sensor' as const,
      unit: (sensor.unit as string) ?? '%',
      setpoint: 0,
      pvMin: 0,
      pvMax: 100,
    }));

    const actuatorsOffset = sensors.length;
    const actuators = (parsed.actuators as Array<Record<string, unknown>>).map((actuator, index) => ({
      id: `var_${actuatorsOffset + index}`,
      name: actuator.name as string,
      type: 'atuador' as const,
      unit: (actuator.unit as string) ?? '%',
      setpoint: 0,
      pvMin: 0,
      pvMax: 100,
      linkedSensorIds: Array.isArray(actuator.linkedSensorIds)
        ? (actuator.linkedSensorIds as string[]).map((sensorId) => {
            const sensorIndex = (parsed.sensors as Array<Record<string, unknown>>).findIndex((sensor) => sensor.id === sensorId);
            return sensorIndex >= 0 ? `var_${sensorIndex}` : sensorId;
          })
        : [],
    }));

    const sensorIndexByExportId = new Map(
      (parsed.sensors as Array<Record<string, unknown>>).map((sensor, index) => [sensor.id as string, index])
    );
    const actuatorIndexByExportId = new Map(
      (parsed.actuators as Array<Record<string, unknown>>).map((actuator, index) => [actuator.id as string, actuatorsOffset + index])
    );
    const setpointSensorMap = new Map(
      (parsed.setpoints as Array<Record<string, unknown>>).map((setpoint) => [setpoint.id as string, setpoint.sensorId as string])
    );

    const data = (parsed.data as Array<Record<string, unknown>>).map((sample) => {
      const point: PlantDataPoint = {
        time: Number(sample.time ?? 0),
      };

      const sensorsRecord = sample.sensors as Record<string, number>;
      for (const [sensorId, value] of Object.entries(sensorsRecord)) {
        const index = sensorIndexByExportId.get(sensorId);
        if (index !== undefined) {
          point[`var_${index}_pv`] = Number(value ?? 0);
        }
      }

      const setpointsRecord = sample.setpoints as Record<string, number>;
      for (const [setpointId, value] of Object.entries(setpointsRecord)) {
        const sensorId = setpointSensorMap.get(setpointId);
        const index = sensorId ? sensorIndexByExportId.get(sensorId) : undefined;
        if (index !== undefined) {
          point[`var_${index}_sp`] = Number(value ?? 0);
        }
      }

      const actuatorsRecord = sample.actuators as Record<string, number>;
      for (const [actuatorId, value] of Object.entries(actuatorsRecord)) {
        const index = actuatorIndexByExportId.get(actuatorId);
        if (index !== undefined) {
          point[`var_${index}_pv`] = Number(value ?? 0);
        }
      }

      return point;
    });

    const stats = computePlantStats(data);
    const plant = upsertWorkspacePlant({
      id: generateId(),
      name: ((parsed.meta as Record<string, unknown>).name as string) ?? request.filePath,
      sampleTimeMs: normalizeSampleTimeMs((parsed.meta as Record<string, unknown>).sampleTimeMs as number | undefined, stats.dt * 1000),
      connected: false,
      paused: false,
      variables: [...sensors, ...actuators],
      controllers: [],
      driver: null,
      driverId: null,
      stats,
      source: 'workspace',
    });

    const variableStats = plant.variables.map((variable, index) => computeVariableStats(data, index, variable));
    const seriesCatalog = buildPlantSeriesCatalog(plant.id, plant.variables);

    return {
      success: true,
      plant,
      data,
      stats: plant.stats,
      variableStats,
      seriesCatalog,
    };
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : 'Erro ao abrir arquivo';
    return { success: false, error: errorMessage };
  }
}

export function applyPlantTelemetryPacket(packet: PlantTelemetryPacket): PlantDataPoint[] {
  return ingestPlantTelemetry(packet);
}
