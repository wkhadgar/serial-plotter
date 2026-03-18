use tauri::State;

use crate::core::error::ErrorDto;
use crate::core::models::plugin::{
    CreatePluginRequest, PluginRegistry, PluginType, UpdatePluginRequest,
};
use crate::core::services::plugin::PluginService;
use crate::core::services::plugin_import::PluginImportService;
use crate::state::AppState;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ImportPluginFileRequest {
    pub content: String,
}

#[tauri::command]
pub fn create_plugin(
    state: State<'_, AppState>,
    request: CreatePluginRequest,
) -> Result<PluginRegistry, ErrorDto> {
    let plugin = PluginService::create(state.plugins(), request).map_err(ErrorDto::from)?;

    Ok(plugin)
}

#[tauri::command]
pub fn get_plugin(state: State<'_, AppState>, id: String) -> Result<PluginRegistry, ErrorDto> {
    PluginService::get(state.plugins(), &id).map_err(ErrorDto::from)
}

#[tauri::command]
pub fn update_plugin(
    state: State<'_, AppState>,
    request: UpdatePluginRequest,
) -> Result<PluginRegistry, ErrorDto> {
    PluginService::update(state.plugins(), request).map_err(ErrorDto::from)
}

#[tauri::command]
pub fn list_plugins(state: State<'_, AppState>) -> Vec<PluginRegistry> {
    PluginService::list(state.plugins())
}

#[tauri::command]
pub fn load_plugins(state: State<'_, AppState>) -> Result<Vec<PluginRegistry>, ErrorDto> {
    PluginService::load_all(state.plugins()).map_err(ErrorDto::from)
}

#[tauri::command]
pub fn delete_plugin(state: State<'_, AppState>, id: String) -> Result<PluginRegistry, ErrorDto> {
    PluginService::remove(state.plugins(), &id).map_err(ErrorDto::from)
}

#[tauri::command]
pub fn import_plugin_file(request: ImportPluginFileRequest) -> Result<PluginRegistry, ErrorDto> {
    PluginImportService::parse_file(&request.content).map_err(ErrorDto::from)
}

#[tauri::command]
#[allow(non_snake_case)]
pub fn list_plugins_by_type(
    state: State<'_, AppState>,
    pluginType: PluginType,
) -> Vec<PluginRegistry> {
    PluginService::list_by_type(state.plugins(), pluginType)
}
