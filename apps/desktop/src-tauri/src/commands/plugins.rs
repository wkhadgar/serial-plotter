use tauri::State;

use crate::core::error::ErrorDto;
use crate::core::models::plugin::{
    CreatePluginRequest, PluginRegistry, PluginType, UpdatePluginRequest,
};
use crate::core::services::plugin::PluginService;
use crate::state::AppState;

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
#[allow(non_snake_case)]
pub fn list_plugins_by_type(
    state: State<'_, AppState>,
    pluginType: PluginType,
) -> Vec<PluginRegistry> {
    PluginService::list_by_type(state.plugins(), pluginType)
}
