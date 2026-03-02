/**
 * ============================================================================
 * TIPOS DO ANALYZER - Baseado no formato PlantExportJSON
 * ============================================================================
 *
 * Após importar um JSON exportado, os dados são convertidos para estas
 * estruturas internas que alimentam os gráficos do Analyzer.
 */

/**
 * Informações de um atuador vinculado a um sensor
 */
export interface AnalyzerActuatorInfo {
  id: string;
  name: string;
  unit: string;
}

/**
 * Uma variável (sensor) do analyzer com seus relacionamentos
 */
export interface AnalyzerVariable {
  index: number;
  sensorId: string;
  sensorName: string;
  sensorUnit: string;
  setpointId: string;
  actuators: AnalyzerActuatorInfo[];
  selected: boolean;
}

/**
 * Dados processados de um sensor para plotagem
 */
export interface ProcessedVariableData {
  variable: AnalyzerVariable;
  sensorData: Array<{ time: number; value: number }>;
  setpointData: Array<{ time: number; value: number }>;
  actuatorsData: Array<{
    id: string;
    name: string;
    data: Array<{ time: number; value: number }>;
  }>;
  sensorRange: { min: number; max: number };
  actuatorRange: { min: number; max: number };
}
