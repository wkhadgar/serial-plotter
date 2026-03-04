import { invoke } from '@tauri-apps/api/core';
import type { Plant, PlantVariable } from '$lib/types/plant';
import type { DriverConfig } from '$lib/types/driver';
import type { Controller } from '$lib/types/controller';
import { generateId } from '$lib/utils/format';

const mockDelay = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

export interface CreatePlantBackendRequest {
  name: string;
  variables: Array<{
    name: string;
    type: 'sensor' | 'atuador';
    unit: string;
    setpoint: number;
    pv_min: number;
    pv_max: number;
    linked_sensor_ids?: string[];
  }>;
  driver_id?: string | null;
  controller_ids?: string[] | null;
}

export interface CreatePlantBackendResponse {
  id: string;
  name: string;
  connected: boolean;
  paused: boolean;
  variables: PlantVariable[];
  stats: {
    dt: number;
    uptime: number;
  };
  driver_id?: string | null;
  controller_ids?: string[] | null;
}

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

export async function createPlant(request: CreatePlantRequest): Promise<CreatePlantResponse> {
  try {
    if (!request.name.trim()) {
      return { success: false, error: 'Nome da planta é obrigatório' };
    }

    if (request.variables.length === 0) {
      return { success: false, error: 'Pelo menos uma variável deve ser definida' };
    }

    const backendRequest: CreatePlantBackendRequest = {
      name: request.name,
      variables: request.variables.map(v => ({
        name: v.name,
        type: v.type,
        unit: v.unit,
        setpoint: v.setpoint,
        pv_min: v.pvMin,
        pv_max: v.pvMax,
        linked_sensor_ids: v.linkedSensorIds,
      })),
      driver_id: request.driverId || null,
      controller_ids: null,
    };

    const backendResponse = await invoke<CreatePlantBackendResponse>(
      'create_plant',
      { request: backendRequest }
    );

    const plant: Plant = {
      id: backendResponse.id,
      name: backendResponse.name,
      connected: backendResponse.connected,
      paused: backendResponse.paused,
      variables: backendResponse.variables,
      stats: {
        dt: backendResponse.stats.dt,
        uptime: backendResponse.stats.uptime,
      },
      controllers: request.controllers || [], // Controladores padrão/vazios por enquanto
    };

    return { success: true, plant };
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : 'Erro desconhecido ao criar planta';
    console.error('Erro ao criar planta:', error);
    return { success: false, error: errorMessage };
  }
}

export async function openPlant(request: OpenPlantRequest): Promise<OpenPlantResponse> {
  await mockDelay(600);
  if (!request.filePath.endsWith('.plant') && !request.filePath.endsWith('.json')) {
    return { success: false, error: 'Formato de arquivo não suportado. Use .plant ou .json' };
  }

  if (Math.random() < 0.15) {
    return { success: false, error: 'Arquivo corrompido ou inválido' };
  }

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
    }],
    stats: { dt: 0, uptime: 0 },
    controllers: [],
  };

  return { success: true, plant };
}

export async function saveDriver(request: SaveDriverRequest): Promise<SaveDriverResponse> {
  await mockDelay(400);

  if (!request.driver.name.trim()) {
    return { success: false, error: 'Nome do driver é obrigatório' };
  }

  const driver: DriverConfig = {
    ...request.driver,
    id: generateId(),
  };

  console.log('Novo driver criado:', driver);

  return { success: true, driver };
}

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
