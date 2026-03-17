use core::error;
use std::fs::{self, remove_dir_all};
use std::path::PathBuf;

use crate::core::error::{AppError, AppResult};
use crate::core::models::plant::Plant;
use crate::core::models::plugin::{PluginRegistry, PluginType};
use crate::core::services::workspace;

const APP_WORKSPACE_DIR: &str = "Senamby/workspace";
const DRIVERS_DIR: &str = "drivers";
const CONTROLLERS_DIR: &str = "controllers";
const PLANTS_DIR: &str = "plants";
const REGISTRY_FILE: &str = "registry.json";
const SOURCE_FILE: &str = "main.py";

pub struct WorkspaceService;

impl WorkspaceService {
    pub fn save_plugin_registry(plugin: &PluginRegistry, source_code: &str) -> AppResult<()> {
        let plugin_dir = Self::plugin_directory(&plugin.name, plugin.plugin_type)?;
        fs::create_dir_all(&plugin_dir).map_err(|error| {
            AppError::IoError(format!(
                "Falha ao criar diretório do {:?} '{1}': {2}",
                plugin.plugin_type, plugin.name, error
            ))
        })?;

        let source_path = plugin_dir.join(SOURCE_FILE);
        fs::write(&source_path, source_code).map_err(|error| {
            AppError::IoError(format!(
                "Falha ao salvar código fonte do {:?} '{}' em '{}': {}",
                plugin.plugin_type,
                plugin.name,
                source_path.display(),
                error
            ))
        })?;

        let registry_payload = serde_json::to_string_pretty(plugin).map_err(|error| {
            AppError::IoError(format!(
                "Falha ao serializar registro do {:?} '{1}': {2}",
                plugin.plugin_type, plugin.name, error
            ))
        })?;

        let registry_path = plugin_dir.join(REGISTRY_FILE);
        fs::write(&registry_path, registry_payload).map_err(|error| {
            AppError::IoError(format!(
                "Falha ao salvar registro do {:?} '{}' em '{}': {}",
                plugin.plugin_type,
                plugin.name,
                registry_path.display(),
                error
            ))
        })?;

        Ok(())
    }

    pub fn update_plugin_registry(
        plugin: &PluginRegistry,
        source_code: &str,
        previous_plugin_name: &str,
        previous_plugin_type: PluginType,
    ) -> AppResult<()> {
        let previous_dir = Self::plugin_directory(previous_plugin_name, previous_plugin_type)?;
        let next_dir = Self::plugin_directory(&plugin.name, plugin.plugin_type)?;

        if previous_dir != next_dir && previous_dir.exists() {
            remove_dir_all(&previous_dir).map_err(|error| {
                AppError::IoError(format!(
                    "Falha ao remover diretório antigo do plugin '{}': {}",
                    previous_dir.display(),
                    error
                ))
            })?;
        }

        Self::save_plugin_registry(plugin, source_code)?;

        Ok(())
    }

    pub fn read_plugin_source(plugin_name: &str, plugin_type: PluginType) -> AppResult<String> {
        let source_path = Self::plugin_source_path(plugin_name, plugin_type)?;

        fs::read_to_string(&source_path).map_err(|error| {
            AppError::IoError(format!(
                "Falha ao ler código fonte do plugin em '{}': {}",
                source_path.display(),
                error
            ))
        })
    }

    pub fn save_plant_registry(plant: &Plant) -> AppResult<()> {
        let workspace_root = Self::workspace_root()?;
        let plant_dir = workspace_root.join(PLANTS_DIR).join(plant.name.trim());

        fs::create_dir_all(&plant_dir).map_err(|error| {
            AppError::IoError(format!(
                "Falha ao criar diretório da planta '{}': {}",
                plant.name, error
            ))
        })?;

        let registry_path = plant_dir.join(REGISTRY_FILE);
        let registry_payload = serde_json::to_string_pretty(plant).map_err(|error| {
            AppError::IoError(format!(
                "Falha ao serializar registro da planta {}: {}",
                plant.name, error
            ))
        })?;
        fs::write(&registry_path, registry_payload).map_err(|error| {
            AppError::IoError(format!(
                "Falha ao salvar o registro da planta {}: {}",
                plant.name, error
            ))
        })?;

        Ok(())
    }

    fn plugin_directory(plugin_name: &str, plugin_type: PluginType) -> AppResult<PathBuf> {
        let workspace_root = Self::workspace_root()?;

        match plugin_type {
            PluginType::Driver => Ok(workspace_root.join(DRIVERS_DIR).join(plugin_name)),
            PluginType::Controller => Ok(workspace_root.join(CONTROLLERS_DIR).join(plugin_name)),
        }
    }

    fn plugin_source_path(plugin_name: &str, plugin_type: PluginType) -> AppResult<PathBuf> {
        Ok(Self::plugin_directory(plugin_name, plugin_type)?.join(SOURCE_FILE))
    }

    #[cfg(not(test))]
    fn workspace_root() -> AppResult<PathBuf> {
        let documents_dir = dirs::document_dir().ok_or_else(|| {
            AppError::IoError("Não foi possível localizar o diretório Documentos".into())
        })?;

        Ok(documents_dir.join(APP_WORKSPACE_DIR))
    }

    #[cfg(test)]
    fn workspace_root() -> AppResult<PathBuf> {
        Ok(std::env::temp_dir().join(APP_WORKSPACE_DIR))
    }
}
