/**
 * ============================================================================
 * PLUGIN BACKEND SERVICE (MOCK)
 * ============================================================================
 *
 * Serviço mock para operações com plugins.
 * Preparado para integração com backend Rust via Tauri IPC.
 *
 * Funções:
 * - listPlugins: lista plugins disponíveis (registrados/importados)
 * - validatePluginFile: valida JSON de plugin importado
 * - registerPlugin: registra um plugin no sistema  
 * - validatePluginInstance: valida uma config de instância
 */

import type {
  PluginDefinition,
  PluginFileJSON,
  SchemaFieldType,
} from '$lib/types/plugin';
import { validatePluginJSON, getDefaultValueForType, isFieldRequired, AUTO_SCHEMA_FIELDS } from '$lib/types/plugin';
import { generateId } from '$lib/utils/format';

// ─── Response Types ─────────────────────────────────────────────────────────

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

// ─── In-Memory Plugin Registry (mock) ───────────────────────────────────────

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
  },
];

// ─── Helpers ────────────────────────────────────────────────────────────────

function mockDelay(ms: number = 300): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// ─── Public API ─────────────────────────────────────────────────────────────

/**
 * Lista todos os plugins registrados.
 * Filtrável por kind.
 */
export async function listPlugins(kind?: 'driver' | 'controller'): Promise<PluginDefinition[]> {
  await mockDelay(150);
  if (kind) return _pluginRegistry.filter((p) => p.kind === kind);
  return [..._pluginRegistry];
}

/**
 * Busca um plugin pelo ID.
 */
export async function getPlugin(pluginId: string): Promise<PluginDefinition | null> {
  await mockDelay(50);
  return _pluginRegistry.find((p) => p.id === pluginId) ?? null;
}

/**
 * Valida um arquivo JSON de plugin.
 * Em produção, chamaria o backend Rust para validar o código-fonte também.
 */
export async function validatePluginFile(json: unknown): Promise<ValidatePluginResponse> {
  await mockDelay(500);

  const error = validatePluginJSON(json);
  if (error) {
    return { success: false, error };
  }

  const fileJson = json as PluginFileJSON;

  // Normaliza schema (garante defaultValue quando presente)
  const schema = fileJson.schema.map((field) => ({
    ...field,
    ...(field.defaultValue !== undefined
      ? { defaultValue: field.defaultValue }
      : {}),
  }));

  // Auto-injeta num_sensors/num_actuators para drivers
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
    description: fileJson.description,
    version: fileJson.version,
    author: fileJson.author,
  };

  return { success: true, plugin };
}

/**
 * Registra um plugin no sistema (cria ou importa).
 * Retorna o plugin com ID atribuído.
 */
export async function registerPlugin(plugin: PluginDefinition): Promise<RegisterPluginResponse> {
  await mockDelay(400);

  // Verifica nome duplicado
  if (_pluginRegistry.some((p) => p.name === plugin.name && p.id !== plugin.id)) {
    return { success: false, error: `Já existe um plugin com o nome "${plugin.name}"` };
  }

  // Se já existe (mesmo ID), atualiza
  const idx = _pluginRegistry.findIndex((p) => p.id === plugin.id);
  if (idx >= 0) {
    _pluginRegistry[idx] = plugin;
  } else {
    _pluginRegistry.push(plugin);
  }

  return { success: true, plugin };
}

/**
 * Valida os valores de configuração de uma instância.
 * Em produção, chamaria o backend para validação mais profunda.
 */
export async function validatePluginInstanceConfig(
  pluginId: string,
  config: Record<string, unknown>
): Promise<{ success: boolean; error?: string }> {
  await mockDelay(200);

  const plugin = _pluginRegistry.find((p) => p.id === pluginId);
  if (!plugin) return { success: false, error: 'Plugin não encontrado' };

  // Valida campos obrigatórios (sem defaultValue = obrigatório)
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

/**
 * Future Tauri commands:
 *
 * export async function listPlugins(kind?: string): Promise<PluginDefinition[]> {
 *   return await invoke('list_plugins', { kind });
 * }
 *
 * export async function validatePluginFile(filePath: string): Promise<ValidatePluginResponse> {
 *   return await invoke('validate_plugin_file', { filePath });
 * }
 *
 * export async function registerPlugin(plugin: PluginDefinition): Promise<RegisterPluginResponse> {
 *   return await invoke('register_plugin', { plugin });
 * }
 */
