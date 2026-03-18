use crate::core::error::{AppError, AppResult};
use crate::core::models::plant::{
    CreatePlantControllerRequest, CreatePlantDriverRequest, CreatePlantRequest,
    CreatePlantVariableRequest, Plant, PlantController, PlantDriver, PlantStats, PlantVariable,
    UpdatePlantRequest, VariableType,
};
use crate::core::models::plugin::{PluginRegistry, PluginType};
use crate::core::services::workspace::WorkspaceService;
use crate::state::{PlantStore, PluginStore};
use std::collections::HashMap;
use uuid::Uuid;

pub struct PlantService;

#[derive(Debug, Clone)]
struct PlantRuntimeSnapshot {
    previous_name: String,
    previous_sample_time_ms: u64,
    connected: bool,
    paused: bool,
    stats: PlantStats,
}

impl PlantService {
    pub fn create(
        store: &PlantStore,
        plugins: &PluginStore,
        request: CreatePlantRequest,
    ) -> AppResult<Plant> {
        Self::validate_payload(
            None,
            &request.name,
            request.sample_time_ms,
            &request.variables,
            &request.driver,
            &request.controllers,
            store,
            plugins,
        )?;

        let plant = Self::build_plant(request, plugins)?;
        WorkspaceService::save_plant_registry(&plant)?;

        if let Err(error) = store.insert(plant.clone()) {
            let _ = WorkspaceService::delete_plant_registry(&plant.name);
            return Err(error);
        }

        Ok(plant)
    }

    pub fn update(
        store: &PlantStore,
        plugins: &PluginStore,
        request: UpdatePlantRequest,
    ) -> AppResult<Plant> {
        Self::validate_payload(
            Some(request.id.as_str()),
            &request.name,
            request.sample_time_ms,
            &request.variables,
            &request.driver,
            &request.controllers,
            store,
            plugins,
        )?;

        let runtime = store.with_plant(&request.id, |plant| PlantRuntimeSnapshot {
            previous_name: plant.name.clone(),
            previous_sample_time_ms: plant.sample_time_ms,
            connected: plant.connected,
            paused: plant.paused,
            stats: plant.stats.clone(),
        })?;

        let previous_name = runtime.previous_name.clone();
        let updated_plant = Self::build_updated_plant(request, plugins, runtime)?;
        WorkspaceService::update_plant_registry(&updated_plant, &previous_name)?;
        store.replace(&updated_plant.id, updated_plant.clone())?;

        Ok(updated_plant)
    }

    fn build_plant(request: CreatePlantRequest, plugins: &PluginStore) -> AppResult<Plant> {
        let plant_id = format!("plant_{}", Uuid::new_v4());
        let sample_time_ms = request.sample_time_ms;
        let driver_plugin =
            Self::resolve_plugin(plugins, &request.driver.plugin_id, PluginType::Driver)?;
        let variables = Self::build_variables(request.variables);
        let controllers = request
            .controllers
            .into_iter()
            .map(|controller| Self::build_controller(controller, plugins))
            .collect::<AppResult<Vec<_>>>()?;

        Ok(Plant {
            id: plant_id,
            name: request.name.trim().to_string(),
            sample_time_ms,
            variables,
            driver: Self::build_driver(request.driver, &driver_plugin),
            controllers,
            connected: false,
            paused: false,
            stats: PlantStats {
                dt: sample_time_ms as f64 / 1000.0,
                uptime: 0,
            },
        })
    }

