import { invoke } from '@tauri-apps/api/core';
import type { Controller, ControllerParam, ControllerType } from '$lib/types/controller';
import type { BuiltInPluginKind, PluginDefinition, PluginKind } from '$lib/types/plugin';
import { getDefaultValueForType, isBuiltInPluginKind, normalizePluginKind, validatePluginJSON } from '$lib/types/plugin';
import type {
  CreatePluginRequest,
  CreatePluginResponse,
  PluginRegistryDto,
} from './types';
import { generateId } from '$lib/utils/format';

const STORAGE_KEY = 'senamby.desktop.plugins.workspace';

type PluginWorkspaceState = {
  localPlugins: PluginDefinition[];
  deletedPluginIds: string[];
};

const DEFAULT_WORKSPACE_STATE: PluginWorkspaceState = {
  localPlugins: [],
  deletedPluginIds: [],
};

function canUseStorage(): boolean {
  return typeof window !== 'undefined' && typeof localStorage !== 'undefined';
}

function loadWorkspaceState(): PluginWorkspaceState {
  if (!canUseStorage()) return { ...DEFAULT_WORKSPACE_STATE };

  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return { ...DEFAULT_WORKSPACE_STATE };
    const parsed = JSON.parse(raw) as PluginWorkspaceState;
    return {
      localPlugins: Array.isArray(parsed.localPlugins) ? parsed.localPlugins : [],
      deletedPluginIds: Array.isArray(parsed.deletedPluginIds) ? parsed.deletedPluginIds : [],
    };
  } catch (error) {
    console.error('Erro ao carregar estado local de plugins:', error);
    return { ...DEFAULT_WORKSPACE_STATE };
  }
}

function saveWorkspaceState(state: PluginWorkspaceState): void {
  if (!canUseStorage()) return;
  localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
}

function normalizePlugin(plugin: PluginDefinition): PluginDefinition {
  return {
    ...plugin,
    id: plugin.id || generateId(),
    kind: normalizePluginKind(plugin.kind),
    sourceFile: plugin.sourceFile || (plugin.runtime === 'python' ? 'main.py' : 'plugin.rs'),
    schema: plugin.schema ?? [],
    dependencies: plugin.dependencies ?? [],
    source: plugin.source ?? 'workspace',
  };
}

function upsertWorkspacePlugin(plugin: PluginDefinition): PluginDefinition {
  const normalized = normalizePlugin({ ...plugin, source: plugin.source ?? 'workspace' });
  const state = loadWorkspaceState();
  const index = state.localPlugins.findIndex((entry) => entry.id === normalized.id);

  if (index >= 0) {
    state.localPlugins[index] = normalized;
  } else {
    state.localPlugins.unshift(normalized);
  }

  state.deletedPluginIds = state.deletedPluginIds.filter((id) => id !== normalized.id);
  saveWorkspaceState(state);
  return normalized;
}

function removeWorkspacePlugin(pluginId: string): void {
  const state = loadWorkspaceState();
  state.localPlugins = state.localPlugins.filter((plugin) => plugin.id !== pluginId);
  saveWorkspaceState(state);
}

function markPluginDeleted(pluginId: string): void {
  const state = loadWorkspaceState();
  if (!state.deletedPluginIds.includes(pluginId)) {
    state.deletedPluginIds.push(pluginId);
  }
  saveWorkspaceState(state);
}

function mapDtoToPlugin(dto: PluginRegistryDto): PluginDefinition {
  return {
    id: dto.id,
    name: dto.name,
    kind: dto.type,
    runtime: dto.runtime,
    sourceFile: dto.source_file ?? (dto.runtime === 'python' ? 'main.py' : 'plugin.rs'),
    sourceCode: dto.source_code ?? undefined,
    schema: (dto.schema ?? []).map((field) => ({
      name: field.name,
      type: field.type as PluginDefinition['schema'][number]['type'],
      defaultValue: field.default_value as PluginDefinition['schema'][number]['defaultValue'],
      description: field.description ?? undefined,
    })),
    dependencies: (dto.dependencies ?? []).map((dependency) => ({
      name: dependency.name,
      version: dependency.version,
    })),
    description: dto.description ?? undefined,
    version: dto.version ?? undefined,
    author: dto.author ?? undefined,
    source: 'backend',
  };
}

