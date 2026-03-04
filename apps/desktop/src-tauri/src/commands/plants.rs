use crate::core::error::ErrorDto;
use crate::core::models::plant::{CreatePlantRequest, PlantResponse};
use crate::core::services::plant::PlantService;

#[tauri::command]
pub fn create_plant(request: CreatePlantRequest) -> Result<PlantResponse, ErrorDto> {
    let plant = PlantService::create_plant(request).map_err(ErrorDto::from)?;
    Ok(PlantResponse::from(&plant))
}
