/**
 * ============================================================================
 * PLANT BACKEND SERVICE (MOCK)
 * ============================================================================
 * 
 * Serviço mock para operações de criação/abertura de plantas.
 * Este será substituído por chamadas reais ao backend Tauri.
 */

import type { Plant, PlantVariable } from '$lib/types/plant';
import type { DriverConfig } from '$lib/types/driver';
import type { Controller } from '$lib/types/controller';
import { generateId } from '$lib/utils/format';

export interface CreatePlantRequest {
  name: string;
  driverId: string;
  variables: PlantVariable[];
  controllers: Controller[];
}

export interface CreatePlantResponse {
  success: boolean;
  plant?: Plant;
  error?: string;
}

export interface OpenPlantRequest {
  filePath: string;
}

export interface OpenPlantResponse {
  success: boolean;
  plant?: Plant;
  error?: string;
}

export interface SaveDriverRequest {
  driver: Omit<DriverConfig, 'id'>;
}

export interface SaveDriverResponse {
  success: boolean;
  driver?: DriverConfig;
  error?: string;
}

/**
 * Mock: Simula delay de rede
 */
function mockDelay(ms: number = 500): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

/**
 * Mock: Cria uma nova planta
 */
export async function createPlant(request: CreatePlantRequest): Promise<CreatePlantResponse> {
  await mockDelay(800);

  // Simula validação
  if (!request.name.trim()) {
    return { success: false, error: 'Nome da planta é obrigatório' };
  }

  if (!request.driverId) {
    return { success: false, error: 'Driver de comunicação é obrigatório' };
  }

  if (request.variables.length === 0) {
    return { success: false, error: 'Pelo menos uma variável deve ser definida' };
  }

  // Simula chance de falha (10%)
  if (Math.random() < 0.1) {
    return { success: false, error: 'Falha ao conectar com o driver de comunicação' };
  }

  const plant: Plant = {
    id: generateId(),
    name: request.name,
    connected: false,
    paused: false,
    variables: request.variables,
    stats: { errorAvg: 0, stability: 100, uptime: 0 },
    controllers: request.controllers,
  };

  return { success: true, plant };
}

/**
 * Mock: Abre uma planta de arquivo
 */
export async function openPlant(request: OpenPlantRequest): Promise<OpenPlantResponse> {
  await mockDelay(600);

  // Simula validação de arquivo
  if (!request.filePath.endsWith('.plant') && !request.filePath.endsWith('.json')) {
    return { success: false, error: 'Formato de arquivo não suportado. Use .plant ou .json' };
  }

  // Simula chance de falha (15%)
  if (Math.random() < 0.15) {
    return { success: false, error: 'Arquivo corrompido ou inválido' };
  }

  // Simula planta carregada
  const plant: Plant = {
    id: generateId(),
    name: request.filePath.split('/').pop()?.replace(/\.(plant|json)$/, '') || 'Planta Importada',
    connected: false,
    paused: false,
    variables: [{
      id: 'var_0',
      name: 'Temperatura',
      type: 'sensor',
      unit: '°C',
      setpoint: 50,
      pvMin: 0,
      pvMax: 100,
      mvMin: 0,
      mvMax: 100,
    }],
    stats: { errorAvg: 0, stability: 100, uptime: 0 },
    controllers: [],
  };

  return { success: true, plant };
}

/**
 * Mock: Salva um novo driver
 */
export async function saveDriver(request: SaveDriverRequest): Promise<SaveDriverResponse> {
  await mockDelay(400);

  if (!request.driver.name.trim()) {
    return { success: false, error: 'Nome do driver é obrigatório' };
  }

  const driver: DriverConfig = {
    ...request.driver,
    id: generateId(),
  };

  return { success: true, driver };
}

/**
 * Mock: Lista drivers salvos
 */
export async function listDrivers(): Promise<DriverConfig[]> {
  await mockDelay(200);

  return [
    {
      id: 'drv_1',
      name: 'PLC Principal - Modbus',
      type: 'modbus-tcp',
      description: 'Controlador principal da linha 1',
      settings: {
        host: '192.168.1.10',
        port: 502,
        unitId: 1,
      },
    },
    {
      id: 'drv_2',
      name: 'Sensor Serial',
      type: 'serial-raw',
      description: 'Sensores de temperatura via RS485',
      settings: {
        port: '/dev/ttyUSB0',
        baudRate: 9600,
        dataBits: 8,
        parity: 'none',
        stopBits: 1,
        lineEnding: 'crlf',
      },
    },
    {
      id: 'drv_3',
      name: 'MQTT Broker Local',
      type: 'mqtt',
      settings: {
        broker: 'localhost',
        port: 1883,
        clientId: 'senamby',
        topic: 'plant/#',
      },
    },
  ];
}

/**
 * Mock: Lista controladores pré-definidos/templates
 */
export async function listControllerTemplates(): Promise<Controller[]> {
  await mockDelay(150);

  return [
    {
      id: 'tpl_pid',
      name: 'PID Clássico',
      type: 'PID',
      active: false,
      params: {
        kp: { type: 'number', value: 1.0, label: 'Kp (Proporcional)' },
        ki: { type: 'number', value: 0.1, label: 'Ki (Integral)' },
        kd: { type: 'number', value: 0.05, label: 'Kd (Derivativo)' },
        manualMode: { type: 'boolean', value: false, label: 'Modo Manual' },
      },
    },
    {
      id: 'tpl_pid_ag',
      name: 'PID Agressivo',
      type: 'PID',
      active: false,
      params: {
        kp: { type: 'number', value: 2.5, label: 'Kp (Proporcional)' },
        ki: { type: 'number', value: 0.3, label: 'Ki (Integral)' },
        kd: { type: 'number', value: 0.1, label: 'Kd (Derivativo)' },
        manualMode: { type: 'boolean', value: false, label: 'Modo Manual' },
      },
    },
    {
      id: 'tpl_onoff',
      name: 'On/Off Style',
      type: 'Level',
      active: false,
      params: {
        hysteresis: { type: 'number', value: 2.0, label: 'Histerese' },
        manualMode: { type: 'boolean', value: false, label: 'Modo Manual' },
      },
    },
  ];
}
