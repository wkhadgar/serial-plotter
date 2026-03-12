use crate::core::error::{AppError, AppResult};
use crate::core::models::plant::{
    CreatePlantRequest, CreatePlantVariableRequest, Plant, PlantStats, PlantVariable,
};
use crate::state::PlantStore;
use uuid::Uuid;

pub struct PlantService;

impl PlantService {
    pub fn create(store: &PlantStore, request: CreatePlantRequest) -> AppResult<Plant> {
        Self::validate_request(&request, store)?;

        let plant = Self::build_plant(request);

        store.insert(plant.clone())?;
        Ok(plant)
    }

    fn build_plant(request: CreatePlantRequest) -> Plant {
        let plant_id = format!("plant_{}", Uuid::new_v4());
        let sample_time_ms = request.sample_time_ms;

        let variables = request
            .variables
            .into_iter()
            .enumerate()
            .map(|(idx, var)| PlantVariable {
                id: format!("var_{}", idx),
                name: var.name,
                var_type: var.var_type,
                unit: var.unit,
                setpoint: var.setpoint,
                pv_min: var.pv_min,
                pv_max: var.pv_max,
                linked_sensor_ids: var.linked_sensor_ids,
            })
            .collect();

        Plant {
            id: plant_id,
            name: request.name,
            sample_time_ms,
            variables,
            driver_id: request.driver_id,
            controller_ids: request.controller_ids,
            connected: false,
            paused: false,
            stats: PlantStats {
                dt: sample_time_ms as f64 / 1000.0,
                uptime: 0,
            },
        }
    }

    fn validate_request(request: &CreatePlantRequest, store: &PlantStore) -> AppResult<()> {
        if request.name.trim().is_empty() {
            return Err(AppError::InvalidArgument(
                "Nome da planta é obrigatório".into(),
            ));
        }

        if store.exists_by_name(&request.name) {
            return Err(AppError::InvalidArgument(format!(
                "Planta com NOME '{}' já existe",
                request.name
            )));
        }

        if request.variables.is_empty() {
            return Err(AppError::InvalidArgument(
                "Pelo menos uma variável deve ser definida".into(),
            ));
        }

        if request.sample_time_ms == 0 {
            return Err(AppError::InvalidArgument(
                "Tempo de amostragem deve ser maior que 0 ms".into(),
            ));
        }

        for (idx, var) in request.variables.iter().enumerate() {
            Self::validate_variable(var).map_err(|e| {
                AppError::InvalidArgument(format!("Variável {} inválida: {}", idx, e))
            })?;
        }

        Ok(())
    }

    fn validate_variable(var: &CreatePlantVariableRequest) -> AppResult<()> {
        if var.name.trim().is_empty() {
            return Err(AppError::InvalidArgument(
                "Nome da variável é obrigatório".into(),
            ));
        }

        if var.pv_min >= var.pv_max {
            return Err(AppError::InvalidArgument(
                "pv_min deve ser menor que pv_max".into(),
            ));
        }

        if var.setpoint < var.pv_min || var.setpoint > var.pv_max {
            return Err(AppError::InvalidArgument(
                "setpoint deve estar entre pv_min e pv_max".into(),
            ));
        }

        Ok(())
    }

    pub fn get(store: &PlantStore, id: &str) -> AppResult<Plant> {
        store.get(id)
    }

    pub fn list(store: &PlantStore) -> Vec<Plant> {
        store.list()
    }

    pub fn remove(store: &PlantStore, id: &str) -> AppResult<Plant> {
        store.remove(id)
    }

    pub fn connect(store: &PlantStore, id: &str) -> AppResult<Plant> {
        store.update(id, |plant| {
            plant.connected = true;
        })
    }

    pub fn disconnect(store: &PlantStore, id: &str) -> AppResult<Plant> {
        store.update(id, |plant| {
            plant.connected = false;
            plant.paused = false;
        })
    }

    pub fn pause(store: &PlantStore, id: &str) -> AppResult<Plant> {
        store.update(id, |plant| {
            plant.paused = true;
        })
    }

