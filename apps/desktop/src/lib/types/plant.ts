import type { Controller } from './controller';

/**
 * ============================================================================
 * TIPOS DE PLANTA - Estrutura Multi-Variável
 * ============================================================================
 * 
 * Uma planta pode ter múltiplas variáveis de controle.
 * Cada variável tem seu próprio PV, SP e MV.
 */

/**
 * Ponto de dados de uma variável individual
 */
export interface VariableDataPoint {
  time: number;
  pv: number;   // Process Variable (valor lido do sensor)
  sp: number;   // Setpoint (valor desejado)
  mv: number;   // Manipulated Variable (saída do controlador)
}

/**
 * Estatísticas de uma variável
 */
export interface VariableStats {
  errorAvg: number;     // Erro médio absoluto (|SP - PV|)
  stability: number;    // Estabilidade (0-100%) baseada no ripple/ruído
  ripple: number;       // Variação ponto-a-ponto do PV (ruído/oscilação)
}

/**
 * Tipo de variável: sensor (medição) ou atuador (ação)
 */
export type VariableType = 'sensor' | 'atuador';

export const VARIABLE_TYPE_LABELS: Record<VariableType, string> = {
  sensor: 'Sensor',
  atuador: 'Atuador',
};

/**
 * Definição de uma variável de controle
 */
export interface PlantVariable {
  id: string;
  name: string;           // Nome da variável (ex: "Temperatura", "Pressão")
  type: VariableType;     // Tipo: sensor (medição) ou atuador (ação)
  unit: string;           // Unidade (ex: "°C", "bar", "%")
  setpoint: number;       // Setpoint atual (só para sensores)
  pvMin: number;          // Range mínimo do PV (para escala do gráfico)
  pvMax: number;          // Range máximo do PV
  mvMin: number;          // Range mínimo do MV
  mvMax: number;          // Range máximo do MV
  linkedSensorIds?: string[];  // IDs dos sensores vinculados (só para atuadores)
}

/**
 * Ponto de dados da planta completa (todas as variáveis)
 * Estrutura: { time, var_0_pv, var_0_sp, var_0_mv, var_1_pv, ... }
 */
export interface PlantDataPoint {
  time: number;
  [key: string]: number;  // var_{index}_{pv|sp|mv}
}

/**
 * Utilitários para acessar dados de variáveis
 */
export function getVariableKeys(varIndex: number) {
  return {
    pv: `var_${varIndex}_pv`,
    sp: `var_${varIndex}_sp`,
    mv: `var_${varIndex}_mv`,
  };
}

export function extractVariableData(
  data: PlantDataPoint[],
  varIndex: number
): VariableDataPoint[] {
  const keys = getVariableKeys(varIndex);
  return data.map(point => ({
    time: point.time,
    pv: point[keys.pv] ?? 0,
    sp: point[keys.sp] ?? 0,
    mv: point[keys.mv] ?? 0,
  }));
}

export interface PlantStats {
  errorAvg: number;
  stability: number;
  uptime: number;
}

export interface Plant {
  id: string;
  name: string;
  connected: boolean;
  paused: boolean;
  variables: PlantVariable[];  // Array de variáveis (pode ser 1 ou mais)
  stats: PlantStats;
  controllers: Controller[];
}

/**
 * Cria uma variável padrão
 */
export function createDefaultVariable(index: number, name?: string, type: VariableType = 'sensor'): PlantVariable {
  return {
    id: `var_${index}`,
    name: name ?? `Variável ${index + 1}`,
    type,
    unit: '%',
    setpoint: type === 'sensor' ? 50 : 0,
    pvMin: 0,
    pvMax: 100,
    mvMin: 0,
    mvMax: 100,
    ...(type === 'atuador' ? { linkedSensorIds: [] } : {}),
  };
}

export type { Controller } from './controller';

