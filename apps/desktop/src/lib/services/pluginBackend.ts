import type {
  PluginDefinition,
  PluginFileJSON,
  SchemaFieldType,
} from '$lib/types/plugin';
import { validatePluginJSON, getDefaultValueForType, isFieldRequired, AUTO_SCHEMA_FIELDS, DRIVER_REQUIRED_METHODS } from '$lib/types/plugin';
import { generateId } from '$lib/utils/format';


export interface ValidatePluginResponse {
  success: boolean;
  plugin?: PluginDefinition;
  error?: string;
}

export interface RegisterPluginResponse {
  success: boolean;
  plugin?: PluginDefinition;
  error?: string;
}

const _pluginRegistry: PluginDefinition[] = [
  {
    id: 'plg_modbus_tcp',
    name: 'Modbus TCP Driver',
    kind: 'driver',
    runtime: 'python',
    sourceFile: 'modbus_tcp_driver.py',
    description: 'Driver de comunicação Modbus TCP',
    version: '1.0.0',
    schema: [
      { name: 'num_sensors', type: 'int', description: 'Número de sensores que o driver vai lidar' },
      { name: 'num_actuators', type: 'int', description: 'Número de atuadores que o driver vai lidar' },
      { name: 'host', type: 'string', defaultValue: '192.168.1.100', description: 'Endereço IP do dispositivo' },
      { name: 'port', type: 'int', defaultValue: 502, description: 'Porta TCP' },
      { name: 'unit_id', type: 'int', defaultValue: 1, description: 'ID da unidade Modbus' },
      { name: 'timeout', type: 'float', defaultValue: 1.0, description: 'Timeout de conexão (s)' },
      { name: 'auto_reconnect', type: 'bool', defaultValue: true, description: 'Reconectar automaticamente' },
      { name: 'register_addresses', type: 'list', defaultValue: [40001, 40002, 40003], description: 'Endereços de registros a ler' },
    ],
    dependencies: [
      { name: 'pymodbus', version: '>=3.0.0' },
    ],
  },
  {
    id: 'plg_serial_raw',
    name: 'Serial Raw Driver',
    kind: 'driver',
    runtime: 'python',
    sourceFile: 'serial_raw_driver.py',
    description: 'Comunicação serial direta',
    version: '1.0.0',
    schema: [
      { name: 'num_sensors', type: 'int', description: 'Número de sensores que o driver vai lidar' },
      { name: 'num_actuators', type: 'int', description: 'Número de atuadores que o driver vai lidar' },
      { name: 'port', type: 'string', defaultValue: '/dev/ttyUSB0', description: 'Porta serial' },
      { name: 'baud_rate', type: 'int', defaultValue: 9600, description: 'Baud rate' },
      { name: 'data_bits', type: 'int', defaultValue: 8 },
      { name: 'parity', type: 'string', defaultValue: 'none' },
      { name: 'stop_bits', type: 'int', defaultValue: 1 },
    ],
    dependencies: [
      { name: 'pyserial', version: '>=3.5' },
    ],
  },
  {
    id: 'plg_mqtt',
    name: 'MQTT Driver',
    kind: 'driver',
    runtime: 'python',
    sourceFile: 'mqtt_driver.py',
    description: 'Driver MQTT para IoT',
    version: '1.0.0',
    schema: [
      { name: 'num_sensors', type: 'int', description: 'Número de sensores que o driver vai lidar' },
      { name: 'num_actuators', type: 'int', description: 'Número de atuadores que o driver vai lidar' },
      { name: 'broker', type: 'string', defaultValue: 'localhost', description: 'Endereço do broker' },
      { name: 'port', type: 'int', defaultValue: 1883 },
      { name: 'client_id', type: 'string', defaultValue: 'senamby-client' },
      { name: 'topic', type: 'string', description: 'Tópico base' },
      { name: 'username', type: 'string', defaultValue: '' },
      { name: 'password', type: 'string', defaultValue: '' },
      { name: 'use_tls', type: 'bool', defaultValue: false },
      { name: 'subscriptions', type: 'list', defaultValue: ['sensors/#', 'actuators/#'], description: 'Tópicos extras' },
    ],
    dependencies: [
      { name: 'paho-mqtt', version: '>=2.0.0' },
    ],
  },
];


