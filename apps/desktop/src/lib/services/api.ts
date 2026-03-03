/**
 * ============================================================================
 * API — Camada Unificada de Backend
 * ============================================================================
 *
 * Ponto único de acesso para todas as operações de backend.
 * Cada módulo de serviço (plantBackend, pluginBackend, analyzerBackend, etc.)
 * já funciona como mock — este arquivo centraliza re-exports e define o
 * contrato de integração.
 *
 * INTEGRAÇÃO COM BACKEND REAL (Tauri IPC):
 * ─────────────────────────────────────────
 * Quando o backend Rust estiver pronto, mude as implementações nos
 * arquivos individuais de *Backend.ts para usar `invoke()`:
 *
 *   import { invoke } from '@tauri-apps/api/core';
 *
 *   export async function createPlant(req) {
 *     return invoke('create_plant', { request: req });
 *   }
 *
 * Os componentes importam de api.ts e não precisam mudar.
 *
 * ADICIONANDO NOVOS MÓDULOS:
 * ─────────────────────────────────────────
 * 1. Crie o serviço mock em services/<modulo>Backend.ts
 * 2. Exporte as funções e tipos aqui
 * 3. Componentes importam de '$lib/services/api'
 */

// ─── Plant ──────────────────────────────────────────────────────────────────

export {
  createPlant,
  openPlant,
  saveDriver,
  type CreatePlantRequest,
  type CreatePlantResponse,
  type OpenPlantRequest,
  type OpenPlantResponse,
  type SaveDriverRequest,
  type SaveDriverResponse,
} from './plantBackend';

// ─── Plugin ─────────────────────────────────────────────────────────────────

export {
  listPlugins,
  getPlugin,
  validatePluginFile,
  registerPlugin,
  validatePluginInstanceConfig,
  validateDriverSourceCode,
  type ValidatePluginResponse,
  type RegisterPluginResponse,
  type ValidateDriverCodeResponse,
} from './pluginBackend';

// ─── Analyzer ───────────────────────────────────────────────────────────────

export {
  processJSONFile,
} from './analyzerBackend';

// ─── File Dialog ────────────────────────────────────────────────────────────

export {
  openFileDialog,
  openFilesDialog,
  readFileAsText,
  FILE_FILTERS,
  type FileFilter,
  type OpenFileOptions,
  type FileResult,
} from './fileDialog';

// ─── Export ─────────────────────────────────────────────────────────────────

export {
  exportPlantDataCSV,
  exportPlantDataJSON,
} from './export';

// ─── Simulation (mock data source — will be replaced by real MCU stream) ──

export {
  startSimulation,
} from './simulation';
