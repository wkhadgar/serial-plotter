use crate::core::error::ErrorDto;
use crate::core::models::plant::{
    CreatePlantRequest, Plant, PlantResponse, RemovePlantControllerRequest,
    SavePlantControllerConfigRequest, SavePlantSetpointRequest, UpdatePlantRequest,
};
use crate::core::services::plant::PlantService;
use crate::core::services::plant_import::{
    ImportPlantFileResponse, OpenPlantFileResponse, PlantImportFileRequest, PlantImportService,
};
use crate::core::services::runtime::{DriverRuntimeService, PlantRuntimeManager};
use crate::state::AppState;
use serde::Deserialize;
use tauri::{AppHandle, State};

#[derive(Debug, Deserialize)]
pub struct ImportFileRequest {
    #[serde(rename = "fileName")]
    pub file_name: String,
    pub content: String,
}

impl From<ImportFileRequest> for PlantImportFileRequest {
    fn from(value: ImportFileRequest) -> Self {
        Self {
            file_name: value.file_name,
            content: value.content,
        }
    }
}

fn into_plant_response(
    runtimes: &PlantRuntimeManager,
    result: crate::core::error::AppResult<Plant>,
) -> Result<PlantResponse, ErrorDto> {
    result
        .map(|plant| PlantResponse::from(runtimes.apply_live_stats(plant)))
        .map_err(ErrorDto::from)
}

#[tauri::command]
pub fn create_plant(
    state: State<'_, AppState>,
    request: CreatePlantRequest,
) -> Result<PlantResponse, ErrorDto> {
    into_plant_response(state.runtimes(), PlantService::create(
        state.plants(),
        state.plugins(),
        request,
    ))
}

#[tauri::command]
pub fn update_plant(
    state: State<'_, AppState>,
    request: UpdatePlantRequest,
) -> Result<PlantResponse, ErrorDto> {
    into_plant_response(state.runtimes(), PlantService::update(
        state.plants(),
        state.plugins(),
        request,
    ))
}

#[tauri::command]
pub fn list_plants(state: State<'_, AppState>) -> Vec<PlantResponse> {
    state
        .runtimes()
        .apply_live_stats_batch(PlantService::list(state.plants()))
        .into_iter()
        .map(PlantResponse::from)
        .collect()
}

#[tauri::command]
pub fn get_plant(state: State<'_, AppState>, id: String) -> Result<PlantResponse, ErrorDto> {
    into_plant_response(state.runtimes(), PlantService::get(state.plants(), &id))
}

#[tauri::command]
pub fn close_plant(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<PlantResponse, ErrorDto> {
    into_plant_response(state.runtimes(), DriverRuntimeService::close(
        &app,
        state.plants(),
        state.runtimes(),
        &id,
    ))
}

#[tauri::command]
pub fn remove_plant(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<PlantResponse, ErrorDto> {
    into_plant_response(state.runtimes(), DriverRuntimeService::remove(
        &app,
        state.plants(),
        state.runtimes(),
        &id,
    ))
}

#[tauri::command]
pub fn connect_plant(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<PlantResponse, ErrorDto> {
    into_plant_response(state.runtimes(), DriverRuntimeService::connect(
        &app,
        state.plants(),
        state.plugins(),
        state.runtimes(),
        &id,
    ))
}

#[tauri::command]
pub fn disconnect_plant(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<PlantResponse, ErrorDto> {
    into_plant_response(state.runtimes(), DriverRuntimeService::disconnect(
        &app,
        state.plants(),
        state.runtimes(),
        &id,
    ))
}

#[tauri::command]
pub fn pause_plant(state: State<'_, AppState>, id: String) -> Result<PlantResponse, ErrorDto> {
    into_plant_response(state.runtimes(), DriverRuntimeService::pause(
        state.plants(),
        state.runtimes(),
        &id,
    ))
}

#[tauri::command]
pub fn resume_plant(state: State<'_, AppState>, id: String) -> Result<PlantResponse, ErrorDto> {
    into_plant_response(state.runtimes(), DriverRuntimeService::resume(
        state.plants(),
        state.runtimes(),
        &id,
    ))
}

#[tauri::command]
pub fn save_controller(
    state: State<'_, AppState>,
    request: SavePlantControllerConfigRequest,
) -> Result<PlantResponse, ErrorDto> {
    into_plant_response(state.runtimes(), DriverRuntimeService::save_controller_config(
        state.plants(),
        state.plugins(),
        state.runtimes(),
        request,
    ))
}

#[tauri::command]
pub fn remove_controller(
    state: State<'_, AppState>,
    request: RemovePlantControllerRequest,
) -> Result<PlantResponse, ErrorDto> {
    into_plant_response(state.runtimes(), DriverRuntimeService::remove_controller(
        state.plants(),
        state.plugins(),
        state.runtimes(),
        request,
    ))
}

#[tauri::command]
pub fn save_setpoint(
    state: State<'_, AppState>,
    request: SavePlantSetpointRequest,
) -> Result<PlantResponse, ErrorDto> {
    into_plant_response(state.runtimes(), DriverRuntimeService::save_setpoint(
        state.plants(),
        state.runtimes(),
        request,
    ))
}

#[tauri::command]
pub fn open_plant_file(request: ImportFileRequest) -> Result<OpenPlantFileResponse, ErrorDto> {
    PlantImportService::open_file(request.into()).map_err(ErrorDto::from)
}

#[tauri::command]
pub fn import_plant_file(
    state: State<'_, AppState>,
    request: ImportFileRequest,
) -> Result<ImportPlantFileResponse, ErrorDto> {
    PlantImportService::import_file(state.plants(), state.plugins(), request.into())
        .map_err(ErrorDto::from)
}
