import type { ProcessedVariableData, AnalyzerActuatorInfo } from '$lib/types/analyzer';
import type { PlantExportJSON, ExportSensor } from '$lib/types/plantExport';
import { validatePlantExportJSON } from '$lib/types/plantExport';

interface BackendResponse {
  success: boolean;
  error?: string;
  variables?: ProcessedVariableData[];
  plantName?: string;
}

/**
 * ============================================================================
 * ANALYZER BACKEND SERVICE
 * ============================================================================
 *
 * Lê um arquivo JSON no formato PlantExportJSON, valida a estrutura,
 * e converte para ProcessedVariableData[] pronto para plotagem.
 *
 * Arquitetura preparada para integração com backend Rust via Tauri IPC:
 * basta substituir a leitura do arquivo por `invoke('analyze_json', { json })`.
 */

/**
 * Calcula range com padding de 10%
 */
function calcRange(values: number[]): { min: number; max: number } {
  if (values.length === 0) return { min: 0, max: 100 };
  const min = Math.min(...values);
  const max = Math.max(...values);
  const pad = (max - min) * 0.1 || 1;
  return { min: Math.floor(min - pad), max: Math.ceil(max + pad) };
}

/**
 * Converte o JSON exportado em ProcessedVariableData[] para o Analyzer.
 * Cada sensor gera uma entrada com seu setpoint e atuadores vinculados.
 */
function convertJSONToProcessedVariables(json: PlantExportJSON): ProcessedVariableData[] {
  const result: ProcessedVariableData[] = [];

  json.sensors.forEach((sensor: ExportSensor, index: number) => {
    // Dados do sensor ao longo do tempo
    const sensorData = json.data.map((sample) => ({
      time: sample.time,
      value: sample.sensors[sensor.id] ?? 0,
    }));

    // Dados do setpoint
    const setpointData = json.data.map((sample) => ({
      time: sample.time,
      value: sample.setpoints[sensor.setpointId] ?? 0,
    }));

    // Dados dos atuadores vinculados
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

    // Calcula ranges
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

/**
 * Processa um arquivo JSON exportado pelo Plotter.
 * Lê o arquivo, valida e converte para dados de análise.
 */
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

/**
 * Future Tauri command (commented out for reference)
 *
 * import { invoke } from '@tauri-apps/api/core';
 *
 * export async function processJSONFile(filePath: string): Promise<BackendResponse> {
 *   return await invoke('analyze_json_file', { filePath });
 * }
 */
