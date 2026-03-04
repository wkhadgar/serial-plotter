import type { ProcessedVariableData, AnalyzerActuatorInfo } from '$lib/types/analyzer';
import type { PlantExportJSON, ExportSensor } from '$lib/types/plantExport';
import { validatePlantExportJSON } from '$lib/types/plantExport';

interface BackendResponse {
  success: boolean;
  error?: string;
  variables?: ProcessedVariableData[];
  plantName?: string;
}

function calcRange(values: number[]): { min: number; max: number } {
  if (values.length === 0) return { min: 0, max: 100 };
  const min = Math.min(...values);
  const max = Math.max(...values);
  const pad = (max - min) * 0.1 || 1;
  return { min: Math.floor(min - pad), max: Math.ceil(max + pad) };
}

function convertJSONToProcessedVariables(json: PlantExportJSON): ProcessedVariableData[] {
  const result: ProcessedVariableData[] = [];

  json.sensors.forEach((sensor: ExportSensor, index: number) => {
    const sensorData = json.data.map((sample) => ({
      time: sample.time,
      value: sample.sensors[sensor.id] ?? 0,
    }));

    const setpointData = json.data.map((sample) => ({
      time: sample.time,
      value: sample.setpoints[sensor.setpointId] ?? 0,
    }));

    const linkedActuators = json.actuators.filter((a) =>
      sensor.actuatorIds.includes(a.id)
    );

    const actuatorsData = linkedActuators.map((act) => ({
      id: act.id,
      name: act.name,
      data: json.data.map((sample) => ({
        time: sample.time,
        value: sample.actuators[act.id] ?? 0,
      })),
    }));

    const actuatorInfos: AnalyzerActuatorInfo[] = linkedActuators.map((a) => ({
      id: a.id,
      name: a.name,
      unit: a.unit,
    }));

    const allSensorValues = sensorData.map((d) => d.value);
    const allSetpointValues = setpointData.map((d) => d.value);
    const allActuatorValues = actuatorsData.flatMap((a) => a.data.map((d) => d.value));

    const sensorRange = calcRange([...allSensorValues, ...allSetpointValues]);
    const actuatorRange = calcRange(allActuatorValues);

    result.push({
      variable: {
        index,
        sensorId: sensor.id,
        sensorName: sensor.name,
        sensorUnit: sensor.unit,
        setpointId: sensor.setpointId,
        actuators: actuatorInfos,
        selected: false,
      },
      sensorData,
      setpointData,
      actuatorsData,
      sensorRange,
      actuatorRange,
    });
  });

  return result;
}

export async function processJSONFile(file: File): Promise<BackendResponse> {
  try {
    const text = await file.text();
    let parsed: unknown;

    try {
      parsed = JSON.parse(text);
    } catch {
      return { success: false, error: 'Arquivo não contém JSON válido' };
    }

    const validationError = validatePlantExportJSON(parsed);
    if (validationError) {
      return { success: false, error: validationError };
    }

    const json = parsed as PlantExportJSON;
    const variables = convertJSONToProcessedVariables(json);

    return {
      success: true,
      variables,
      plantName: json.meta.name,
    };
  } catch (error) {
    return {
      success: false,
      error: `Erro ao ler arquivo: ${error instanceof Error ? error.message : 'Erro desconhecido'}`,
    };
  }
}
