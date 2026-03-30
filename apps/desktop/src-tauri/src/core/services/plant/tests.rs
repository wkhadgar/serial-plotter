use super::*;
use crate::core::models::plant::{ControllerParam, ControllerParamType, VariableType};
use crate::core::models::plugin::{
    PluginRuntime, PluginSchemaField, SchemaFieldType, SchemaFieldValue,
};
use crate::core::services::workspace::test_workspace_root;
use crate::state::PluginStore;
use std::collections::HashMap;
use std::path::PathBuf;

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
            entry_class: "Driver".to_string(),
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
            entry_class: "Controller".to_string(),
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

#[test]
fn test_fill_missing_controller_params_uses_schema_defaults() {
    let plugin = PluginRegistry {
        id: "controller_plugin".to_string(),
        name: "TCLAB Controller".to_string(),
        plugin_type: PluginType::Controller,
        runtime: PluginRuntime::Python,
        entry_class: "TclabController".to_string(),
        schema: vec![PluginSchemaField {
            name: "open_duty_1".to_string(),
            field_type: SchemaFieldType::Float,
            default_value: Some(SchemaFieldValue::Float(37.5)),
            description: Some("Open duty 1".to_string()),
        }],
        source_file: Some("main.py".to_string()),
        source_code: None,
        dependencies: vec![],
        description: None,
        version: None,
        author: None,
    };
    let mut params = HashMap::new();

    let changed = PlantService::fill_missing_controller_params(&mut params, &plugin);

    assert!(changed);
    let param = params.get("open_duty_1").expect("parametro ausente");
    assert_eq!(param.param_type, ControllerParamType::Number);
    assert_eq!(param.label, "Open duty 1");
    match &param.value {
        SchemaFieldValue::Float(value) => assert!((*value - 37.5).abs() < f64::EPSILON),
        other => panic!("valor inesperado para parametro default: {other:?}"),
    }
}

fn create_valid_request(name: &str) -> CreatePlantRequest {
    CreatePlantRequest {
        name: name.to_string(),
        sample_time_ms: 100,
        variables: vec![create_test_variable("Temperatura")],
        driver: crate::core::models::plant::CreatePlantDriverRequest {
            plugin_id: "driver_plugin".to_string(),
            config: HashMap::new(),
        },
        controllers: vec![],
    }
}

fn plant_registry_path(name: &str) -> PathBuf {
    test_workspace_root()
        .join("plants")
        .join(name)
        .join("registry.json")
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
    let created = PlantService::create(&store, &plugins, create_valid_request("Planta 1")).unwrap();

    let updated = PlantService::update(
        &store,
        &plugins,
        UpdatePlantRequest {
            id: created.id.clone(),
            name: "Planta Atualizada".to_string(),
            sample_time_ms: 200,
            variables: vec![create_test_variable("Nova Variável")],
            driver: crate::core::models::plant::CreatePlantDriverRequest {
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
        name: String::new(),
        sample_time_ms: 100,
        variables: vec![create_test_variable("Temperatura")],
        driver: crate::core::models::plant::CreatePlantDriverRequest {
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
        driver: crate::core::models::plant::CreatePlantDriverRequest {
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
        driver: crate::core::models::plant::CreatePlantDriverRequest {
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
        driver: crate::core::models::plant::CreatePlantDriverRequest {
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
        driver: crate::core::models::plant::CreatePlantDriverRequest {
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
        driver: crate::core::models::plant::CreatePlantDriverRequest {
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
    assert!(plant.controllers[0].active);
}

#[test]
fn test_create_plant_preserves_controller_active_flag() {
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
        driver: crate::core::models::plant::CreatePlantDriverRequest {
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
    assert!(plant.controllers.iter().all(|controller| controller.active));
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
        driver: crate::core::models::plant::CreatePlantDriverRequest {
            plugin_id: "driver_plugin".to_string(),
            config: HashMap::new(),
        },
        controllers: vec![],
    };

    let result = PlantService::create(&store, &plugins, request);
    assert!(result.is_err());
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
    let registry_path = plant_registry_path(&plant.name);

    assert_eq!(store.count(), 1);
    assert!(registry_path.exists());

    let removed = PlantService::remove(&store, &plant.id).unwrap();
    assert_eq!(removed.name, "To Remove");
    assert_eq!(store.count(), 0);
    assert!(!registry_path.exists());
}

#[test]
fn test_close_plant_unloads_but_preserves_registry() {
    let store = PlantStore::new();
    let plugins = create_plugin_store();
    let request = create_valid_request("To Close");
    let plant = PlantService::create(&store, &plugins, request).unwrap();
    let plant = store
        .update(&plant.id, |plant| {
            plant.controllers.push(PlantController {
                id: "ctrl_close".to_string(),
                plugin_id: "controller_plugin".to_string(),
                plugin_name: "PID".to_string(),
                name: "Controller Close".to_string(),
                controller_type: "PID".to_string(),
                active: true,
                input_variable_ids: vec!["var_0".to_string()],
                output_variable_ids: vec!["var_0".to_string()],
                params: HashMap::new(),
                runtime_status: ControllerRuntimeStatus::Synced,
            });
        })
        .unwrap();
    WorkspaceService::update_plant_registry(&plant, &plant.name).unwrap();
    let registry_path = plant_registry_path(&plant.name);

    assert!(registry_path.exists());

    let closed = PlantService::close(&store, &plant.id).unwrap();
    assert_eq!(closed.name, "To Close");
    assert!(closed
        .controllers
        .iter()
        .all(|controller| !controller.active));
    assert_eq!(store.count(), 0);
    assert!(registry_path.exists());

    let registry_contents = std::fs::read_to_string(&registry_path).unwrap();
    let persisted: serde_json::Value = serde_json::from_str(&registry_contents).unwrap();
    let controllers = persisted
        .get("controllers")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    assert_eq!(controllers.len(), 1);
    assert_eq!(
        controllers[0]
            .get("active")
            .and_then(serde_json::Value::as_bool),
        Some(false)
    );
}

#[test]
fn test_remove_plant_not_found() {
    let store = PlantStore::new();
    let result = PlantService::remove(&store, "invalid_id");
    assert!(result.is_err());
}
