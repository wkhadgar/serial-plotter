import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import type { Plant, PlantDataPoint, PlantVariable } from '$lib/types/plant';
import type {
  PlantExportJSON,
  ExportSensor,
  ExportActuator,
  ExportSetpoint,
  ExportDataSample,
} from '$lib/types/plantExport';
import { EXPORT_FORMAT_VERSION } from '$lib/types/plantExport';


function sanitizeName(name: string): string {
  return name.replace(/\s+/g, '_');
}

function isTauriRuntime(): boolean {
  if (typeof window === 'undefined') return false;
  return '__TAURI_INTERNALS__' in window || '__TAURI__' in window;
}

function normalizeDialogPath(path: string | string[] | null): string | null {
  if (typeof path === 'string') return path;
  if (Array.isArray(path) && path.length > 0) return path.join('/');
  return null;
}

function triggerDownload(content: string, filename: string, mimeType: string): void {
  const blob = new Blob([content], { type: `${mimeType};charset=utf-8` });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = filename;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
}

async function saveContent(
  content: string,
  filename: string,
  mimeType: string,
  title: string,
  filterName: string,
  extension: string
): Promise<boolean> {
  if (!isTauriRuntime()) {
    triggerDownload(content, filename, mimeType);
    return true;
  }

  try {
    const selected = await save({
      title,
      defaultPath: filename,
      filters: [{ name: filterName, extensions: [extension] }],
    });
    const path = normalizeDialogPath(selected);
    if (!path) return false;

    await invoke('save_export_file', {
      request: {
        path,
        content,
      },
    });
    return true;
  } catch (error) {
    console.error('Falha ao salvar arquivo de exportação:', error);
    return false;
  }
}

function classifyVariables(variables: PlantVariable[]) {
  const sensors = variables.filter((v) => v.type === 'sensor');
  const actuators = variables.filter((v) => v.type === 'atuador');
  return { sensors, actuators };
}


export async function exportPlantDataCSV(plant: Plant, data: PlantDataPoint[]): Promise<boolean> {
  if (data.length === 0) return false;

  const { sensors, actuators } = classifyVariables(plant.variables);
  const varIndex = (v: PlantVariable) => parseInt(v.id.replace('var_', ''), 10);

  const columns: { header: string; getValue: (pt: PlantDataPoint) => number }[] = [];

  columns.push({ header: 'seconds', getValue: (pt) => pt.time });

  sensors.forEach((sensor, sensorIndex) => {
    const idx = varIndex(sensor);
    const labelIndex = sensorIndex + 1;
    columns.push({ header: `sensor_${labelIndex}`, getValue: (pt) => pt[`var_${idx}_pv`] ?? 0 });
    columns.push({ header: `sp_${labelIndex}`, getValue: (pt) => pt[`var_${idx}_sp`] ?? 0 });
  });

  actuators.forEach((actuator, actuatorIndex) => {
    const idx = varIndex(actuator);
    const labelIndex = actuatorIndex + 1;
    columns.push({ header: `atuador_${labelIndex}`, getValue: (pt) => pt[`var_${idx}_pv`] ?? 0 });
  });

  const headerLine = columns.map((c) => c.header).join(',');
  const rows = data.map((pt) =>
    columns.map((c) => {
      const v = c.getValue(pt);
      return Number.isFinite(v) ? v.toFixed(4) : '0';
    }).join(',')
  );

  const csv = [headerLine, ...rows].join('\n');
  return saveContent(
    csv,
    `${sanitizeName(plant.name)}_data.csv`,
    'text/csv',
    'Salvar CSV',
    'CSV',
    'csv'
  );
}


export function buildPlantExportJSON(plant: Plant, data: PlantDataPoint[]): PlantExportJSON {
  const { sensors, actuators } = classifyVariables(plant.variables);
  const varIndex = (v: PlantVariable) => parseInt(v.id.replace('var_', ''), 10);

  const exportSensors: ExportSensor[] = sensors.map((s) => {
    const linked = actuators.filter((a) => a.linkedSensorIds?.includes(s.id));
    return {
      id: s.id,
      name: s.name,
      unit: s.unit,
      actuatorIds: linked.map((a) => a.id),
      setpointId: `sp_${s.id}`,
    };
  });

  const exportActuators: ExportActuator[] = actuators.map((a) => ({
    id: a.id,
    name: a.name,
    unit: a.unit,
    linkedSensorIds: a.linkedSensorIds ?? [],
  }));

  const exportSetpoints: ExportSetpoint[] = sensors.map((s) => ({
    id: `sp_${s.id}`,
    sensorId: s.id,
  }));

  const exportData: ExportDataSample[] = data.map((pt) => {
    const sensorValues: Record<string, number> = {};
    const setpointValues: Record<string, number> = {};
    const actuatorValues: Record<string, number> = {};

    for (const s of sensors) {
      const idx = varIndex(s);
      sensorValues[s.id] = pt[`var_${idx}_pv`] ?? 0;
      setpointValues[`sp_${s.id}`] = pt[`var_${idx}_sp`] ?? 0;
    }

    for (const a of actuators) {
      const idx = varIndex(a);
      actuatorValues[a.id] = pt[`var_${idx}_pv`] ?? 0;
    }

    return {
      time: pt.time,
      sensors: sensorValues,
      setpoints: setpointValues,
      actuators: actuatorValues,
    };
  });

  const duration = data.length > 1 ? data[data.length - 1].time - data[0].time : 0;

  return {
    meta: {
      name: plant.name,
      exportedAt: new Date().toISOString(),
      version: EXPORT_FORMAT_VERSION,
      sampleCount: data.length,
      duration,
      sampleTimeMs: plant.sampleTimeMs,
    },
    sensors: exportSensors,
    actuators: exportActuators,
    setpoints: exportSetpoints,
    data: exportData,
  };
}

export async function exportPlantDataJSON(plant: Plant, data: PlantDataPoint[]): Promise<boolean> {
  if (data.length === 0) return false;

  const json = buildPlantExportJSON(plant, data);
  const content = JSON.stringify(json, null, 2);
  return saveContent(
    content,
    `${sanitizeName(plant.name)}_data.json`,
    'application/json',
    'Salvar JSON',
    'JSON',
    'json'
  );
}