    fn build_updated_plant(
        request: UpdatePlantRequest,
        plugins: &PluginStore,
        runtime: PlantRuntimeSnapshot,
    ) -> AppResult<Plant> {
        let driver_plugin =
            Self::resolve_plugin(plugins, &request.driver.plugin_id, PluginType::Driver)?;
        let variables = Self::build_variables(request.variables);
        let controllers = request
            .controllers
            .into_iter()
            .map(|controller| Self::build_controller(controller, plugins))
            .collect::<AppResult<Vec<_>>>()?;

        let mut stats = runtime.stats;
        if !runtime.connected || stats.dt == runtime.previous_sample_time_ms as f64 / 1000.0 {
            stats.dt = request.sample_time_ms as f64 / 1000.0;
        }

        Ok(Plant {
            id: request.id,
            name: request.name.trim().to_string(),
            sample_time_ms: request.sample_time_ms,
            variables,
            driver: Self::build_driver(request.driver, &driver_plugin),
            controllers,
            connected: runtime.connected,
            paused: runtime.paused,
            stats,
        })
    }

    fn build_variables(variables: Vec<CreatePlantVariableRequest>) -> Vec<PlantVariable> {
        variables
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
            .collect()
    }

    fn validate_payload(
        current_id: Option<&str>,
        name: &str,
        sample_time_ms: u64,
        variables: &[CreatePlantVariableRequest],
        driver: &CreatePlantDriverRequest,
        controllers: &[CreatePlantControllerRequest],
        store: &PlantStore,
        plugins: &PluginStore,
    ) -> AppResult<()> {
        if name.trim().is_empty() {
            return Err(AppError::InvalidArgument(
                "Nome da planta é obrigatório".into(),
            ));
        }

        let has_duplicate_name = current_id
            .map(|id| store.exists_by_name_except(id, name))
            .unwrap_or_else(|| store.exists_by_name(name));

        if has_duplicate_name {
            return Err(AppError::InvalidArgument(format!(
                "Planta com NOME '{}' já existe",
                name
            )));
        }

        if variables.is_empty() {
            return Err(AppError::InvalidArgument(
                "Pelo menos uma variável deve ser definida".into(),
            ));
        }

        if sample_time_ms == 0 {
            return Err(AppError::InvalidArgument(
                "Tempo de amostragem deve ser maior que 0 ms".into(),
            ));
        }

        if driver.plugin_id.trim().is_empty() {
            return Err(AppError::InvalidArgument(
                "Um driver de comunicação é obrigatório".into(),
            ));
        }

        Self::resolve_plugin(plugins, &driver.plugin_id, PluginType::Driver)?;

        for (idx, var) in variables.iter().enumerate() {
            Self::validate_variable(var).map_err(|error| {
                AppError::InvalidArgument(format!("Variável {} inválida: {}", idx + 1, error))
            })?;
        }

        for (idx, controller) in controllers.iter().enumerate() {
            Self::validate_controller(controller, variables, plugins).map_err(|error| {
                AppError::InvalidArgument(format!("Controlador {} inválido: {}", idx + 1, error))
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

    fn validate_controller(
        controller: &CreatePlantControllerRequest,
        variables: &[CreatePlantVariableRequest],
        plugins: &PluginStore,
    ) -> AppResult<()> {
        if controller.plugin_id.trim().is_empty() {
            return Err(AppError::InvalidArgument(
                "Plugin do controlador é obrigatório".into(),
            ));
        }

        if controller.name.trim().is_empty() {
            return Err(AppError::InvalidArgument(
                "Nome do controlador é obrigatório".into(),
            ));
        }

        if controller.controller_type.trim().is_empty() {
            return Err(AppError::InvalidArgument(
                "Tipo do controlador é obrigatório".into(),
            ));
        }

        if controller.input_variable_ids.is_empty() {
            return Err(AppError::InvalidArgument(
                "O controlador precisa de pelo menos uma variável de entrada".into(),
            ));
        }

        if controller.output_variable_ids.is_empty() {
            return Err(AppError::InvalidArgument(
                "O controlador precisa de pelo menos uma variável de saída".into(),
            ));
        }

        let variable_types = Self::build_variable_type_map(variables);

        for input_id in &controller.input_variable_ids {
            match variable_types.get(input_id) {
                Some(VariableType::Sensor) => {}
                Some(VariableType::Atuador) => {
                    return Err(AppError::InvalidArgument(format!(
                        "A variável '{}' não pode ser usada como entrada",
                        input_id
                    )));
                }
                None => {
                    return Err(AppError::InvalidArgument(format!(
                        "Variável de entrada '{}' não existe",
                        input_id
                    )));
                }
            }
        }

        for output_id in &controller.output_variable_ids {
            match variable_types.get(output_id) {
                Some(VariableType::Atuador) => {}
                Some(VariableType::Sensor) => {
                    return Err(AppError::InvalidArgument(format!(
                        "A variável '{}' não pode ser usada como saída",
                        output_id
                    )));
                }
                None => {
                    return Err(AppError::InvalidArgument(format!(
                        "Variável de saída '{}' não existe",
                        output_id
                    )));
                }
            }
        }

        Self::resolve_plugin(plugins, &controller.plugin_id, PluginType::Controller)?;
        Ok(())
    }

    fn build_variable_type_map(
        variables: &[CreatePlantVariableRequest],
    ) -> HashMap<String, VariableType> {
        variables
            .iter()
            .enumerate()
            .map(|(idx, variable)| (format!("var_{}", idx), variable.var_type))
            .collect()
    }

    fn resolve_plugin(
        plugins: &PluginStore,
        plugin_id: &str,
        expected_type: PluginType,
    ) -> AppResult<PluginRegistry> {
        let plugin = plugins.get(plugin_id)?;

        if plugin.plugin_type != expected_type {
            return Err(AppError::InvalidArgument(format!(
                "Plugin '{}' não é do tipo {}",
                plugin.name,
                expected_type.as_label()
            )));
        }

        Ok(plugin)
    }

    fn build_driver(request: CreatePlantDriverRequest, plugin: &PluginRegistry) -> PlantDriver {
        PlantDriver {
            plugin_id: plugin.id.clone(),
            plugin_name: plugin.name.clone(),
            runtime: plugin.runtime,
            source_file: plugin.source_file.clone(),
            source_code: plugin.source_code.clone(),
            config: request.config,
        }
    }

    fn build_controller(
        request: CreatePlantControllerRequest,
        plugins: &PluginStore,
    ) -> AppResult<PlantController> {
        let plugin = Self::resolve_plugin(plugins, &request.plugin_id, PluginType::Controller)?;

        Ok(PlantController {
            id: request
                .id
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| format!("ctrl_{}", Uuid::new_v4().simple())),
            plugin_id: plugin.id,
            name: request.name.trim().to_string(),
            controller_type: request.controller_type.trim().to_string(),
            active: false,
            input_variable_ids: request.input_variable_ids,
            output_variable_ids: request.output_variable_ids,
            params: request.params,
        })
    }

    pub fn get(store: &PlantStore, id: &str) -> AppResult<Plant> {
        store.get(id)
    }

    pub fn list(store: &PlantStore) -> Vec<Plant> {
        store.list()
    }

    pub fn remove(store: &PlantStore, id: &str) -> AppResult<Plant> {
        let plant = store.get(id)?;
        WorkspaceService::delete_plant_registry(&plant.name)?;
        store.remove(id)
    }

    #[allow(dead_code)]
    pub fn connect(store: &PlantStore, id: &str) -> AppResult<Plant> {
        store.update(id, |plant| {
            plant.connected = true;
        })
    }

    #[allow(dead_code)]
    pub fn disconnect(store: &PlantStore, id: &str) -> AppResult<Plant> {
        store.update(id, |plant| {
            plant.connected = false;
            plant.paused = false;
        })
    }

    #[allow(dead_code)]
    pub fn pause(store: &PlantStore, id: &str) -> AppResult<Plant> {
        store.update(id, |plant| {
            plant.paused = true;
        })
    }

    #[allow(dead_code)]
    pub fn resume(store: &PlantStore, id: &str) -> AppResult<Plant> {
        store.update(id, |plant| {
            plant.paused = false;
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::plant::{ControllerParam, ControllerParamType, VariableType};
    use crate::core::models::plugin::{PluginRuntime, SchemaFieldValue};
    use crate::state::PluginStore;
    use std::collections::HashMap;

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

    fn create_plugin_store() -> PluginStore {
        let store = PluginStore::new();

        store
            .insert(PluginRegistry {
                id: "driver_plugin".to_string(),
                name: "Driver Python".to_string(),
                plugin_type: PluginType::Driver,
                runtime: PluginRuntime::Python,
                schema: vec![],
                source_file: Some("driver.py".to_string()),
                source_code: Some("class Driver:\n    pass".to_string()),
                dependencies: vec![],
                description: None,
                version: None,
                author: None,
            })
            .unwrap();

        store
            .insert(PluginRegistry {
                id: "controller_plugin".to_string(),
                name: "PID".to_string(),
                plugin_type: PluginType::Controller,
                runtime: PluginRuntime::Python,
                schema: vec![],
                source_file: Some("controller.py".to_string()),
                source_code: Some("class Controller:\n    pass".to_string()),
                dependencies: vec![],
                description: None,
                version: None,
                author: None,
            })
            .unwrap();

        store
    }

    fn create_valid_request(name: &str) -> CreatePlantRequest {
        CreatePlantRequest {
            name: name.to_string(),
            sample_time_ms: 100,
            variables: vec![create_test_variable("Temperatura")],
            driver: CreatePlantDriverRequest {
                plugin_id: "driver_plugin".to_string(),
                config: HashMap::new(),
            },
            controllers: vec![],
        }
    }

    #[test]
    fn test_create_plant_success() {
        let store = PlantStore::new();
        let plugins = create_plugin_store();
        let request = create_valid_request("Planta 1");

        let result = PlantService::create(&store, &plugins, request);
        assert!(result.is_ok());

        let plant = result.unwrap();
        assert_eq!(plant.name, "Planta 1");
        assert_eq!(plant.sample_time_ms, 100);
        assert_eq!(plant.variables.len(), 1);
        assert_eq!(plant.driver.plugin_id, "driver_plugin");
        assert!(!plant.connected);
        assert!(!plant.paused);
        assert!(store.exists(&plant.id));
    }

    #[test]
    fn test_update_plant_success() {
        let store = PlantStore::new();
        let plugins = create_plugin_store();
        let created =
            PlantService::create(&store, &plugins, create_valid_request("Planta 1")).unwrap();

        let updated = PlantService::update(
            &store,
            &plugins,
            UpdatePlantRequest {
                id: created.id.clone(),
                name: "Planta Atualizada".to_string(),
                sample_time_ms: 200,
                variables: vec![create_test_variable("Nova Variável")],
                driver: CreatePlantDriverRequest {
                    plugin_id: "driver_plugin".to_string(),
                    config: HashMap::new(),
                },
                controllers: vec![],
            },
        )
        .unwrap();

        assert_eq!(updated.name, "Planta Atualizada");
        assert_eq!(updated.sample_time_ms, 200);
        assert_eq!(updated.variables[0].name, "Nova Variável");
    }

    #[test]
    fn test_create_plant_empty_name() {
        let store = PlantStore::new();
        let plugins = create_plugin_store();
        let request = CreatePlantRequest {
            name: "".to_string(),
            sample_time_ms: 100,
            variables: vec![create_test_variable("Temperatura")],
            driver: CreatePlantDriverRequest {
                plugin_id: "driver_plugin".to_string(),
                config: HashMap::new(),
            },
            controllers: vec![],
        };

        let result = PlantService::create(&store, &plugins, request);
        assert!(result.is_err());
        assert_eq!(store.count(), 0);
    }

    #[test]
    fn test_create_plant_whitespace_name() {
        let store = PlantStore::new();
        let plugins = create_plugin_store();
        let request = CreatePlantRequest {
            name: "   ".to_string(),
            sample_time_ms: 100,
            variables: vec![create_test_variable("Temperatura")],
            driver: CreatePlantDriverRequest {
                plugin_id: "driver_plugin".to_string(),
                config: HashMap::new(),
            },
            controllers: vec![],
        };

        let result = PlantService::create(&store, &plugins, request);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_plant_no_variables() {
        let store = PlantStore::new();
        let plugins = create_plugin_store();
        let request = CreatePlantRequest {
            name: "Planta 1".to_string(),
            sample_time_ms: 100,
            variables: vec![],
            driver: CreatePlantDriverRequest {
                plugin_id: "driver_plugin".to_string(),
                config: HashMap::new(),
            },
            controllers: vec![],
        };

        let result = PlantService::create(&store, &plugins, request);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_plant_invalid_pv_range() {
        let store = PlantStore::new();
        let plugins = create_plugin_store();
        let mut var = create_test_variable("Temp");
        var.pv_min = 100.0;
        var.pv_max = 0.0;

        let request = CreatePlantRequest {
            name: "Planta 1".to_string(),
            sample_time_ms: 100,
            variables: vec![var],
            driver: CreatePlantDriverRequest {
                plugin_id: "driver_plugin".to_string(),
                config: HashMap::new(),
            },
            controllers: vec![],
        };

        let result = PlantService::create(&store, &plugins, request);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_plant_invalid_setpoint() {
        let store = PlantStore::new();
        let plugins = create_plugin_store();
        let mut var = create_test_variable("Temp");
        var.setpoint = 150.0;

        let request = CreatePlantRequest {
            name: "Planta 1".to_string(),
            sample_time_ms: 100,
            variables: vec![var],
            driver: CreatePlantDriverRequest {
                plugin_id: "driver_plugin".to_string(),
                config: HashMap::new(),
            },
            controllers: vec![],
        };

        let result = PlantService::create(&store, &plugins, request);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_plant_multiple_variables() {
        let store = PlantStore::new();
        let plugins = create_plugin_store();
        let var1 = create_test_variable("Temperatura");
        let mut var2 = create_test_variable("Umidade");
        var2.unit = "%".to_string();
        var2.var_type = VariableType::Atuador;
        var2.setpoint = 0.0;

        let request = CreatePlantRequest {
            name: "Planta Complexa".to_string(),
            sample_time_ms: 250,
            variables: vec![var1, var2],
            driver: CreatePlantDriverRequest {
                plugin_id: "driver_plugin".to_string(),
                config: HashMap::new(),
            },
            controllers: vec![CreatePlantControllerRequest {
                id: Some("ctrl_1".to_string()),
                plugin_id: "controller_plugin".to_string(),
                name: "PID 1".to_string(),
                controller_type: "PID".to_string(),
                active: true,
                input_variable_ids: vec!["var_0".to_string()],
                output_variable_ids: vec!["var_1".to_string()],
                params: HashMap::from([(
                    "kp".to_string(),
                    ControllerParam {
                        param_type: ControllerParamType::Number,
                        value: SchemaFieldValue::Float(1.0),
                        label: "Kp".to_string(),
                    },
                )]),
            }],
        };

        let result = PlantService::create(&store, &plugins, request);
        assert!(result.is_ok());

        let plant = result.unwrap();
        assert_eq!(plant.variables.len(), 2);
        assert_eq!(plant.variables[0].id, "var_0");
        assert_eq!(plant.variables[1].id, "var_1");
        assert_eq!(plant.sample_time_ms, 250);
        assert_eq!(plant.driver.plugin_id, "driver_plugin");
        assert_eq!(plant.controllers.len(), 1);
        assert!(!plant.controllers[0].active);
    }

    #[test]
    fn test_create_plant_always_starts_controllers_disabled() {
        let store = PlantStore::new();
        let plugins = create_plugin_store();
        let var1 = create_test_variable("Temperatura");
        let mut var2 = create_test_variable("Valvula A");
        var2.var_type = VariableType::Atuador;
        var2.setpoint = 0.0;

        let request = CreatePlantRequest {
            name: "Planta Com Conflito".to_string(),
            sample_time_ms: 250,
            variables: vec![var1, var2],
            driver: CreatePlantDriverRequest {
                plugin_id: "driver_plugin".to_string(),
                config: HashMap::new(),
            },
            controllers: vec![
                CreatePlantControllerRequest {
                    id: Some("ctrl_1".to_string()),
                    plugin_id: "controller_plugin".to_string(),
                    name: "PID 1".to_string(),
                    controller_type: "PID".to_string(),
                    active: true,
                    input_variable_ids: vec!["var_0".to_string()],
                    output_variable_ids: vec!["var_1".to_string()],
                    params: HashMap::new(),
                },
                CreatePlantControllerRequest {
                    id: Some("ctrl_2".to_string()),
                    plugin_id: "controller_plugin".to_string(),
                    name: "PID 2".to_string(),
                    controller_type: "PID".to_string(),
                    active: true,
                    input_variable_ids: vec!["var_0".to_string()],
                    output_variable_ids: vec!["var_1".to_string()],
                    params: HashMap::new(),
                },
            ],
        };

        let result = PlantService::create(&store, &plugins, request);
        assert!(result.is_ok());

        let plant = result.unwrap();
        assert_eq!(plant.controllers.len(), 2);
        assert!(plant
            .controllers
            .iter()
            .all(|controller| !controller.active));
    }

    #[test]
    fn test_create_plant_duplicate_name() {
        let store = PlantStore::new();
        let plugins = create_plugin_store();

        let request1 = create_valid_request("Mesma Planta");
        PlantService::create(&store, &plugins, request1).unwrap();

        let request2 = create_valid_request("Mesma Planta");
        let result = PlantService::create(&store, &plugins, request2);
        assert!(result.is_err());
        assert_eq!(store.count(), 1);
    }

    #[test]
    fn test_create_plant_invalid_sample_time() {
        let store = PlantStore::new();
        let plugins = create_plugin_store();
        let request = CreatePlantRequest {
            name: "Planta 1".to_string(),
            sample_time_ms: 0,
            variables: vec![create_test_variable("Temperatura")],
            driver: CreatePlantDriverRequest {
                plugin_id: "driver_plugin".to_string(),
                config: HashMap::new(),
            },
            controllers: vec![],
        };

        let result = PlantService::create(&store, &plugins, request);
        assert!(result.is_err());
    }

    #[test]
    fn test_connect_disconnect() {
        let store = PlantStore::new();
        let plugins = create_plugin_store();
        let request = create_valid_request("Test");
        let plant = PlantService::create(&store, &plugins, request).unwrap();

        let connected = PlantService::connect(&store, &plant.id).unwrap();
        assert!(connected.connected);

        let disconnected = PlantService::disconnect(&store, &plant.id).unwrap();
        assert!(!disconnected.connected);
    }

    #[test]
    fn test_pause_resume() {
        let store = PlantStore::new();
        let plugins = create_plugin_store();
        let request = create_valid_request("Test");
        let plant = PlantService::create(&store, &plugins, request).unwrap();

        let paused = PlantService::pause(&store, &plant.id).unwrap();
        assert!(paused.paused);

        let resumed = PlantService::resume(&store, &plant.id).unwrap();
        assert!(!resumed.paused);
    }

    #[test]
    fn test_get_plant() {
        let store = PlantStore::new();
        let plugins = create_plugin_store();
        let request = create_valid_request("Test Get");
        let created = PlantService::create(&store, &plugins, request).unwrap();

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
        let plugins = create_plugin_store();

        PlantService::create(&store, &plugins, create_valid_request("Plant A")).unwrap();
        PlantService::create(&store, &plugins, create_valid_request("Plant B")).unwrap();

        let plants = PlantService::list(&store);
        assert_eq!(plants.len(), 2);
    }

    #[test]
    fn test_remove_plant() {
        let store = PlantStore::new();
        let plugins = create_plugin_store();
        let request = create_valid_request("To Remove");
        let plant = PlantService::create(&store, &plugins, request).unwrap();

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
