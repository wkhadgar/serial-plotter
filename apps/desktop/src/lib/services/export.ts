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

function classifyVariables(variables: PlantVariable[]) {
  const sensors = variables.filter((v) => v.type === 'sensor');
  const actuators = variables.filter((v) => v.type === 'atuador');
  return { sensors, actuators };
}


export function exportPlantDataCSV(plant: Plant, data: PlantDataPoint[]): boolean {
  if (data.length === 0) return false;

  const { sensors, actuators } = classifyVariables(plant.variables);
  const varIndex = (v: PlantVariable) => parseInt(v.id.replace('var_', ''), 10);

  const columns: { header: string; getValue: (pt: PlantDataPoint) => number }[] = [];

  columns.push({ header: 'seconds', getValue: (pt) => pt.time });

  for (const s of sensors) {
    const idx = varIndex(s);
    const name = sanitizeName(s.name);
    columns.push({ header: `sensor_${name}`, getValue: (pt) => pt[`var_${idx}_pv`] ?? 0 });
    columns.push({ header: `setpoint_${name}`, getValue: (pt) => pt[`var_${idx}_sp`] ?? 0 });
  }

  for (const a of actuators) {
    const idx = varIndex(a);
    const name = sanitizeName(a.name);
    columns.push({ header: `atuador_${name}`, getValue: (pt) => pt[`var_${idx}_pv`] ?? 0 });
  }

  const headerLine = columns.map((c) => c.header).join(',');
  const rows = data.map((pt) =>
    columns.map((c) => {
      const v = c.getValue(pt);
      return Number.isFinite(v) ? v.toFixed(4) : '0';
    }).join(',')
  );

  const csv = [headerLine, ...rows].join('\n');
  triggerDownload(csv, `${sanitizeName(plant.name)}_data.csv`, 'text/csv');
  return true;
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

export function exportPlantDataJSON(plant: Plant, data: PlantDataPoint[]): boolean {
  if (data.length === 0) return false;

  const json = buildPlantExportJSON(plant, data);
  const content = JSON.stringify(json, null, 2);
  triggerDownload(content, `${sanitizeName(plant.name)}_data.json`, 'application/json');
  return true;
}
