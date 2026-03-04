export interface ExportSensor {
  id: string;
  name: string;
  unit: string;
  actuatorIds: string[];
  setpointId: string;
}

export interface ExportActuator {
  id: string;
  name: string;
  unit: string;
  linkedSensorIds: string[];
}

export interface ExportSetpoint {
  id: string;
  sensorId: string;
}

export interface ExportPlantMeta {
  name: string;
  exportedAt: string;
  version: string;
  sampleCount: number;
  duration: number;
}

export interface ExportDataSample {
  time: number;
  sensors: Record<string, number>;
  setpoints: Record<string, number>;
  actuators: Record<string, number>;
}

export interface PlantExportJSON {
  meta: ExportPlantMeta;
  sensors: ExportSensor[];
  actuators: ExportActuator[];
  setpoints: ExportSetpoint[];
  data: ExportDataSample[];
}

export const EXPORT_FORMAT_VERSION = '1.0.0';

export function validatePlantExportJSON(obj: unknown): string | null {
  if (!obj || typeof obj !== 'object') {
    return 'Arquivo inválido: não é um objeto JSON';
  }

  const json = obj as Record<string, unknown>;

  if (!json.meta || typeof json.meta !== 'object') {
    return 'Arquivo inválido: campo "meta" ausente';
  }
  const meta = json.meta as Record<string, unknown>;
  if (typeof meta.name !== 'string') return 'meta.name deve ser uma string';
  if (typeof meta.version !== 'string') return 'meta.version deve ser uma string';
  if (typeof meta.sampleCount !== 'number') return 'meta.sampleCount deve ser um número';

  if (!Array.isArray(json.sensors)) {
    return 'Arquivo inválido: campo "sensors" deve ser um array';
  }
  for (const s of json.sensors as unknown[]) {
    const sensor = s as Record<string, unknown>;
    if (typeof sensor?.id !== 'string') return 'Cada sensor deve ter um "id" string';
    if (typeof sensor?.name !== 'string') return 'Cada sensor deve ter um "name" string';
    if (!Array.isArray(sensor?.actuatorIds)) return 'Cada sensor deve ter "actuatorIds" array';
    if (typeof sensor?.setpointId !== 'string') return 'Cada sensor deve ter "setpointId" string';
  }

  if (!Array.isArray(json.actuators)) {
    return 'Arquivo inválido: campo "actuators" deve ser um array';
  }
  for (const a of json.actuators as unknown[]) {
    const act = a as Record<string, unknown>;
    if (typeof act?.id !== 'string') return 'Cada atuador deve ter um "id" string';
    if (typeof act?.name !== 'string') return 'Cada atuador deve ter um "name" string';
    if (!Array.isArray(act?.linkedSensorIds)) return 'Cada atuador deve ter "linkedSensorIds" array';
  }

  if (!Array.isArray(json.setpoints)) {
    return 'Arquivo inválido: campo "setpoints" deve ser um array';
  }

  if (!Array.isArray(json.data)) {
    return 'Arquivo inválido: campo "data" deve ser um array';
  }
  if ((json.data as unknown[]).length === 0) {
    return 'Arquivo inválido: campo "data" está vazio';
  }
  const firstSample = (json.data as unknown[])[0] as Record<string, unknown>;
  if (typeof firstSample?.time !== 'number') return 'Cada amostra deve ter "time" numérico';
  if (!firstSample?.sensors || typeof firstSample.sensors !== 'object') return 'Cada amostra deve ter "sensors" objeto';
  if (!firstSample?.setpoints || typeof firstSample.setpoints !== 'object') return 'Cada amostra deve ter "setpoints" objeto';
  if (!firstSample?.actuators || typeof firstSample.actuators !== 'object') return 'Cada amostra deve ter "actuators" objeto';

  return null;
}
