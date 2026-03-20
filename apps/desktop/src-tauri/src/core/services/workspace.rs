use crate::core::error::{AppError, AppResult};
use crate::core::models::plant::Plant;
use crate::core::models::plugin::{PluginRegistry, PluginType};
use std::path::{Component, Path, PathBuf};

mod io;
mod paths;
mod plant_registry;
mod plugin_registry;

pub struct WorkspaceService;

impl WorkspaceService {
    pub fn save_plugin_registry(plugin: &PluginRegistry, source_code: &str) -> AppResult<()> {
        plugin_registry::save(plugin, source_code)
    }

    pub fn update_plugin_registry(
        plugin: &PluginRegistry,
        source_code: &str,
        previous_plugin_name: &str,
        previous_plugin_type: PluginType,
    ) -> AppResult<()> {
        plugin_registry::update(
            plugin,
            source_code,
            previous_plugin_name,
            previous_plugin_type,
        )
    }

    pub fn read_plugin_source(plugin_name: &str, plugin_type: PluginType) -> AppResult<String> {
        plugin_registry::read_source(plugin_name, plugin_type)
    }

    pub fn delete_plugin_registry(plugin_name: &str, plugin_type: PluginType) -> AppResult<()> {
        plugin_registry::delete(plugin_name, plugin_type)
    }

    pub fn save_plant_registry(plant: &Plant) -> AppResult<()> {
        plant_registry::save(plant)
    }

    pub fn update_plant_registry(plant: &Plant, previous_plant_name: &str) -> AppResult<()> {
        plant_registry::update(plant, previous_plant_name)
    }

    pub fn load_plugin_registries() -> AppResult<Vec<PluginRegistry>> {
        plugin_registry::load()
    }

    pub fn delete_plant_registry(plant_name: &str) -> AppResult<()> {
        plant_registry::delete(plant_name)
    }

    pub fn plugin_directory(plugin_name: &str, plugin_type: PluginType) -> AppResult<PathBuf> {
        paths::plugin_directory(plugin_name, plugin_type)
    }

    pub fn env_directory(env_hash: &str) -> AppResult<PathBuf> {
        paths::env_directory(env_hash)
    }

    pub fn runtime_directory(runtime_id: &str) -> AppResult<PathBuf> {
        paths::runtime_directory(runtime_id)
    }

    pub fn runtime_root_directory() -> AppResult<PathBuf> {
        paths::runtime_root_directory()
    }
}

pub(crate) fn map_serde_error(context: &str, error: serde_json::Error) -> AppError {
    AppError::IoError(format!("{context}: {error}"))
}

pub(crate) fn map_io_error(context: &str, error: std::io::Error) -> AppError {
    AppError::IoError(format!("{context}: {error}"))
}

pub(crate) fn normalize_entity_name(name: &str) -> &str {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        name
    } else {
        trimmed
    }
}

pub(crate) fn ensure_non_empty_name(name: &str, entity: &str) -> AppResult<String> {
    let normalized = normalize_entity_name(name);
    if normalized.is_empty() {
        return Err(AppError::InvalidArgument(format!(
            "Nome de {entity} não pode ser vazio"
        )));
    }

    Ok(normalized.to_string())
}

pub(crate) fn ensure_safe_workspace_component(name: &str, entity: &str) -> AppResult<String> {
    let normalized = ensure_non_empty_name(name, entity)?;
    let path = Path::new(&normalized);

    if path.is_absolute() {
        return Err(AppError::InvalidArgument(format!(
            "Nome de {entity} não pode ser caminho absoluto"
        )));
    }

    if normalized.contains('/') || normalized.contains('\\') {
        return Err(AppError::InvalidArgument(format!(
            "Nome de {entity} não pode conter separadores de diretório"
        )));
    }

    if normalized.chars().any(|character| character.is_control()) {
        return Err(AppError::InvalidArgument(format!(
            "Nome de {entity} contém caracteres inválidos"
        )));
    }

    if path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::CurDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(AppError::InvalidArgument(format!(
            "Nome de {entity} contém componentes de caminho inválidos"
        )));
    }

    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::ensure_safe_workspace_component;

    #[test]
    fn safe_workspace_component_accepts_regular_names() {
        let value = ensure_safe_workspace_component("Meu Driver 1", "plugin").unwrap();
        assert_eq!(value, "Meu Driver 1");
    }

    #[test]
    fn safe_workspace_component_rejects_path_traversal() {
        assert!(ensure_safe_workspace_component("../driver", "plugin").is_err());
        assert!(ensure_safe_workspace_component("driver/teste", "plugin").is_err());
        assert!(ensure_safe_workspace_component("driver\\teste", "plugin").is_err());
    }
}
