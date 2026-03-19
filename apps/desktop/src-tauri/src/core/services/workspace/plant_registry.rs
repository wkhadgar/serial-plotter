use crate::core::error::AppResult;
use crate::core::models::plant::Plant;
use crate::core::services::workspace::normalize_entity_name;

use super::io::{create_dir_all, remove_dir_if_exists, write_json_pretty};
use super::paths::{plant_directory, REGISTRY_FILE};

pub(super) fn save(plant: &Plant) -> AppResult<()> {
    let plant_dir = plant_directory(&plant.name)?;
    create_dir_all(
        &plant_dir,
        &format!("Falha ao criar diretório da planta '{}'", plant.name),
    )?;

    let registry_path = plant_dir.join(REGISTRY_FILE);
    write_json_pretty(
        &registry_path,
        plant,
        &format!("Falha ao serializar registro da planta '{}'", plant.name),
        &format!(
            "Falha ao salvar registro da planta '{}' em '{}'",
            plant.name,
            registry_path.display()
        ),
    )
}

pub(super) fn update(plant: &Plant, previous_plant_name: &str) -> AppResult<()> {
    let previous_dir = plant_directory(previous_plant_name)?;
    let next_dir = plant_directory(&plant.name)?;

    if previous_dir != next_dir {
        remove_dir_if_exists(
            &previous_dir,
            &format!(
                "Falha ao remover diretório antigo da planta '{}'",
                previous_dir.display()
            ),
        )?;
    }

    save(plant)
}

pub(super) fn delete(plant_name: &str) -> AppResult<()> {
    let normalized_name = normalize_entity_name(plant_name);
    if normalized_name.is_empty() {
        return Ok(());
    }

    let plant_dir = plant_directory(normalized_name)?;
    remove_dir_if_exists(
        &plant_dir,
        &format!(
            "Falha ao remover diretório da planta '{}' em '{}'",
            normalized_name,
            plant_dir.display()
        ),
    )
}
