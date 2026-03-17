use crate::core::error::AppResult;
use crate::core::models::plant::Plant;
use crate::core::services::workspace::{map_io_error, map_serde_error, normalize_entity_name};
use std::fs;

use super::paths::{plant_directory, REGISTRY_FILE};

pub(super) fn save(plant: &Plant) -> AppResult<()> {
    let plant_dir = plant_directory(&plant.name)?;
    fs::create_dir_all(&plant_dir).map_err(|error| {
        map_io_error(
            &format!("Falha ao criar diretório da planta '{}'", plant.name),
            error,
        )
    })?;

    let registry_payload = serde_json::to_string_pretty(plant).map_err(|error| {
        map_serde_error(
            &format!("Falha ao serializar registro da planta '{}'", plant.name),
            error,
        )
    })?;

    let registry_path = plant_dir.join(REGISTRY_FILE);
    fs::write(&registry_path, registry_payload).map_err(|error| {
        map_io_error(
            &format!(
                "Falha ao salvar registro da planta '{}' em '{}'",
                plant.name,
                registry_path.display()
            ),
            error,
        )
    })?;

    Ok(())
}

pub(super) fn update(plant: &Plant, previous_plant_name: &str) -> AppResult<()> {
    let previous_dir = plant_directory(previous_plant_name)?;
    let next_dir = plant_directory(&plant.name)?;

    if previous_dir != next_dir && previous_dir.exists() {
        fs::remove_dir_all(&previous_dir).map_err(|error| {
            map_io_error(
                &format!(
                    "Falha ao remover diretório antigo da planta '{}'",
                    previous_dir.display()
                ),
                error,
            )
        })?;
    }

    save(plant)
}

pub(super) fn delete(plant_name: &str) -> AppResult<()> {
    let normalized_name = normalize_entity_name(plant_name);
    if normalized_name.is_empty() {
        return Ok(());
    }

    let plant_dir = plant_directory(normalized_name)?;
    if !plant_dir.exists() {
        return Ok(());
    }

    fs::remove_dir_all(&plant_dir).map_err(|error| {
        map_io_error(
            &format!(
                "Falha ao remover diretório da planta '{}' em '{}'",
                normalized_name,
                plant_dir.display()
            ),
            error,
        )
    })
}
