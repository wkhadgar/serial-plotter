use crate::core::error::ErrorDto;
use crate::core::models::plant::{CreatePlantRequest, PlantResponse, UpdatePlantRequest};
use crate::core::services::plant::PlantService;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn create_plant(
    state: State<'_, AppState>,
    request: CreatePlantRequest,
) -> Result<PlantResponse, ErrorDto> {
    let plant =
        PlantService::create(state.plants(), state.plugins(), request).map_err(ErrorDto::from)?;
    Ok(plant.into())
}

#[tauri::command]
pub fn update_plant(
    state: State<'_, AppState>,
    request: UpdatePlantRequest,
) -> Result<PlantResponse, ErrorDto> {
    let plant =
        PlantService::update(state.plants(), state.plugins(), request).map_err(ErrorDto::from)?;
    Ok(plant.into())
}

#[tauri::command]
pub fn list_plants(state: State<'_, AppState>) -> Vec<PlantResponse> {
    PlantService::list(state.plants())
        .into_iter()
        .map(PlantResponse::from)
        .collect()
}

#[tauri::command]
pub fn get_plant(state: State<'_, AppState>, id: String) -> Result<PlantResponse, ErrorDto> {
    let plant = PlantService::get(state.plants(), &id).map_err(ErrorDto::from)?;
    Ok(plant.into())
}

#[tauri::command]
pub fn remove_plant(state: State<'_, AppState>, id: String) -> Result<PlantResponse, ErrorDto> {
    let plant = PlantService::remove(state.plants(), &id).map_err(ErrorDto::from)?;
    Ok(plant.into())
}

#[tauri::command]
pub fn connect_plant(state: State<'_, AppState>, id: String) -> Result<PlantResponse, ErrorDto> {
    let plant = PlantService::connect(state.plants(), &id).map_err(ErrorDto::from)?;
    Ok(plant.into())
}

#[tauri::command]
pub fn disconnect_plant(state: State<'_, AppState>, id: String) -> Result<PlantResponse, ErrorDto> {
    let plant = PlantService::disconnect(state.plants(), &id).map_err(ErrorDto::from)?;
    Ok(plant.into())
}

#[tauri::command]
pub fn pause_plant(state: State<'_, AppState>, id: String) -> Result<PlantResponse, ErrorDto> {
    let plant = PlantService::pause(state.plants(), &id).map_err(ErrorDto::from)?;
    Ok(plant.into())
}

#[tauri::command]
pub fn resume_plant(state: State<'_, AppState>, id: String) -> Result<PlantResponse, ErrorDto> {
    let plant = PlantService::resume(state.plants(), &id).map_err(ErrorDto::from)?;
    Ok(plant.into())
}
