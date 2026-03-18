import type {
  PlantDataPoint,
  PlantSeriesCatalog,
  PlantSeriesDescriptor,
  PlantStats,
  PlantTelemetryBufferConfig,
  VariableStats,
} from '$lib/types/plant';

const _data = new Map<string, PlantDataPoint[]>();
const _stats = new Map<string, PlantStats>();
const _variableStats = new Map<string, VariableStats>();
const _seriesCatalog = new Map<string, PlantSeriesDescriptor[]>();
const _bufferConfig = new Map<string, PlantTelemetryBufferConfig>();

const DEFAULT_STATS: Readonly<PlantStats> = Object.freeze({
  dt: 0,
  uptime: 0,
});

const DEFAULT_VAR_STATS: Readonly<VariableStats> = Object.freeze({
  errorAvg: 0,
  stability: 100,
  ripple: 0,
});

const DEFAULT_BUFFER_CONFIG: Readonly<PlantTelemetryBufferConfig> = Object.freeze({
  maxPoints: 20_000,
  trimTo: 15_000,
});

function normalizeBufferConfig(config: Partial<PlantTelemetryBufferConfig> = {}): PlantTelemetryBufferConfig {
  const maxPoints = Math.max(1_000, Math.floor(config.maxPoints ?? DEFAULT_BUFFER_CONFIG.maxPoints));
  const trimTo = Math.min(maxPoints, Math.max(500, Math.floor(config.trimTo ?? DEFAULT_BUFFER_CONFIG.trimTo)));

  return {
    maxPoints,
    trimTo,
  };
}

function getResolvedBufferConfig(plantId: string): PlantTelemetryBufferConfig {
  return _bufferConfig.get(plantId) ?? { ...DEFAULT_BUFFER_CONFIG };
}

function trimDataInPlace(plantId: string, data: PlantDataPoint[]): PlantDataPoint[] {
  const config = getResolvedBufferConfig(plantId);

  if (data.length <= config.maxPoints) {
    return data;
  }

  data.splice(0, data.length - config.trimTo);
  return data;
}

function normalizeSeriesCatalog(series: PlantSeriesDescriptor[]): PlantSeriesDescriptor[] {
  const byKey = new Map<string, PlantSeriesDescriptor>();

  for (const item of series) {
    const key = item.key.trim();

    if (!key) {
      continue;
    }

    byKey.set(key, {
      key,
      label: item.label.trim() || key,
      role: item.role,
    });
  }

  return Array.from(byKey.values());
}

function getVariableStatsKey(plantId: string, varIndex: number): string {
  return `${plantId}_var_${varIndex}`;
}

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

export function setPlantData(plantId: string, data: PlantDataPoint[]): void {
  _data.set(plantId, data);
}

export function appendPlantData(plantId: string, points: PlantDataPoint[]): PlantDataPoint[] {
  if (points.length === 0) {
    return getPlantData(plantId);
  }

  const config = getResolvedBufferConfig(plantId);

  if (points.length >= config.maxPoints) {
    const next = points.slice(-config.trimTo);
    _data.set(plantId, next);
    return next;
  }

  const data = getPlantData(plantId);
  data.push(...points);
  return trimDataInPlace(plantId, data);
}

export function setPlantBufferConfig(
  plantId: string,
  config: Partial<PlantTelemetryBufferConfig>
): PlantTelemetryBufferConfig {
  const next = normalizeBufferConfig({
    ...getResolvedBufferConfig(plantId),
    ...config,
  });

  _bufferConfig.set(plantId, next);
  trimDataInPlace(plantId, getPlantData(plantId));
  return next;
}

export function getPlantBufferConfig(plantId: string): PlantTelemetryBufferConfig {
  return getResolvedBufferConfig(plantId);
}

export function getPlantSeriesCatalog(plantId: string): PlantSeriesDescriptor[] {
  return _seriesCatalog.get(plantId) ?? [];
}

export function getPlantSeriesLabel(plantId: string, key: string, fallback: string = key): string {
  return getPlantSeriesCatalog(plantId).find((series) => series.key === key)?.label ?? fallback;
}

export function setPlantSeriesCatalog(payload: PlantSeriesCatalog): PlantSeriesDescriptor[] {
  const normalized = normalizeSeriesCatalog(payload.series);
  _seriesCatalog.set(payload.plantId, normalized);
  return normalized;
}

export function seedPlantSeriesCatalog(payload: PlantSeriesCatalog): PlantSeriesDescriptor[] {
  const existing = getPlantSeriesCatalog(payload.plantId);

  if (existing.length === 0) {
    return setPlantSeriesCatalog(payload);
  }

  const missing = normalizeSeriesCatalog(
    payload.series.filter((item) => !existing.some((current) => current.key === item.key))
  );

  if (missing.length === 0) {
    return existing;
  }

  const merged = [...existing, ...missing];
  _seriesCatalog.set(payload.plantId, merged);
  return merged;
}

export function ingestPlantTelemetry(payload: {
  plantId: string;
  points?: PlantDataPoint[];
  stats?: PlantStats;
  variableStats?: VariableStats[];
  series?: PlantSeriesDescriptor[];
}): PlantDataPoint[] {
  if (payload.series?.length) {
    setPlantSeriesCatalog({
      plantId: payload.plantId,
      series: payload.series,
    });
  }

  if (payload.stats) {
    setPlantStats(payload.plantId, payload.stats);
  }

  payload.variableStats?.forEach((stats, index) => {
    setVariableStats(payload.plantId, index, stats);
  });

  return appendPlantData(payload.plantId, payload.points ?? []);
}

export function getVariableStats(plantId: string, varIndex: number): VariableStats {
  return _variableStats.get(getVariableStatsKey(plantId, varIndex)) ?? { ...DEFAULT_VAR_STATS };
}

export function setVariableStats(plantId: string, varIndex: number, stats: VariableStats): void {
  _variableStats.set(getVariableStatsKey(plantId, varIndex), stats);
}

export function clearVariableStats(plantId: string): void {
  for (const key of _variableStats.keys()) {
    if (key.startsWith(`${plantId}_var_`)) {
      _variableStats.delete(key);
    }
  }
}

export function clearPlant(plantId: string): void {
  const data = getPlantData(plantId);
  data.length = 0;
  _stats.set(plantId, { ...DEFAULT_STATS });
  _seriesCatalog.delete(plantId);
  _bufferConfig.delete(plantId);
  clearVariableStats(plantId);
}
