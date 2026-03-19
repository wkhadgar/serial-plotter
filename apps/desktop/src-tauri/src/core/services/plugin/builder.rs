use super::metadata::{normalize_optional_text, resolve_entry_class};
use crate::core::models::plugin::{CreatePluginRequest, PluginRegistry, UpdatePluginRequest};
use uuid::Uuid;

pub(super) fn build_plugin(request: CreatePluginRequest, source_file_name: &str) -> PluginRegistry {
    PluginRegistry {
        id: format!("plugin_{}", Uuid::new_v4()),
        name: request.name.trim().to_string(),
        plugin_type: request.plugin_type,
        runtime: request.runtime,
        entry_class: resolve_entry_class(
            request.entry_class.as_deref(),
            &request.name,
            request.plugin_type,
        ),
        schema: request.schema,
        source_file: Some(source_file_name.to_string()),
        source_code: None,
        dependencies: request.dependencies,
        description: normalize_optional_text(request.description),
        version: normalize_optional_text(request.version),
        author: normalize_optional_text(request.author),
    }
}

pub(super) fn build_updated_plugin(
    request: UpdatePluginRequest,
    plugin_type: crate::core::models::plugin::PluginType,
    source_file_name: &str,
) -> PluginRegistry {
    PluginRegistry {
        id: request.id,
        name: request.name.trim().to_string(),
        plugin_type,
        runtime: request.runtime,
        entry_class: resolve_entry_class(
            request.entry_class.as_deref(),
            &request.name,
            plugin_type,
        ),
        schema: request.schema,
        source_file: Some(source_file_name.to_string()),
        source_code: None,
        dependencies: request.dependencies,
        description: normalize_optional_text(request.description),
        version: normalize_optional_text(request.version),
        author: normalize_optional_text(request.author),
    }
}
