use crate::core::error::{AppError, AppResult};
use crate::core::models::plant::Plant;
use crate::state::normalized_name_key;
use parking_lot::RwLock;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct PlantStore {
    state: RwLock<PlantStoreInner>,
}

#[derive(Debug, Default)]
struct PlantStoreInner {
    plants: HashMap<String, Plant>,
    names: HashMap<String, String>,
}

impl PlantStore {
    pub fn new() -> Self {
        Self {
            state: RwLock::new(PlantStoreInner::default()),
        }
    }

    pub fn insert(&self, plant: Plant) -> AppResult<()> {
        let mut state = self.state.write();
        Self::insert_into_state(&mut state, plant)?;
        Ok(())
    }

    pub fn get(&self, id: &str) -> AppResult<Plant> {
        self.state
            .read()
            .plants
            .get(id)
            .cloned()
            .ok_or_else(|| AppError::NotFound(format!("Planta '{}' não encontrada", id)))
    }

    pub fn list(&self) -> Vec<Plant> {
        self.state.read().plants.values().cloned().collect()
    }

    pub fn remove(&self, id: &str) -> AppResult<Plant> {
        let mut state = self.state.write();
        let plant = state
            .plants
            .remove(id)
            .ok_or_else(|| AppError::NotFound(format!("Planta '{}' não encontrada", id)))?;
        state.names.remove(&Self::name_key(&plant.name));
        Ok(plant)
    }

    pub fn update<F>(&self, id: &str, updater: F) -> AppResult<Plant>
    where
        F: FnOnce(&mut Plant),
    {
        let mut state = self.state.write();
        let (previous_name_key, next_name_key, plant_id, plant_snapshot) = {
            let plant = state
                .plants
                .get_mut(id)
                .ok_or_else(|| AppError::NotFound(format!("Planta '{}' não encontrada", id)))?;
            let previous_name_key = Self::name_key(&plant.name);
            updater(plant);
            (
                previous_name_key,
                Self::name_key(&plant.name),
                plant.id.clone(),
                plant.clone(),
            )
        };

        if previous_name_key != next_name_key {
            if let Some(existing_id) = state.names.get(&next_name_key) {
                if existing_id != id {
                    return Err(AppError::InvalidArgument(format!(
                        "Planta com NOME '{}' já existe",
                        plant_snapshot.name.trim()
                    )));
                }
            }
            state.names.remove(&previous_name_key);
            state.names.insert(next_name_key, plant_id);
        }
        Ok(plant_snapshot)
    }

    pub fn replace(&self, id: &str, next_plant: Plant) -> AppResult<()> {
        let mut state = self.state.write();
        let previous_name_key = {
            let current = state
                .plants
                .get(id)
                .ok_or_else(|| AppError::NotFound(format!("Planta '{}' não encontrada", id)))?;
            Self::name_key(&current.name)
        };
        let next_name_key = Self::name_key(&next_plant.name);

        if next_name_key.is_empty() {
            return Err(AppError::InvalidArgument(
                "Nome da planta é obrigatório".into(),
            ));
        }

        if let Some(existing_id) = state.names.get(&next_name_key) {
            if existing_id != id {
                return Err(AppError::InvalidArgument(format!(
                    "Planta com NOME '{}' já existe",
                    next_plant.name.trim()
                )));
            }
        }

        state.names.remove(&previous_name_key);
        state.names.insert(next_name_key, next_plant.id.clone());
        state.plants.insert(id.to_string(), next_plant);
        Ok(())
    }

    #[cfg(test)]
    pub fn exists(&self, id: &str) -> bool {
        self.state.read().plants.contains_key(id)
    }

    pub fn exists_by_name(&self, name: &str) -> bool {
        let key = Self::name_key(name);
        !key.is_empty() && self.state.read().names.contains_key(&key)
    }

    pub fn exists_by_name_except(&self, id: &str, name: &str) -> bool {
        let key = Self::name_key(name);
        if key.is_empty() {
            return false;
        }

        self.state
            .read()
            .names
            .get(&key)
            .is_some_and(|existing_id| existing_id != id)
    }

    #[cfg(test)]
    pub fn count(&self) -> usize {
        self.state.read().plants.len()
    }

    fn insert_into_state(state: &mut PlantStoreInner, plant: Plant) -> AppResult<()> {
        if state.plants.contains_key(&plant.id) {
            return Err(AppError::InvalidArgument(format!(
                "Planta com ID '{}' já existe",
                plant.id
            )));
        }

        let name_key = Self::name_key(&plant.name);
        if name_key.is_empty() {
            return Err(AppError::InvalidArgument(
                "Nome da planta é obrigatório".into(),
            ));
        }

        if state.names.contains_key(&name_key) {
            return Err(AppError::InvalidArgument(format!(
                "Planta com NOME '{}' já existe",
                plant.name.trim()
            )));
        }

        state.names.insert(name_key, plant.id.clone());
        state.plants.insert(plant.id.clone(), plant);
        Ok(())
    }

    fn name_key(name: &str) -> String {
        normalized_name_key(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::plant::{
        Plant, PlantController, PlantDriver, PlantStats, PlantVariable, VariableType,
    };
    use crate::core::models::plugin::PluginRuntime;
    use std::collections::HashMap;

    fn create_test_plant(id: &str, name: &str) -> Plant {
        Plant {
            id: id.to_string(),
            name: name.to_string(),
            sample_time_ms: 100,
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
            driver: PlantDriver {
                plugin_id: "driver_plugin".to_string(),
                plugin_name: "Driver Python".to_string(),
                runtime: PluginRuntime::Python,
                source_file: Some("driver.py".to_string()),
                source_code: Some("class Driver:\n    pass".to_string()),
                config: HashMap::new(),
            },
            controllers: Vec::<PlantController>::new(),
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
    fn test_insert_duplicate_name_ignores_case_and_whitespace() {
        let store = PlantStore::new();

        assert!(store
            .insert(create_test_plant("plant_1", "Planta A"))
            .is_ok());
        assert!(store
            .insert(create_test_plant("plant_2", "  planta a  "))
            .is_err());
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
