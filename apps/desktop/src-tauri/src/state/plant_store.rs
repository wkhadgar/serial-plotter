use crate::core::error::{AppError, AppResult};
use crate::core::models::plant::Plant;
use parking_lot::RwLock;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct PlantStore {
    plants: RwLock<HashMap<String, Plant>>,
}

impl PlantStore {
    pub fn new() -> Self {
        Self {
            plants: RwLock::new(HashMap::new()),
        }
    }

    pub fn insert(&self, plant: Plant) -> AppResult<()> {
        let mut plants = self.plants.write();

        if plants.contains_key(&plant.id) {
            return Err(AppError::InvalidArgument(format!(
                "Planta com ID '{}' já existe",
                plant.id
            )));
        }

        plants.insert(plant.id.clone(), plant);
        Ok(())
    }

    pub fn get(&self, id: &str) -> AppResult<Plant> {
        let plants = self.plants.read();

        plants
            .get(id)
            .cloned()
            .ok_or_else(|| AppError::NotFound(format!("Planta '{}' não encontrada", id)))
    }

    pub fn list(&self) -> Vec<Plant> {
        let plants = self.plants.read();
        plants.values().cloned().collect()
    }

    pub fn remove(&self, id: &str) -> AppResult<Plant> {
        let mut plants = self.plants.write();

        plants
            .remove(id)
            .ok_or_else(|| AppError::NotFound(format!("Planta '{}' não encontrada", id)))
    }

    pub fn update<F>(&self, id: &str, updater: F) -> AppResult<Plant>
    where
        F: FnOnce(&mut Plant),
    {
        let mut plants = self.plants.write();

        let plant = plants
            .get_mut(id)
            .ok_or_else(|| AppError::NotFound(format!("Planta '{}' não encontrada", id)))?;

        updater(plant);
        Ok(plant.clone())
    }

    pub fn exists(&self, id: &str) -> bool {
        let plants = self.plants.read();
        plants.contains_key(id)
    }

    pub fn exists_by_name(&self, name: &str) -> bool {
        let plants = self.plants.read();

        plants.values().any(|plant| plant.name == name)
    }

    pub fn count(&self) -> usize {
        let plants = self.plants.read();
        plants.len()
    }

    pub fn clear(&self) {
        let mut plants = self.plants.write();
        plants.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::plant::{PlantStats, PlantVariable, VariableType};

    fn create_test_plant(id: &str, name: &str) -> Plant {
        Plant {
            id: id.to_string(),
            name: name.to_string(),
            variables: vec![PlantVariable {
                id: "var_0".to_string(),
                name: "Temperatura".to_string(),
                var_type: VariableType::Sensor,
                unit: "°C".to_string(),
                setpoint: 50.0,
                pv_min: 0.0,
                pv_max: 100.0,
                linked_sensor_ids: None,
            }],
            driver_id: None,
            controller_ids: None,
            connected: false,
            paused: false,
            stats: PlantStats::default(),
        }
    }

    #[test]
    fn test_insert_and_get() {
        let store = PlantStore::new();
        let plant = create_test_plant("plant_1", "Test Plant");

        assert!(store.insert(plant.clone()).is_ok());

        let retrieved = store.get("plant_1").unwrap();
        assert_eq!(retrieved.name, "Test Plant");
    }

    #[test]
    fn test_insert_duplicate_fails() {
        let store = PlantStore::new();
        let plant = create_test_plant("plant_1", "Test Plant");

        assert!(store.insert(plant.clone()).is_ok());
        assert!(store.insert(plant).is_err());
    }

    #[test]
    fn test_list() {
        let store = PlantStore::new();
        store
            .insert(create_test_plant("plant_1", "Plant 1"))
            .unwrap();
        store
            .insert(create_test_plant("plant_2", "Plant 2"))
            .unwrap();

        let plants = store.list();
        assert_eq!(plants.len(), 2);
    }

    #[test]
    fn test_remove() {
        let store = PlantStore::new();
        store
            .insert(create_test_plant("plant_1", "Test Plant"))
            .unwrap();

        let removed = store.remove("plant_1").unwrap();
        assert_eq!(removed.name, "Test Plant");
        assert!(!store.exists("plant_1"));
    }

    #[test]
    fn test_update() {
        let store = PlantStore::new();
        store
            .insert(create_test_plant("plant_1", "Original"))
            .unwrap();

        let updated = store
            .update("plant_1", |p| {
                p.name = "Updated".to_string();
                p.connected = true;
            })
            .unwrap();

        assert_eq!(updated.name, "Updated");
        assert!(updated.connected);
    }
}