    pub fn resume(store: &PlantStore, id: &str) -> AppResult<Plant> {
        store.update(id, |plant| {
            plant.paused = false;
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::plant::VariableType;

    fn create_test_variable(name: &str) -> CreatePlantVariableRequest {
        CreatePlantVariableRequest {
            name: name.to_string(),
            var_type: VariableType::Sensor,
            unit: "C".to_string(),
            setpoint: 50.0,
            pv_min: 0.0,
            pv_max: 100.0,
            linked_sensor_ids: None,
        }
    }

    fn create_valid_request(name: &str) -> CreatePlantRequest {
        CreatePlantRequest {
            name: name.to_string(),
            sample_time_ms: 100,
            variables: vec![create_test_variable("Temperatura")],
            driver_id: None,
            controller_ids: None,
        }
    }

    #[test]
    fn test_create_plant_success() {
        let store = PlantStore::new();
        let request = create_valid_request("Planta 1");

        let result = PlantService::create(&store, request);
        assert!(result.is_ok());

        let plant = result.unwrap();
        assert_eq!(plant.name, "Planta 1");
        assert_eq!(plant.sample_time_ms, 100);
        assert_eq!(plant.variables.len(), 1);
        assert!(!plant.connected);
        assert!(!plant.paused);
        assert!(store.exists(&plant.id));
    }

    #[test]
    fn test_create_plant_empty_name() {
        let store = PlantStore::new();
        let request = CreatePlantRequest {
            name: "".to_string(),
            sample_time_ms: 100,
            variables: vec![create_test_variable("Temperatura")],
            driver_id: None,
            controller_ids: None,
        };

        let result = PlantService::create(&store, request);
        assert!(result.is_err());
        assert_eq!(store.count(), 0);
    }

    #[test]
    fn test_create_plant_whitespace_name() {
        let store = PlantStore::new();
        let request = CreatePlantRequest {
            name: "   ".to_string(),
            sample_time_ms: 100,
            variables: vec![create_test_variable("Temperatura")],
            driver_id: None,
            controller_ids: None,
        };

        let result = PlantService::create(&store, request);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_plant_no_variables() {
        let store = PlantStore::new();
        let request = CreatePlantRequest {
            name: "Planta 1".to_string(),
            sample_time_ms: 100,
            variables: vec![],
            driver_id: None,
            controller_ids: None,
        };

        let result = PlantService::create(&store, request);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_plant_invalid_pv_range() {
        let store = PlantStore::new();
        let mut var = create_test_variable("Temp");
        var.pv_min = 100.0;
        var.pv_max = 0.0;

        let request = CreatePlantRequest {
            name: "Planta 1".to_string(),
            sample_time_ms: 100,
            variables: vec![var],
            driver_id: None,
            controller_ids: None,
        };

        let result = PlantService::create(&store, request);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_plant_invalid_setpoint() {
        let store = PlantStore::new();
        let mut var = create_test_variable("Temp");
        var.setpoint = 150.0;

        let request = CreatePlantRequest {
            name: "Planta 1".to_string(),
            sample_time_ms: 100,
            variables: vec![var],
            driver_id: None,
            controller_ids: None,
        };

        let result = PlantService::create(&store, request);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_plant_multiple_variables() {
        let store = PlantStore::new();
        let var1 = create_test_variable("Temperatura");
        let mut var2 = create_test_variable("Umidade");
        var2.unit = "%".to_string();

        let request = CreatePlantRequest {
            name: "Planta Complexa".to_string(),
            sample_time_ms: 250,
            variables: vec![var1, var2],
            driver_id: Some("driver_1".to_string()),
            controller_ids: Some(vec!["ctrl_1".to_string()]),
        };

        let result = PlantService::create(&store, request);
        assert!(result.is_ok());

        let plant = result.unwrap();
        assert_eq!(plant.variables.len(), 2);
        assert_eq!(plant.variables[0].id, "var_0");
        assert_eq!(plant.variables[1].id, "var_1");
        assert_eq!(plant.sample_time_ms, 250);
        assert_eq!(plant.driver_id, Some("driver_1".to_string()));
    }

    #[test]
    fn test_create_plant_duplicate_name() {
        let store = PlantStore::new();

        let request1 = create_valid_request("Mesma Planta");
        PlantService::create(&store, request1).unwrap();

        let request2 = create_valid_request("Mesma Planta");
        let result = PlantService::create(&store, request2);
        assert!(result.is_err());
        assert_eq!(store.count(), 1);
    }

    #[test]
    fn test_create_plant_invalid_sample_time() {
        let store = PlantStore::new();
        let request = CreatePlantRequest {
            name: "Planta 1".to_string(),
            sample_time_ms: 0,
            variables: vec![create_test_variable("Temperatura")],
            driver_id: None,
            controller_ids: None,
        };

        let result = PlantService::create(&store, request);
        assert!(result.is_err());
    }

    #[test]
    fn test_connect_disconnect() {
        let store = PlantStore::new();
        let request = create_valid_request("Test");
        let plant = PlantService::create(&store, request).unwrap();

        let connected = PlantService::connect(&store, &plant.id).unwrap();
        assert!(connected.connected);

        let disconnected = PlantService::disconnect(&store, &plant.id).unwrap();
        assert!(!disconnected.connected);
    }

    #[test]
    fn test_pause_resume() {
        let store = PlantStore::new();
        let request = create_valid_request("Test");
        let plant = PlantService::create(&store, request).unwrap();

        let paused = PlantService::pause(&store, &plant.id).unwrap();
        assert!(paused.paused);

        let resumed = PlantService::resume(&store, &plant.id).unwrap();
        assert!(!resumed.paused);
    }

    #[test]
    fn test_get_plant() {
        let store = PlantStore::new();
        let request = create_valid_request("Test Get");
        let created = PlantService::create(&store, request).unwrap();

        let found = PlantService::get(&store, &created.id).unwrap();
        assert_eq!(found.name, "Test Get");
    }

    #[test]
    fn test_get_plant_not_found() {
        let store = PlantStore::new();
        let result = PlantService::get(&store, "invalid_id");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_plants() {
        let store = PlantStore::new();

        PlantService::create(&store, create_valid_request("Plant A")).unwrap();
        PlantService::create(&store, create_valid_request("Plant B")).unwrap();

        let plants = PlantService::list(&store);
        assert_eq!(plants.len(), 2);
    }

    #[test]
    fn test_remove_plant() {
        let store = PlantStore::new();
        let request = create_valid_request("To Remove");
        let plant = PlantService::create(&store, request).unwrap();

        assert_eq!(store.count(), 1);

        let removed = PlantService::remove(&store, &plant.id).unwrap();
        assert_eq!(removed.name, "To Remove");
        assert_eq!(store.count(), 0);
    }

    #[test]
    fn test_remove_plant_not_found() {
        let store = PlantStore::new();
        let result = PlantService::remove(&store, "invalid_id");
        assert!(result.is_err());
    }
}
