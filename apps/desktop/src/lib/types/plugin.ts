export type BuiltInPluginKind = 'driver' | 'controller';
export type PluginKind = BuiltInPluginKind | (string & {});

export type PluginRuntime = 'python' | 'rust-native';

export type SchemaFieldType = 'bool' | 'int' | 'float' | 'string' | 'list';

export const BUILTIN_PLUGIN_KINDS: BuiltInPluginKind[] = ['driver', 'controller'];

export const PLUGIN_KIND_LABELS: Record<BuiltInPluginKind, string> = {
  driver: 'Driver',
  controller: 'Controlador',
};

export function isBuiltInPluginKind(kind: string): kind is BuiltInPluginKind {
  return BUILTIN_PLUGIN_KINDS.includes(kind as BuiltInPluginKind);
}

export function getPluginKindLabel(kind: PluginKind): string {
  if (isBuiltInPluginKind(kind)) {
    return PLUGIN_KIND_LABELS[kind];
  }

  return kind
    .split(/[_-\s]+/)
    .filter(Boolean)
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ');
}

export function normalizePluginKind(kind: string): PluginKind {
  return kind
    .trim()
    .toLowerCase()
    .replace(/\s+/g, '_')
    .replace(/[^a-z0-9_-]/g, '') as PluginKind;
}

export const PLUGIN_RUNTIME_LABELS: Record<PluginRuntime, string> = {
  python: 'Python',
  'rust-native': 'Rust Nativo',
};

export const PLUGIN_CREATION_RUNTIMES: readonly PluginRuntime[] = ['python'];

export const SCHEMA_FIELD_TYPE_LABELS: Record<SchemaFieldType, string> = {
  bool: 'Boolean',
  int: 'Inteiro',
  float: 'Decimal',
  string: 'Texto',
  list: 'Lista',
};

export const AUTO_SCHEMA_FIELDS: PluginSchemaField[] = [
  { name: 'num_sensors', type: 'int', description: 'Quantidade de sensores usada no driver' },
  { name: 'num_actuators', type: 'int', description: 'Quantidade de atuadores usada no driver' },
];

export const RESERVED_FIELD_NAMES = AUTO_SCHEMA_FIELDS.map(f => f.name);

export function isAutoSchemaField(fieldName: string): boolean {
  return RESERVED_FIELD_NAMES.includes(fieldName);
}

export interface PluginSchemaField {
  name: string;
  type: SchemaFieldType;
  defaultValue?: SchemaFieldValue;
  description?: string;
}

export type SchemaFieldValue = boolean | number | string | SchemaFieldValue[];

export function isFieldRequired(field: PluginSchemaField): boolean {
  return field.defaultValue === undefined;
}

export function pluginNeedsInitialConfig(plugin: Pick<PluginDefinition, 'schema'>): boolean {
  return plugin.schema.some(isFieldRequired);
}

export interface PluginDependency {
  name: string;
  version: string;
}

export interface PluginDefinition {
  id: string;
  name: string;
  kind: PluginKind;
  runtime: PluginRuntime;
  entryClass: string;
  sourceFile: string;
  sourceCode?: string;
  schema: PluginSchemaField[];
  dependencies?: PluginDependency[];
  description?: string;
  version?: string;
  author?: string;
  source?: 'backend' | 'workspace';
}

export interface PluginInstance {
  pluginId: string;
  pluginName: string;
  pluginKind: PluginKind;
  config: Record<string, SchemaFieldValue>;
}

export interface PluginFileJSON {
  name: string;
  kind: PluginKind;
  runtime: PluginRuntime;
  entryClass?: string;
  sourceFile: string;
  schema: PluginSchemaField[];
  dependencies?: PluginDependency[];
  description?: string;
  version?: string;
  author?: string;
}

export const DRIVER_REQUIRED_METHODS = [
  'connect',
  'stop',
  'read',
] as const;

export const CONTROLLER_REQUIRED_METHODS = [
  'compute',
] as const;

export function toPluginEntryClassName(pluginName: string, kind: PluginKind): string {
  const fallback = normalizePluginKind(kind) === 'controller' ? 'MyController' : 'MyDriver';
  if (!pluginName.trim()) return fallback;

  const className = pluginName
    .replace(/[^a-zA-Z0-9\s_]/g, '')
    .split(/[\s_]+/)
    .filter(Boolean)
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1).toLowerCase())
    .join('');

  return className || fallback;
}

