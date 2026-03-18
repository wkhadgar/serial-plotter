use crate::core::error::ErrorDto;
use crate::core::models::plant::{CreatePlantRequest, PlantResponse, UpdatePlantRequest};
use crate::core::services::plant::PlantService;
use crate::core::services::plant_import::{
    ImportPlantFileResponse, OpenPlantFileResponse, PlantImportFileRequest, PlantImportService,
};
use crate::core::services::runtime::DriverRuntimeService;
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
pub fn connect_plant(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<PlantResponse, ErrorDto> {
    let plant =
        DriverRuntimeService::connect(&app, state.plants(), state.plugins(), state.runtimes(), &id)
            .map_err(ErrorDto::from)?;

    Ok(plant.into())
}

#[tauri::command]
pub fn disconnect_plant(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<PlantResponse, ErrorDto> {
    let plant = DriverRuntimeService::disconnect(&app, state.plants(), state.runtimes(), &id)
        .map_err(ErrorDto::from)?;

    Ok(plant.into())
}

#[tauri::command]
pub fn pause_plant(state: State<'_, AppState>, id: String) -> Result<PlantResponse, ErrorDto> {
    let plant = DriverRuntimeService::pause(state.plants(), state.runtimes(), &id)
        .map_err(ErrorDto::from)?;
    Ok(plant.into())
}

#[tauri::command]
pub fn resume_plant(state: State<'_, AppState>, id: String) -> Result<PlantResponse, ErrorDto> {
    let plant = DriverRuntimeService::resume(state.plants(), state.runtimes(), &id)
        .map_err(ErrorDto::from)?;
    Ok(plant.into())
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
