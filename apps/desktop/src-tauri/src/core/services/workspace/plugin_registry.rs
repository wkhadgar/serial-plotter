use crate::core::error::AppResult;
use crate::core::models::plugin::{PluginRegistry, PluginType};
use crate::core::services::workspace::{map_io_error, map_serde_error, normalize_entity_name};
use std::fs;

use super::io::{
    create_dir_all, read_to_string, remove_dir_if_exists, write_json_pretty, write_string,
};
use super::paths::{
    plugin_directory, plugin_root_directory, plugin_source_path, REGISTRY_FILE, SOURCE_FILE,
};

pub(super) fn save(plugin: &PluginRegistry, source_code: &str) -> AppResult<()> {
    let plugin_dir = plugin_directory(&plugin.name, plugin.plugin_type)?;
    create_dir_all(
        &plugin_dir,
        &format!(
            "Falha ao criar diretório do {:?} '{}'",
            plugin.plugin_type, plugin.name
        ),
    )?;

    let source_path = plugin_dir.join(SOURCE_FILE);
    write_string(
        &source_path,
        source_code,
        &format!(
            "Falha ao salvar código fonte do {:?} '{}' em '{}'",
            plugin.plugin_type,
            plugin.name,
            source_path.display()
        ),
    )?;

    let registry_path = plugin_dir.join(REGISTRY_FILE);
    write_json_pretty(
        &registry_path,
        plugin,
        &format!(
            "Falha ao serializar registro do {:?} '{}'",
            plugin.plugin_type, plugin.name
        ),
        &format!(
            "Falha ao salvar registro do {:?} '{}' em '{}'",
            plugin.plugin_type,
            plugin.name,
            registry_path.display()
        ),
    )
}

pub(super) fn update(
    plugin: &PluginRegistry,
    source_code: &str,
    previous_plugin_name: &str,
    previous_plugin_type: PluginType,
) -> AppResult<()> {
    let previous_dir = plugin_directory(previous_plugin_name, previous_plugin_type)?;
    let next_dir = plugin_directory(&plugin.name, plugin.plugin_type)?;

    if previous_dir != next_dir {
        remove_dir_if_exists(
            &previous_dir,
            &format!(
                "Falha ao remover diretório antigo do plugin '{}'",
                previous_dir.display()
            ),
        )?;
    }

    save(plugin, source_code)
}

pub(super) fn read_source(plugin_name: &str, plugin_type: PluginType) -> AppResult<String> {
    let source_path = plugin_source_path(plugin_name, plugin_type)?;
    read_to_string(
        &source_path,
        &format!(
            "Falha ao ler código fonte do plugin em '{}'",
            source_path.display()
        ),
    )
}

pub(super) fn delete(plugin_name: &str, plugin_type: PluginType) -> AppResult<()> {
    let normalized_name = normalize_entity_name(plugin_name);
    if normalized_name.is_empty() {
        return Ok(());
    }

    let plugin_dir = plugin_directory(normalized_name, plugin_type)?;
    remove_dir_if_exists(
        &plugin_dir,
        &format!(
            "Falha ao remover diretório do plugin '{}' em '{}'",
            normalized_name,
            plugin_dir.display()
        ),
    )
}

pub(super) fn load() -> AppResult<Vec<PluginRegistry>> {
    let mut plugins = Vec::new();
    load_plugin_type(PluginType::Controller, &mut plugins)?;
    load_plugin_type(PluginType::Driver, &mut plugins)?;
    Ok(plugins)
}

fn load_plugin_type(plugin_type: PluginType, plugins: &mut Vec<PluginRegistry>) -> AppResult<()> {
    let root = plugin_root_directory(plugin_type)?;
    if !root.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(&root)
        .map_err(|error| map_io_error("Falha ao carregar workspace de plugins", &error))?
    {
        let Ok(entry) = entry else {
            continue;
        };
        let plugin_dir = entry.path();
        if !plugin_dir.is_dir() {
            continue;
        }

        let registry_path = plugin_dir.join(REGISTRY_FILE);
        if !registry_path.exists() {
            continue;
        }

        let content = match read_to_string(
            &registry_path,
            &format!(
                "Falha ao ler registro do plugin em '{}'",
                registry_path.display()
            ),
        ) {
            Ok(content) => content,
            Err(error) => {
                eprintln!("{error}");
                continue;
            }
        };

        let plugin = match serde_json::from_str::<PluginRegistry>(&content) {
            Ok(plugin) => plugin,
            Err(error) => {
                eprintln!(
                    "{}",
                    map_serde_error(
                        &format!(
                            "Falha ao desserializar registro do plugin em '{}'",
                            registry_path.display()
                        ),
                        &error,
                    )
                );
                continue;
            }
        };

        plugins.push(plugin);
    }

    Ok(())
}