export function toDriverClassName(pluginName: string): string {
  return toPluginEntryClassName(pluginName, 'driver');
}

export function toControllerClassName(pluginName: string): string {
  return toPluginEntryClassName(pluginName, 'controller');
}

export function generateDriverTemplate(pluginName: string, entryClass?: string): string {
  const className = (entryClass?.trim() || toDriverClassName(pluginName));
  return `from typing import Any, Dict

class ${className}:
    """Driver: ${pluginName || 'Novo Driver'}"""

    def __init__(self, context: Any) -> None:
        # Contrato atual:
        # - context.config -> configuração do driver
        # - context.plant -> planta, sensores, atuadores e setpoints
        # - context.runtime -> timing, paths e supervisão
        self.context = context

    def connect(self) -> bool:
        # Exemplos uteis:
        # port = self.context.config.get("port")
        # sensor_ids = self.context.plant.sensors.ids
        # sample_time_ms = self.context.runtime.timing.sample_time_ms
        return True

    def stop(self) -> bool:
        return True

    def read(self) -> Dict[str, Dict[str, float]]:
        # O contrato atual de leitura e explicito:
        # {
        #   "sensors": {"var_0": 0.0},
        #   "actuators": {"var_2": 0.0}
        # }
        return {
            "sensors": {
                sensor_id: 0.0
                for sensor_id in self.context.plant.sensors.ids
            },
            "actuators": {}
        }

    def write(self, outputs: Dict[str, float]) -> bool:
        # outputs contem o mapa final de saidas do ciclo:
        # {"var_2": 42.0}
        return True
`;
}

export function generateControllerTemplate(pluginName: string, entryClass?: string): string {
  const className = (entryClass?.trim() || toControllerClassName(pluginName));
  return `from typing import Any, Dict

class ${className}:
    """Controlador: ${pluginName || 'Novo Controlador'}"""

    def __init__(self, context: Any) -> None:
        # Contrato atual:
        # - context.controller -> metadados da instancia, bindings e parametros
        # - context.plant -> variaveis, sensores, atuadores e setpoints
        # - context.runtime -> timing, paths e supervisao
        self.context = context

    def compute(self, snapshot: Dict[str, Any]) -> Dict[str, float]:
        # snapshot contem os dados do ciclo atual:
        # - snapshot["dt_s"]
        # - snapshot["setpoints"]
        # - snapshot["sensors"]
        # - snapshot["actuators"]
        # - snapshot["controller"]
        #
        # Retorne apenas saidas por variable_id de atuador.
        outputs: Dict[str, float] = {}
        for actuator_id in self.context.controller.output_variable_ids:
            outputs[actuator_id] = 0.0
        return outputs
`;
}

const FIELD_NAME_REGEX = /^[a-zA-Z_][a-zA-Z0-9_]*$/;

export function isValidFieldName(name: string): boolean {
  return FIELD_NAME_REGEX.test(name);
}

export function getDefaultValueForType(type: SchemaFieldType): SchemaFieldValue {
  switch (type) {
    case 'bool': return false;
    case 'int': return 0;
    case 'float': return 0.0;
    case 'string': return '';
    case 'list': return [];
  }
}

export function validatePluginJSON(obj: unknown): string | null {
  if (!obj || typeof obj !== 'object') {
    return 'Arquivo inválido: não é um objeto JSON';
  }

  const json = obj as Record<string, unknown>;

  if (typeof json.name !== 'string' || !json.name.trim()) {
    return 'Campo "name" é obrigatório e deve ser uma string';
  }

  if (typeof json.kind !== 'string' || !normalizePluginKind(json.kind).trim()) {
    return 'Campo "kind" deve ser uma string não vazia';
  }

  if (json.runtime !== 'python' && json.runtime !== 'rust-native') {
    return 'Campo "runtime" deve ser "python" ou "rust-native"';
  }

  if (json.entryClass !== undefined && (typeof json.entryClass !== 'string' || !json.entryClass.trim())) {
    return 'Campo "entryClass" deve ser uma string não vazia';
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
