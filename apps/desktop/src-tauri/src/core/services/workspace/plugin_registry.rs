use crate::core::error::AppResult;
use crate::core::models::plugin::{PluginRegistry, PluginType};
use crate::core::services::workspace::{map_io_error, map_serde_error, normalize_entity_name};
use std::fs;

use super::paths::{
    plugin_directory, plugin_root_directory, plugin_source_path, REGISTRY_FILE, SOURCE_FILE,
};

pub(super) fn save(plugin: &PluginRegistry, source_code: &str) -> AppResult<()> {
    let plugin_dir = plugin_directory(&plugin.name, plugin.plugin_type)?;

    fs::create_dir_all(&plugin_dir).map_err(|error| {
        map_io_error(
            &format!(
                "Falha ao criar diretório do {:?} '{}'",
                plugin.plugin_type, plugin.name
            ),
            error,
        )
    })?;

    let source_path = plugin_dir.join(SOURCE_FILE);
    fs::write(&source_path, source_code).map_err(|error| {
        map_io_error(
            &format!(
                "Falha ao salvar código fonte do {:?} '{}' em '{}'",
                plugin.plugin_type,
                plugin.name,
                source_path.display()
            ),
            error,
        )
    })?;

    let registry_payload = serde_json::to_string_pretty(plugin).map_err(|error| {
        map_serde_error(
            &format!(
                "Falha ao serializar registro do {:?} '{}'",
                plugin.plugin_type, plugin.name
            ),
            error,
        )
    })?;

    let registry_path = plugin_dir.join(REGISTRY_FILE);
    fs::write(&registry_path, registry_payload).map_err(|error| {
        map_io_error(
            &format!(
                "Falha ao salvar registro do {:?} '{}' em '{}'",
                plugin.plugin_type,
                plugin.name,
                registry_path.display()
            ),
            error,
        )
    })?;

    Ok(())
}

pub(super) fn update(
    plugin: &PluginRegistry,
    source_code: &str,
    previous_plugin_name: &str,
    previous_plugin_type: PluginType,
) -> AppResult<()> {
    let previous_dir = plugin_directory(previous_plugin_name, previous_plugin_type)?;
    let next_dir = plugin_directory(&plugin.name, plugin.plugin_type)?;

    if previous_dir != next_dir && previous_dir.exists() {
        fs::remove_dir_all(&previous_dir).map_err(|error| {
            map_io_error(
                &format!(
                    "Falha ao remover diretório antigo do plugin '{}'",
                    previous_dir.display()
                ),
                error,
            )
        })?;
    }

    save(plugin, source_code)
}

pub(super) fn read_source(plugin_name: &str, plugin_type: PluginType) -> AppResult<String> {
    let source_path = plugin_source_path(plugin_name, plugin_type)?;

    fs::read_to_string(&source_path).map_err(|error| {
        map_io_error(
            &format!(
                "Falha ao ler código fonte do plugin em '{}'",
                source_path.display()
            ),
            error,
        )
    })
}

pub(super) fn delete(plugin_name: &str, plugin_type: PluginType) -> AppResult<()> {
    let normalized_name = normalize_entity_name(plugin_name);
    if normalized_name.is_empty() {
        return Ok(());
    }

    let plugin_dir = plugin_directory(normalized_name, plugin_type)?;
    if !plugin_dir.exists() {
        return Ok(());
    }

    fs::remove_dir_all(&plugin_dir).map_err(|error| {
        map_io_error(
            &format!(
                "Falha ao remover diretório do plugin '{}' em '{}'",
                normalized_name,
                plugin_dir.display()
            ),
            error,
        )
    })
}

pub(super) fn load() -> AppResult<Vec<PluginRegistry>> {
    let mut plugins: Vec<PluginRegistry> = vec![];

    let controller_workspace = plugin_root_directory(PluginType::Controller)?;
    let driver_workspace = plugin_root_directory(PluginType::Driver)?;

    let mut plugins_folders: Vec<std::path::PathBuf> = if controller_workspace.exists() {
        fs::read_dir(&controller_workspace)
            .map_err(|error| map_io_error("Falha ao carregar workspace de plugins", error))?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.is_dir())
            .collect()
    } else {
        vec![]
    };

    let mut controller_folders: Vec<std::path::PathBuf> = if driver_workspace.exists() {
        fs::read_dir(&driver_workspace)
            .map_err(|error| map_io_error("Falha ao carregar workspace de plugins", error))?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.is_dir())
            .collect()
    } else {
        vec![]
    };

    plugins_folders.append(&mut controller_folders);

    for entry in plugins_folders {
        let Some(ret) = fs::read_dir(entry)
            .map_err(|error| map_io_error("Falha ao carregar pastas dos plugins", error))?
            .filter_map(|entry| entry.ok())
            .find(|entry| entry.file_name().to_str() == Some(REGISTRY_FILE))
        else {
            continue;
        };

        let content = fs::read_to_string(ret.path())
            .map_err(|error| map_io_error("Falha ao ler registro do plugin", error))?;

        let json = serde_json::from_str::<PluginRegistry>(&content)
            .map_err(|error| map_serde_error("Falha ao desserializar registro do plugin", error))?;

        plugins.push(json);
    }

    Ok(plugins)
}
