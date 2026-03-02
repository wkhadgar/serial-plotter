/**
 * ============================================================================
 * PLANT EXPORT FORMAT - Estrutura JSON de Exportação/Importação
 * ============================================================================
 *
 * Formato canônico para salvar e carregar dados de plantas.
 * Preserva todos os relacionamentos entre sensores, setpoints e atuadores.
 *
 * Usado por:
 * - Plotter → botão de download (CSV e JSON)
 * - Analyzer → importar JSON para análise
 * - Backend futuro → mesmo formato via Tauri IPC
 */

/**
 * Definição de um sensor no arquivo exportado
 */
export interface ExportSensor {
  id: string;
  name: string;
  unit: string;
  actuatorIds: string[];   // IDs dos atuadores vinculados
  setpointId: string;      // ID do setpoint associado (ex: "sp_var_0")
}

/**
 * Definição de um atuador no arquivo exportado
 */
export interface ExportActuator {
  id: string;
  name: string;
  unit: string;
  linkedSensorIds: string[];  // IDs dos sensores aos quais está vinculado
}

/**
 * Definição de um setpoint no arquivo exportado
 */
export interface ExportSetpoint {
  id: string;
  sensorId: string;  // ID do sensor ao qual pertence
}

/**
 * Metadados da planta
 */
export interface ExportPlantMeta {
  name: string;
  exportedAt: string;       // ISO timestamp
  version: string;          // Versão do formato (para compatibilidade futura)
  sampleCount: number;      // Número de amostras
  duration: number;         // Duração total em segundos
}

/**
 * Amostra de dados em um instante de tempo
 */
export interface ExportDataSample {
  time: number;
  sensors: Record<string, number>;    // { "var_0": 45.2, "var_1": 2.3 }
  setpoints: Record<string, number>;  // { "sp_var_0": 50, "sp_var_1": 2.5 }
  actuators: Record<string, number>;  // { "var_2": 72.1, "var_3": 55.0 }
}

/**
 * Formato completo do JSON exportado
 */
export interface PlantExportJSON {
  meta: ExportPlantMeta;
  sensors: ExportSensor[];
  actuators: ExportActuator[];
  setpoints: ExportSetpoint[];
  data: ExportDataSample[];
}

/**
 * Versão atual do formato de exportação
 */
export const EXPORT_FORMAT_VERSION = '1.0.0';

/**
 * Valida se um objeto tem a estrutura esperada de PlantExportJSON
 * Retorna null se válido, ou string de erro se inválido
 */
export function validatePlantExportJSON(obj: unknown): string | null {
  if (!obj || typeof obj !== 'object') {
    return 'Arquivo inválido: não é um objeto JSON';
  }

  const json = obj as Record<string, unknown>;

  // Meta
  if (!json.meta || typeof json.meta !== 'object') {
    return 'Arquivo inválido: campo "meta" ausente';
  }
  const meta = json.meta as Record<string, unknown>;
  if (typeof meta.name !== 'string') return 'meta.name deve ser uma string';
  if (typeof meta.version !== 'string') return 'meta.version deve ser uma string';
  if (typeof meta.sampleCount !== 'number') return 'meta.sampleCount deve ser um número';

  // Sensors
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

  // Actuators
  if (!Array.isArray(json.actuators)) {
    return 'Arquivo inválido: campo "actuators" deve ser um array';
  }
  for (const a of json.actuators as unknown[]) {
    const act = a as Record<string, unknown>;
    if (typeof act?.id !== 'string') return 'Cada atuador deve ter um "id" string';
    if (typeof act?.name !== 'string') return 'Cada atuador deve ter um "name" string';
    if (!Array.isArray(act?.linkedSensorIds)) return 'Cada atuador deve ter "linkedSensorIds" array';
  }

  // Setpoints
  if (!Array.isArray(json.setpoints)) {
    return 'Arquivo inválido: campo "setpoints" deve ser um array';
  }

  // Data
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