async function listBackendPlugins(): Promise<PluginDefinition[]> {
  try {
    const response = await invoke<PluginRegistryDto[]>('list_plugins');
    return response.map(mapDtoToPlugin);
  } catch (error) {
    console.warn('Backend de plugins indisponível, usando somente catálogo local:', error);
    return [];
  }
}

async function listBackendPluginsByType(kind: BuiltInPluginKind): Promise<PluginDefinition[]> {
  try {
    const response = await invoke<PluginRegistryDto[]>('list_plugins_by_type', { pluginType: kind });
    return response.map(mapDtoToPlugin);
  } catch (error) {
    console.warn(`Backend de plugins indisponível para o tipo "${kind}", usando catálogo local:`, error);
    return [];
  }
}

function mergePlugins(backendPlugins: PluginDefinition[], workspacePlugins: PluginDefinition[], deletedIds: string[]): PluginDefinition[] {
  const registry = new Map<string, PluginDefinition>();

  for (const plugin of backendPlugins) {
    if (!deletedIds.includes(plugin.id)) {
      registry.set(plugin.id, normalizePlugin(plugin));
    }
  }

  for (const plugin of workspacePlugins) {
    registry.set(plugin.id, normalizePlugin(plugin));
  }

  return Array.from(registry.values()).sort((left, right) => left.name.localeCompare(right.name, 'pt-BR'));
}

function inferControllerType(plugin: PluginDefinition): ControllerType {
  const normalizedName = plugin.name.trim().toUpperCase();

  if (normalizedName.includes('PID')) return 'PID';
  if (normalizedName.includes('FLOW')) return 'Flow';
  if (normalizedName.includes('LEVEL')) return 'Level';

  return (plugin.name.trim() || 'PID') as ControllerType;
}

function mapSchemaFieldToControllerParam(field: PluginDefinition['schema'][number]): ControllerParam {
  const defaultValue = field.defaultValue ?? getDefaultValueForType(field.type);

  if (field.type === 'bool') {
    return {
      type: 'boolean',
      value: typeof defaultValue === 'boolean' ? defaultValue : false,
      label: field.description?.trim() || field.name,
    };
  }

  if (field.type === 'int' || field.type === 'float') {
    return {
      type: 'number',
      value: typeof defaultValue === 'number' ? defaultValue : 0,
      label: field.description?.trim() || field.name,
    };
  }

  return {
    type: 'string',
    value: Array.isArray(defaultValue) ? defaultValue.join(', ') : String(defaultValue ?? ''),
    label: field.description?.trim() || field.name,
  };
}

function toControllerTemplate(plugin: PluginDefinition): Controller {
  return {
    id: plugin.id,
    name: plugin.name,
    type: inferControllerType(plugin),
    active: false,
    params: Object.fromEntries(
      plugin.schema.map((field) => [field.name, mapSchemaFieldToControllerParam(field)])
    ),
  };
}

export async function createPlugin(request: CreatePluginRequest): Promise<CreatePluginResponse> {
  if (!request.name.trim()) {
    return { success: false, error: 'Nome do plugin é obrigatório' };
  }

  const kind = normalizePluginKind(request.kind);

  if (isBuiltInPluginKind(kind)) {
    try {
      const response = await invoke<PluginRegistryDto>('create_plugin', {
        request: {
          name: request.name.trim(),
          type: kind,
          runtime: request.runtime,
          schema: request.schema.map((field) => ({
            name: field.name,
            type: field.type,
            default_value: field.defaultValue,
            description: field.description ?? null,
          })),
          source_file: request.sourceFile ?? null,
          source_code: request.sourceCode ?? null,
          dependencies: (request.dependencies ?? []).map((dependency) => ({
            name: dependency.name,
            version: dependency.version,
          })),
          description: request.description ?? null,
          version: request.version ?? null,
          author: request.author ?? null,
        },
      });

      return { success: true, plugin: mapDtoToPlugin(response) };
    } catch (error) {
      console.warn('Falha ao criar plugin no backend, salvando localmente:', error);
    }
  }

  const plugin = upsertWorkspacePlugin({
    id: generateId(),
    name: request.name.trim(),
    kind,
    runtime: request.runtime,
    sourceFile: request.sourceFile ?? (request.runtime === 'python' ? 'main.py' : 'plugin.rs'),
    sourceCode: request.sourceCode,
    schema: request.schema,
    dependencies: request.dependencies ?? [],
    description: request.description,
    version: request.version,
    author: request.author,
    source: 'workspace',
  });

  return { success: true, plugin };
}

