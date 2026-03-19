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

fn into_plugin_result(
    result: crate::core::error::AppResult<PluginRegistry>,
) -> Result<PluginRegistry, ErrorDto> {
    result.map_err(ErrorDto::from)
}

#[tauri::command]
pub fn create_plugin(
    state: State<'_, AppState>,
    request: CreatePluginRequest,
) -> Result<PluginRegistry, ErrorDto> {
    into_plugin_result(PluginService::create(state.plugins(), request))
}

#[tauri::command]
pub fn get_plugin(state: State<'_, AppState>, id: String) -> Result<PluginRegistry, ErrorDto> {
    into_plugin_result(PluginService::get(state.plugins(), &id))
}

#[tauri::command]
pub fn update_plugin(
    state: State<'_, AppState>,
    request: UpdatePluginRequest,
) -> Result<PluginRegistry, ErrorDto> {
    into_plugin_result(PluginService::update(state.plugins(), request))
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
    into_plugin_result(PluginService::remove(state.plugins(), &id))
}

#[tauri::command]
pub fn import_plugin_file(request: ImportPluginFileRequest) -> Result<PluginRegistry, ErrorDto> {
    into_plugin_result(PluginImportService::parse_file(&request.content))
}

#[tauri::command]
#[allow(non_snake_case)]
pub fn list_plugins_by_type(
    state: State<'_, AppState>,
    pluginType: PluginType,
) -> Vec<PluginRegistry> {
    PluginService::list_by_type(state.plugins(), pluginType)
}
