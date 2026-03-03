/**
 * ============================================================================
 * TIPOS DE PLUGIN - Sistema de Plugins Reutilizáveis
 * ============================================================================
 *
 * Um Plugin é uma unidade reutilizável de código que implementa:
 * - Driver de comunicação (leitura/escrita de dispositivos)
 * - Controlador (algoritmo de controle)
 *
 * Cada plugin define um schema de configuração que é instanciado
 * por planta (cada planta pode usar o mesmo plugin com configs diferentes).
 */

// ─── Tipos Básicos ──────────────────────────────────────────────────────────

/** Tipo do plugin */
export type PluginKind = 'driver' | 'controller';

/** Runtime de execução */
export type PluginRuntime = 'python' | 'rust-native';

/** Tipos de parâmetro do schema */
export type SchemaFieldType = 'bool' | 'int' | 'float' | 'string' | 'list';

export const PLUGIN_KIND_LABELS: Record<PluginKind, string> = {
  driver: 'Driver',
  controller: 'Controlador',
};

export const PLUGIN_RUNTIME_LABELS: Record<PluginRuntime, string> = {
  python: 'Python',
  'rust-native': 'Rust Nativo',
};

export const SCHEMA_FIELD_TYPE_LABELS: Record<SchemaFieldType, string> = {
  bool: 'Boolean',
  int: 'Inteiro',
  float: 'Decimal',
  string: 'Texto',
  list: 'Lista',
};

// ─── Campos Auto-injetados ──────────────────────────────────────────────────

/**
 * Campos injetados automaticamente no schema de todo plugin do tipo 'driver'.
 * Representam quantos sensores/atuadores o driver vai lidar naquela instância.
 * São configurados na hora de adicionar o driver à planta.
 */
export const AUTO_SCHEMA_FIELDS: PluginSchemaField[] = [
  { name: 'num_sensors', type: 'int', description: 'Número de sensores que o driver vai lidar' },
  { name: 'num_actuators', type: 'int', description: 'Número de atuadores que o driver vai lidar' },
];

/** Nomes reservados (não podem ser usados pelo usuário) */
export const RESERVED_FIELD_NAMES = AUTO_SCHEMA_FIELDS.map(f => f.name);

// ─── Schema de Configuração ─────────────────────────────────────────────────

/**
 * Um campo do schema de configuração do plugin.
 * Se defaultValue está ausente (undefined), o campo é obrigatório.
 * Se defaultValue está presente, o campo é opcional e usa esse valor como padrão.
 */
export interface PluginSchemaField {
  name: string;                     // Nome do campo (sem caracteres especiais)
  type: SchemaFieldType;            // Tipo do valor
  defaultValue?: SchemaFieldValue;  // Valor padrão (ausente = obrigatório)
  description?: string;             // Descrição opcional
}

/** Valor possível de um campo do schema */
export type SchemaFieldValue = boolean | number | string | SchemaFieldValue[];

/**
 * Verifica se um campo do schema é obrigatório.
 * Um campo é obrigatório se NÃO possui defaultValue definido.
 */
export function isFieldRequired(field: PluginSchemaField): boolean {
  return field.defaultValue === undefined;
}

// ─── Dependências ───────────────────────────────────────────────────────────

/** Dependência de um plugin Python (nome + versão) */
export interface PluginDependency {
  name: string;
  version: string;  // Ex: ">=1.0.0", "==2.3.1", "^3.0"
}

// ─── Plugin Definition ──────────────────────────────────────────────────────

/**
 * Definição completa de um plugin (reutilizável entre plantas).
 */
export interface PluginDefinition {
  id: string;
  name: string;
  kind: PluginKind;
  runtime: PluginRuntime;
  sourceFile: string;        // main.py ou plugin.dll/.so
  sourceCode?: string;       // Código fonte inline (editor de código)
  schema: PluginSchemaField[];
  dependencies?: PluginDependency[];  // Dependências Python (pip)
  description?: string;
  version?: string;
  author?: string;
}

// ─── Plugin Instance ────────────────────────────────────────────────────────

/**
 * Instância de um plugin configurada para uma planta específica.
 * Contém os valores de configuração concretos.
 */
export interface PluginInstance {
  pluginId: string;          // ID do plugin base
  pluginName: string;        // Nome do plugin (para exibição)
  pluginKind: PluginKind;
  config: Record<string, SchemaFieldValue>;  // Valores dos campos
}

// ─── Formato JSON do Plugin (para import/export) ────────────────────────────

