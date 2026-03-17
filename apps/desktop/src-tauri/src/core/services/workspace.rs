use std::fs;
use std::path::{Path, PathBuf};

use crate::core::error::{AppError, AppResult};
use crate::core::models::plugin::PluginRegistry;

const APP_WORKSPACE_DIR: &str = "Senamby/workspace";
const DRIVERS_DIR: &str = "drivers";
const DRIVER_REGISTRY_FILE: &str = "registry.json";
const DEFAULT_DRIVER_SOURCE_FILE: &str = "driver.py";

pub struct WorkspaceService;

impl WorkspaceService {
    pub fn save_driver_registry(plugin: &PluginRegistry) -> AppResult<()> {
        let driver_dir = Self::driver_directory(&plugin.name)?;
        fs::create_dir_all(&driver_dir).map_err(|error| {
            AppError::IoError(format!(
                "Falha ao criar diretório do driver '{}': {}",
                plugin.name, error
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

        if let Some(source_code) = plugin.source_code.as_deref() {
            let source_file_name = Self::sanitize_source_file_name(plugin.source_file.as_deref());
            let source_path = driver_dir.join(source_file_name);
            fs::write(&source_path, source_code).map_err(|error| {
                AppError::IoError(format!(
                    "Falha ao salvar código fonte do driver em '{}': {}",
                    source_path.display(),
                    error
                ))
            })?;
        }

        Ok(())
    }

    fn driver_directory(plugin_id: &str) -> AppResult<PathBuf> {
        let workspace_root = Self::workspace_root()?;
        Ok(workspace_root.join(DRIVERS_DIR).join(plugin_id))
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

    fn sanitize_source_file_name(source_file: Option<&str>) -> String {
        source_file
            .and_then(|file| {
                let trimmed = file.trim();
                if trimmed.is_empty() {
                    return None;
                }

                Path::new(trimmed)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.to_string())
            })
            .unwrap_or_else(|| DEFAULT_DRIVER_SOURCE_FILE.to_string())
    }
}
