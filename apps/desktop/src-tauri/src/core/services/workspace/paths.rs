#[cfg(not(test))]
use crate::core::error::AppError;
use crate::core::error::AppResult;
use crate::core::models::plugin::PluginType;
use std::path::PathBuf;

const APP_WORKSPACE_DIR: &str = "Senamby/workspace";
const DRIVERS_DIR: &str = "drivers";
const CONTROLLERS_DIR: &str = "controllers";
const PLANTS_DIR: &str = "plants";
const ENVS_DIR: &str = "envs";
const RUNTIMES_DIR: &str = "runtimes";
pub(super) const REGISTRY_FILE: &str = "registry.json";
pub(super) const SOURCE_FILE: &str = "main.py";

pub(super) fn plugin_directory(plugin_name: &str, plugin_type: PluginType) -> AppResult<PathBuf> {
    let plugin_root = plugin_root_directory(plugin_type)?;
    let plugin_name =
        crate::core::services::workspace::ensure_safe_workspace_component(plugin_name, "plugin")?;

    Ok(plugin_root.join(plugin_name))
}

pub(super) fn plugin_root_directory(plugin_type: PluginType) -> AppResult<PathBuf> {
    let workspace_root = workspace_root()?;
    let parent_dir = match plugin_type {
        PluginType::Driver => DRIVERS_DIR,
        PluginType::Controller => CONTROLLERS_DIR,
    };

    Ok(workspace_root.join(parent_dir))
}

pub(super) fn plugin_source_path(plugin_name: &str, plugin_type: PluginType) -> AppResult<PathBuf> {
    Ok(plugin_directory(plugin_name, plugin_type)?.join(SOURCE_FILE))
}

pub(super) fn plant_directory(plant_name: &str) -> AppResult<PathBuf> {
    let workspace_root = workspace_root()?;
    let plant_name =
        crate::core::services::workspace::ensure_safe_workspace_component(plant_name, "planta")?;
    Ok(workspace_root.join(PLANTS_DIR).join(plant_name))
}

pub(super) fn env_directory(env_hash: &str) -> AppResult<PathBuf> {
    let workspace_root = workspace_root()?;
    let env_hash =
        crate::core::services::workspace::ensure_safe_workspace_component(env_hash, "ambiente")?;
    Ok(workspace_root.join(ENVS_DIR).join(env_hash))
}

pub(super) fn runtime_root_directory() -> AppResult<PathBuf> {
    Ok(workspace_root()?.join(RUNTIMES_DIR))
}

pub(super) fn runtime_directory(runtime_id: &str) -> AppResult<PathBuf> {
    let runtime_id =
        crate::core::services::workspace::ensure_safe_workspace_component(runtime_id, "runtime")?;
    Ok(runtime_root_directory()?.join(runtime_id))
}

#[cfg(not(test))]
pub(super) fn workspace_root() -> AppResult<PathBuf> {
    let documents_dir = dirs::document_dir().ok_or_else(|| {
        AppError::IoError("Não foi possível localizar o diretório Documentos".into())
    })?;

    Ok(documents_dir.join(APP_WORKSPACE_DIR))
}

#[cfg(test)]
pub(super) fn workspace_root() -> AppResult<PathBuf> {
    Ok(std::env::temp_dir().join(APP_WORKSPACE_DIR))
}
