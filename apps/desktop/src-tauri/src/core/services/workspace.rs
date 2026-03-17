use std::fs;
use std::path::PathBuf;

use crate::core::error::{AppError, AppResult};
use crate::core::models::plugin::PluginRegistry;

const APP_WORKSPACE_DIR: &str = "Senamby/workspace";
const DRIVERS_DIR: &str = "drivers";
const DRIVER_REGISTRY_FILE: &str = "registry.json";
const DRIVER_SOURCE_FILE: &str = "main.py";

pub struct WorkspaceService;

impl WorkspaceService {
    pub fn save_driver_registry(plugin: &PluginRegistry, source_code: &str) -> AppResult<()> {
        let driver_dir = Self::driver_directory(&plugin.name)?;
        fs::create_dir_all(&driver_dir).map_err(|error| {
            AppError::IoError(format!(
                "Falha ao criar diretório do driver '{}': {}",
                plugin.name, error
            ))
        })?;

        let source_path = driver_dir.join(DRIVER_SOURCE_FILE);
        fs::write(&source_path, source_code).map_err(|error| {
            AppError::IoError(format!(
                "Falha ao salvar código fonte do driver em '{}': {}",
                source_path.display(),
                error
            ))
        })?;

        let registry_payload = serde_json::to_string_pretty(plugin).map_err(|error| {
            AppError::IoError(format!(
                "Falha ao serializar registro do driver '{}': {}",
                plugin.name, error
            ))
        })?;

        let registry_path = driver_dir.join(DRIVER_REGISTRY_FILE);
        fs::write(&registry_path, registry_payload).map_err(|error| {
            AppError::IoError(format!(
                "Falha ao salvar registro do driver em '{}': {}",
                registry_path.display(),
                error
            ))
        })?;

        Ok(())
    }

    fn driver_directory(plugin_name: &str) -> AppResult<PathBuf> {
        let workspace_root = Self::workspace_root()?;
        Ok(workspace_root.join(DRIVERS_DIR).join(plugin_name))
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
