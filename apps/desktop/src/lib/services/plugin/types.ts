import type { PluginDefinition, PluginKind, PluginRuntime, PluginSchemaField, PluginDependency } from '$lib/types/plugin';

export interface CreatePluginDto {
  name: string;
  type: PluginKind;
  runtime: PluginRuntime;
  schema: PluginSchemaFieldDto[];
  source_file?: string | null;
  source_code?: string | null;
  dependencies: PluginDependencyDto[];
  description?: string | null;
  version?: string | null;
  author?: string | null;
}

export interface UpdatePluginDto extends CreatePluginDto {
  id: string;
}

export interface PluginSchemaFieldDto {
  name: string;
  type: string;
  default_value?: unknown;
  description?: string | null;
}

export interface PluginDependencyDto {
  name: string;
  version: string;
}

export interface PluginRegistryDto {
  id: string;
  name: string;
  type: PluginKind;
  runtime: PluginRuntime;
  schema: PluginSchemaFieldDto[];
  source_file?: string | null;
  source_code?: string | null;
  dependencies: PluginDependencyDto[];
  description?: string | null;
  version?: string | null;
  author?: string | null;
}

export interface CreatePluginRequest {
  name: string;
  kind: PluginKind;
  runtime: PluginRuntime;
  schema: PluginSchemaField[];
  sourceFile?: string;
  sourceCode?: string;
  dependencies?: PluginDependency[];
  description?: string;
  version?: string;
  author?: string;
}

export interface UpdatePluginRequest extends CreatePluginRequest {
  id: string;
}

export interface CreatePluginResponse {
  success: boolean;
  plugin?: PluginDefinition;
  error?: string;
}

export interface GetPluginRequest {
  id: string;
}