function mockDelay(ms: number = 300): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}


export async function listPlugins(kind?: 'driver' | 'controller'): Promise<PluginDefinition[]> {
  await mockDelay(150);
  if (kind) return _pluginRegistry.filter((p) => p.kind === kind);
  return [..._pluginRegistry];
}

export async function getPlugin(pluginId: string): Promise<PluginDefinition | null> {
  await mockDelay(50);
  return _pluginRegistry.find((p) => p.id === pluginId) ?? null;
}

export async function validatePluginFile(json: unknown): Promise<ValidatePluginResponse> {
  await mockDelay(500);

  const error = validatePluginJSON(json);
  if (error) {
    return { success: false, error };
  }

  const fileJson = json as PluginFileJSON;

  const schema = fileJson.schema.map((field) => ({
    ...field,
    ...(field.defaultValue !== undefined
      ? { defaultValue: field.defaultValue }
      : {}),
  }));

  const finalSchema = fileJson.kind === 'driver'
    ? [...AUTO_SCHEMA_FIELDS, ...schema.filter(f => f.name !== 'num_sensors' && f.name !== 'num_actuators')]
    : schema;

  const plugin: PluginDefinition = {
    id: 'plg_' + generateId(),
    name: fileJson.name,
    kind: fileJson.kind,
    runtime: fileJson.runtime,
    sourceFile: fileJson.sourceFile,
    schema: finalSchema,
    dependencies: fileJson.dependencies,
    description: fileJson.description,
    version: fileJson.version,
    author: fileJson.author,
  };

  return { success: true, plugin };
}

export async function registerPlugin(plugin: PluginDefinition): Promise<RegisterPluginResponse> {
  await mockDelay(400);

  if (_pluginRegistry.some((p) => p.name === plugin.name && p.id !== plugin.id)) {
    return { success: false, error: `Já existe um plugin com o nome "${plugin.name}"` };
  }

  const idx = _pluginRegistry.findIndex((p) => p.id === plugin.id);
  if (idx >= 0) {
    _pluginRegistry[idx] = plugin;
  } else {
    _pluginRegistry.push(plugin);
    console.log('Novo driver criado:', plugin);
  }

  return { success: true, plugin };
}

export async function validatePluginInstanceConfig(
  pluginId: string,
  config: Record<string, unknown>
): Promise<{ success: boolean; error?: string }> {
  await mockDelay(200);

  const plugin = _pluginRegistry.find((p) => p.id === pluginId);
  if (!plugin) return { success: false, error: 'Plugin não encontrado' };

  for (const field of plugin.schema) {
    if (isFieldRequired(field)) {
      const val = config[field.name];
      if (val === undefined || val === null || val === '') {
        return { success: false, error: `Campo "${field.name}" é obrigatório` };
      }
    }
  }

  return { success: true };
}

export interface ValidateDriverCodeResponse {
  success: boolean;
  errors?: string[];
}

export async function validateDriverSourceCode(
  code: string,
  expectedClassName?: string
): Promise<ValidateDriverCodeResponse> {
  await mockDelay(300);

  const errors: string[] = [];

  const classPattern = /class\s+(\w+)\s*\(\s*MCUDriver\s*\)/;
  const classMatch = code.match(classPattern);

  if (!classMatch) {
    errors.push('Nenhuma classe que herda de MCUDriver foi encontrada');
    return { success: false, errors };
  }

  const foundClass = classMatch[1];

  if (expectedClassName && foundClass !== expectedClassName) {
    errors.push(`Nome da classe esperado: "${expectedClassName}", encontrado: "${foundClass}"`);
  }

  for (const method of DRIVER_REQUIRED_METHODS) {
    const methodPattern = new RegExp(`def\\s+${method}\\s*\\(`);
    if (!methodPattern.test(code)) {
      errors.push(`Método obrigatório ausente: ${method}()`);
    }
  }

  if (errors.length > 0) {
    return { success: false, errors };
  }

  return { success: true };
}
