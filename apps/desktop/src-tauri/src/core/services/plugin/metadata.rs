use crate::core::error::AppResult;
use crate::core::models::plugin::{PluginRegistry, PluginRuntime, PluginType};
use crate::core::services::workspace::WorkspaceService;

pub(super) fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|raw| {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

pub(super) fn resolve_entry_class(
    entry_class: Option<&str>,
    plugin_name: &str,
    plugin_type: PluginType,
) -> String {
    let normalized = entry_class
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    match normalized {
        Some(value) if is_valid_entry_class(&value) => value,
        _ => default_entry_class_for(plugin_name, plugin_type),
    }
}

pub(super) fn ensure_runtime_metadata(plugin: &mut PluginRegistry) -> AppResult<()> {
    if plugin.runtime != PluginRuntime::Python {
        return Ok(());
    }

    let normalized = plugin.entry_class.trim().to_string();
    if !normalized.is_empty() && is_valid_entry_class(&normalized) {
        plugin.entry_class = normalized;
        return Ok(());
    }

    let previous_name = plugin.name.clone();
    let previous_type = plugin.plugin_type;
    plugin.entry_class = default_entry_class_for(&plugin.name, plugin.plugin_type);

    let source_code = WorkspaceService::read_plugin_source(&plugin.name, plugin.plugin_type)?;
    WorkspaceService::update_plugin_registry(plugin, &source_code, &previous_name, previous_type)?;
    Ok(())
}

pub(super) fn is_valid_entry_class(value: &str) -> bool {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => {}
        _ => return false,
    }

    chars.all(|character| character.is_ascii_alphanumeric() || character == '_')
}

pub(super) fn default_entry_class_for(plugin_name: &str, plugin_type: PluginType) -> String {
    let fallback = match plugin_type {
        PluginType::Driver => "MyDriver",
        PluginType::Controller => "MyController",
    };

    let filtered: String = plugin_name
        .trim()
        .chars()
        .filter(|character| {
            character.is_ascii_alphanumeric()
                || character.is_ascii_whitespace()
                || *character == '_'
        })
        .collect();

    let mut class_name = String::new();
    for token in filtered
        .split(|character: char| character.is_ascii_whitespace() || character == '_')
        .filter(|token| !token.is_empty())
    {
        let mut chars = token.chars();
        if let Some(first) = chars.next() {
            class_name.push(first.to_ascii_uppercase());
            for character in chars {
                class_name.push(character.to_ascii_lowercase());
            }
        }
    }

    if class_name.is_empty() {
        fallback.to_string()
    } else {
        class_name
    }
}
