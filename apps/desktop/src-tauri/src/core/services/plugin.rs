mod builder;
mod metadata;
mod source;
mod validation;

use self::builder::{build_plugin, build_updated_plugin};
use self::metadata::ensure_runtime_metadata;
use self::source::{normalize_source_code, resolve_source_code_for_create};
use self::validation::validate_request;
use crate::core::error::{AppError, AppResult};
use crate::core::models::plugin::{
    CreatePluginRequest, PluginRegistry, PluginType, UpdatePluginRequest,
};
use crate::core::services::workspace::WorkspaceService;
use crate::state::PluginStore;

const PYTHON_SOURCE_FILE_NAME: &str = "main.py";

pub struct PluginService;

impl PluginService {
    pub fn create(store: &PluginStore, request: CreatePluginRequest) -> AppResult<PluginRegistry> {
        validate_request(
            store,
            None,
            &request.name,
            request.runtime,
            request.entry_class.as_deref(),
            request.source_file.as_deref(),
            request.source_code.as_deref(),
        )?;

        let source_code = resolve_source_code_for_create(
            request.source_code.as_deref(),
            request.source_file.as_deref(),
            &request.name,
            request.plugin_type,
        )?;
        let plugin = build_plugin(request, PYTHON_SOURCE_FILE_NAME);

        WorkspaceService::save_plugin_registry(&plugin, &source_code)?;

        if let Err(error) = store.insert(plugin.clone()) {
            let _ = WorkspaceService::delete_plugin_registry(&plugin.name, plugin.plugin_type);
            return Err(error);
        }

        Ok(plugin)
    }

    pub fn get(store: &PluginStore, id: &str) -> AppResult<PluginRegistry> {
        let mut plugin = store.get(id)?;
        let source_code = WorkspaceService::read_plugin_source(&plugin.name, plugin.plugin_type)?;
        plugin.source_code = Some(source_code);
        Ok(plugin)
    }

    pub fn list(store: &PluginStore) -> Vec<PluginRegistry> {
        store.list()
    }

    pub fn list_by_type(store: &PluginStore, plugin_type: PluginType) -> Vec<PluginRegistry> {
        store.list_by_type(plugin_type)
    }

    pub fn update(store: &PluginStore, request: UpdatePluginRequest) -> AppResult<PluginRegistry> {
        validate_request(
            store,
            Some(request.id.as_str()),
            &request.name,
            request.runtime,
            request.entry_class.as_deref(),
            request.source_file.as_deref(),
            request.source_code.as_deref(),
        )?;

        let (previous_name, previous_type) = store.read(&request.id, |existing| {
            (existing.name.clone(), existing.plugin_type)
        })?;
        if request.plugin_type != previous_type {
            return Err(AppError::InvalidArgument(
                "Tipo do plugin não pode ser alterado".into(),
            ));
        }

        let resolved_source_code = match normalize_source_code(request.source_code.as_deref()) {
            Some(code) => code,
            None => WorkspaceService::read_plugin_source(&previous_name, previous_type)?,
        };

        let next_plugin = build_updated_plugin(request, previous_type, PYTHON_SOURCE_FILE_NAME);

        WorkspaceService::update_plugin_registry(
            &next_plugin,
            resolved_source_code.as_str(),
            &previous_name,
            previous_type,
        )?;
        store.replace(&next_plugin.id, next_plugin.clone())?;

        Ok(next_plugin)
    }

    pub fn load_all(store: &PluginStore) -> AppResult<Vec<PluginRegistry>> {
        let plugins = WorkspaceService::load_plugin_registries()?
            .into_iter()
            .map(|mut plugin| {
                ensure_runtime_metadata(&mut plugin)?;
                plugin.source_code = None;
                Ok(plugin)
            })
            .collect::<AppResult<Vec<_>>>()?;
        store.sync(plugins)
    }

    pub fn remove(store: &PluginStore, id: &str) -> AppResult<PluginRegistry> {
        let (plugin_name, plugin_type) =
            store.read(id, |existing| (existing.name.clone(), existing.plugin_type))?;
        WorkspaceService::delete_plugin_registry(&plugin_name, plugin_type)?;
        store.remove(id)
    }
}

#[cfg(test)]
mod tests;