export async function listPlugins(): Promise<PluginDefinition[]> {
  const state = loadWorkspaceState();
  const backendPlugins = await listBackendPlugins();
  return mergePlugins(backendPlugins, state.localPlugins, state.deletedPluginIds);
}

export async function listPluginsByType(kind: PluginKind): Promise<PluginDefinition[]> {
  const normalizedKind = normalizePluginKind(kind);
  const state = loadWorkspaceState();
  const workspacePlugins = state.localPlugins.filter(
    (plugin) => normalizePluginKind(plugin.kind) === normalizedKind
  );

  if (!isBuiltInPluginKind(normalizedKind)) {
    return mergePlugins([], workspacePlugins, state.deletedPluginIds).filter(
      (plugin) => normalizePluginKind(plugin.kind) === normalizedKind
    );
  }

  const backendPlugins = await listBackendPluginsByType(normalizedKind);
  return mergePlugins(backendPlugins, workspacePlugins, state.deletedPluginIds).filter(
    (plugin) => normalizePluginKind(plugin.kind) === normalizedKind
  );
}

export async function listControllerTemplates(): Promise<Controller[]> {
  const controllerPlugins = await listPluginsByType('controller');
  return controllerPlugins.map(toControllerTemplate);
}

export async function validatePluginFile(json: unknown): Promise<{ success: boolean; plugin?: PluginDefinition; error?: string }> {
  const validationError = validatePluginJSON(json);

  if (validationError) {
    return { success: false, error: validationError };
  }

  const parsed = json as Record<string, unknown>;
  const plugin = normalizePlugin({
    id: typeof parsed.id === 'string' ? parsed.id : generateId(),
    name: parsed.name as string,
    kind: normalizePluginKind(parsed.kind as string),
    runtime: parsed.runtime as PluginDefinition['runtime'],
    sourceFile: parsed.sourceFile as string,
    sourceCode: typeof parsed.sourceCode === 'string' ? parsed.sourceCode : undefined,
    schema: Array.isArray(parsed.schema) ? (parsed.schema as PluginDefinition['schema']) : [],
    dependencies: Array.isArray(parsed.dependencies) ? (parsed.dependencies as PluginDefinition['dependencies']) ?? [] : [],
    description: typeof parsed.description === 'string' ? parsed.description : undefined,
    version: typeof parsed.version === 'string' ? parsed.version : undefined,
    author: typeof parsed.author === 'string' ? parsed.author : undefined,
    source: 'workspace',
  });

  return { success: true, plugin };
}

export async function registerPlugin(plugin: PluginDefinition): Promise<{ success: boolean; plugin?: PluginDefinition; error?: string }> {
  return createPlugin({
    name: plugin.name,
    kind: plugin.kind,
    runtime: plugin.runtime,
    schema: plugin.schema,
    sourceFile: plugin.sourceFile,
    sourceCode: plugin.sourceCode,
    dependencies: plugin.dependencies,
    description: plugin.description,
    version: plugin.version,
    author: plugin.author,
  });
}

export async function updatePlugin(plugin: PluginDefinition): Promise<{ success: boolean; plugin?: PluginDefinition; error?: string }> {
  try {
    const updated = upsertWorkspacePlugin(plugin);
    return { success: true, plugin: updated };
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : 'Erro ao atualizar plugin';
    return { success: false, error: errorMessage };
  }
}

export async function deletePlugin(pluginId: string): Promise<{ success: boolean; error?: string }> {
  try {
    const plugins = await listPlugins();
    const target = plugins.find((plugin) => plugin.id === pluginId);

    if (!target) {
      return { success: false, error: 'Plugin não encontrado' };
    }

    if (target.source === 'backend') {
      markPluginDeleted(pluginId);
    } else {
      removeWorkspacePlugin(pluginId);
    }

    return { success: true };
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : 'Erro ao excluir plugin';
    return { success: false, error: errorMessage };
  }
}