/**
 * Formato JSON esperado ao importar um plugin de arquivo.
 * Deve seguir o contrato abaixo para ser considerado válido.
 */
export interface PluginFileJSON {
  name: string;
  kind: PluginKind;
  runtime: PluginRuntime;
  sourceFile: string;
  schema: PluginSchemaField[];
  dependencies?: PluginDependency[];
  description?: string;
  version?: string;
  author?: string;
}

// ─── Validação ──────────────────────────────────────────────────────────────

// ─── Template de Driver Python ──────────────────────────────────────────────

/** Métodos obrigatórios que todo driver Python deve implementar */
export const DRIVER_REQUIRED_METHODS = [
  '__init__',
  'connect',
  'reconnect',
  'stop',
  'read',
  'send',
] as const;

/**
 * Converte o nome do plugin em um nome de classe Python válido.
 * Ex: "Modbus TCP Driver" → "ModbusTcpDriver"
 */
export function toDriverClassName(pluginName: string): string {
  if (!pluginName.trim()) return 'MyDriver';
  return pluginName
    .replace(/[^a-zA-Z0-9\s_]/g, '')
    .split(/[\s_]+/)
    .filter(Boolean)
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1).toLowerCase())
    .join('');
}

/**
 * Gera o template padrão de código Python para um driver.
 * Essa interface é obrigatória — o backend valida a presença da classe e métodos.
 */
export function generateDriverTemplate(pluginName: string): string {
  const className = toDriverClassName(pluginName);
  return `from senamby import MCUDriver


class ${className}(MCUDriver):
    """Driver: ${pluginName || 'Novo Driver'}"""

    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        # Inicialização do driver
        pass

    def connect(self):
        """Estabelece conexão com o dispositivo."""
        pass

    def reconnect(self):
        """Reconecta ao dispositivo após perda de conexão."""
        pass

    def stop(self):
        """Encerra a conexão e libera recursos."""
        pass

    def read(self):
        """Lê dados do dispositivo. Retorna os valores dos sensores."""
        pass

    def send(self, *outs):
        """Envia comandos/valores para os atuadores do dispositivo."""
        pass
`;
}

// ─── Validação ──────────────────────────────────────────────────────────────

/** Regex para nomes de campo: apenas letras, números e underscore */
const FIELD_NAME_REGEX = /^[a-zA-Z_][a-zA-Z0-9_]*$/;

/**
 * Valida se um nome de campo é válido (sem caracteres especiais)
 */
export function isValidFieldName(name: string): boolean {
  return FIELD_NAME_REGEX.test(name);
}

/**
 * Retorna o valor padrão para um tipo de campo
 */
export function getDefaultValueForType(type: SchemaFieldType): SchemaFieldValue {
  switch (type) {
    case 'bool': return false;
    case 'int': return 0;
    case 'float': return 0.0;
    case 'string': return '';
    case 'list': return [];
  }
}

/**
 * Valida se um objeto JSON é um plugin válido.
 * Retorna null se válido, mensagem de erro se inválido.
 */
export function validatePluginJSON(obj: unknown): string | null {
  if (!obj || typeof obj !== 'object') {
    return 'Arquivo inválido: não é um objeto JSON';
  }

  const json = obj as Record<string, unknown>;

  if (typeof json.name !== 'string' || !json.name.trim()) {
    return 'Campo "name" é obrigatório e deve ser uma string';
  }

  if (json.kind !== 'driver' && json.kind !== 'controller') {
    return 'Campo "kind" deve ser "driver" ou "controller"';
  }

  if (json.runtime !== 'python' && json.runtime !== 'rust-native') {
    return 'Campo "runtime" deve ser "python" ou "rust-native"';
  }

  if (typeof json.sourceFile !== 'string' || !json.sourceFile.trim()) {
    return 'Campo "sourceFile" é obrigatório';
  }

  if (!Array.isArray(json.schema)) {
    return 'Campo "schema" deve ser um array';
  }

  for (let i = 0; i < json.schema.length; i++) {
    const field = json.schema[i] as Record<string, unknown>;
    if (typeof field?.name !== 'string') return `schema[${i}].name deve ser string`;
    if (!isValidFieldName(field.name as string)) return `schema[${i}].name contém caracteres inválidos`;
    const validTypes: string[] = ['bool', 'int', 'float', 'string', 'list'];
    if (!validTypes.includes(field.type as string)) return `schema[${i}].type inválido`;
  }

  return null;
}
