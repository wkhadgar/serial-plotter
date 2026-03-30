use super::*;
use crate::core::models::plugin::{
    PluginRuntime, PluginSchemaField, PluginType, SchemaFieldType, SchemaFieldValue,
};
use crate::core::services::workspace::test_workspace_root;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn test_workspace_drivers_dir() -> PathBuf {
    test_workspace_root().join("drivers")
}

fn create_valid_request() -> CreatePluginRequest {
    CreatePluginRequest {
        name: format!("test_driver_{}", Uuid::new_v4().simple()),
        plugin_type: PluginType::Driver,
        runtime: PluginRuntime::Python,
        entry_class: Some("TestDriver".to_string()),
        schema: vec![],
        source_file: None,
        source_code: Some("class TestDriver:\n    pass".to_string()),
        dependencies: vec![],
        description: Some("A test driver".to_string()),
        version: Some("1.0.0".to_string()),
        author: Some("Test Author".to_string()),
    }
}

#[test]
fn test_create_plugin_success() {
    let store = PluginStore::new();
    let request = create_valid_request();
    let expected_name = request.name.clone();
    let result = PluginService::create(&store, request);

    assert!(result.is_ok());
    let plugin = result.unwrap();

    assert!(plugin.id.starts_with("plugin_"));
    assert_eq!(plugin.name, expected_name);
    assert_eq!(plugin.source_file.as_deref(), Some("main.py"));
    assert_eq!(plugin.source_code, None);
}

#[test]
fn test_create_driver_with_list_default_persists_schema_structure() {
    let store = PluginStore::new();
    let request = CreatePluginRequest {
        schema: vec![PluginSchemaField {
            name: "channels".to_string(),
            field_type: SchemaFieldType::List,
            default_value: Some(SchemaFieldValue::List(vec![
                SchemaFieldValue::String("A0".to_string()),
                SchemaFieldValue::Int(2),
                SchemaFieldValue::Bool(true),
            ])),
            description: Some("Canais ativos".to_string()),
        }],
        ..create_valid_request()
    };

    let plugin = PluginService::create(&store, request).unwrap();

    let registry_path = test_workspace_drivers_dir()
        .join(&plugin.name)
        .join("registry.json");
    let raw_registry = fs::read_to_string(&registry_path).unwrap();
    let persisted: PluginRegistry = serde_json::from_str(&raw_registry).unwrap();

    assert_eq!(persisted.schema.len(), 1);
    assert_eq!(persisted.source_code, None);
    let default_value = persisted.schema[0].default_value.clone().unwrap();

    match default_value {
        SchemaFieldValue::List(items) => {
            assert_eq!(items.len(), 3);
            assert!(matches!(
                items.first(),
                Some(SchemaFieldValue::String(value)) if value == "A0"
            ));
            assert!(matches!(items.get(1), Some(SchemaFieldValue::Int(2))));
            assert!(matches!(items.get(2), Some(SchemaFieldValue::Bool(true))));
        }
        _ => panic!("default_value deveria ser uma lista"),
    }

    let driver_dir: PathBuf = registry_path
        .parent()
        .map_or_else(std::env::temp_dir, PathBuf::from);
    let _ = fs::remove_dir_all(driver_dir);
}

#[test]
fn test_empty_name_should_fail() {
    let store = PluginStore::new();
    let mut request = create_valid_request();

    request.name = String::new();

    assert!(PluginService::create(&store, request).is_err());
}

#[test]
fn test_driver_without_source_should_fail() {
    let store = PluginStore::new();
    let mut request = create_valid_request();

    request.source_code = None;
    request.source_file = None;

    assert!(PluginService::create(&store, request).is_err());
}

#[test]
fn test_update_plugin_success() {
    let store = PluginStore::new();
    let created = PluginService::create(&store, create_valid_request()).unwrap();

    let updated = PluginService::update(
        &store,
        UpdatePluginRequest {
            id: created.id.clone(),
            name: "updated_driver".to_string(),
            plugin_type: PluginType::Driver,
            runtime: PluginRuntime::Python,
            entry_class: Some("UpdatedDriver".to_string()),
            schema: vec![],
            source_file: Some("updated.py".to_string()),
            source_code: Some("class UpdatedDriver:\n    pass".to_string()),
            dependencies: vec![],
            description: Some("updated".to_string()),
            version: None,
            author: None,
        },
    )
    .unwrap();

    assert_eq!(updated.name, "updated_driver");
    assert_eq!(updated.source_file.as_deref(), Some("main.py"));
    assert_eq!(updated.source_code, None);
}

#[test]
fn test_update_plugin_should_fail_when_type_changes() {
    let store = PluginStore::new();
    let created = PluginService::create(&store, create_valid_request()).unwrap();

    let result = PluginService::update(
        &store,
        UpdatePluginRequest {
            id: created.id.clone(),
            name: "updated_driver".to_string(),
            plugin_type: PluginType::Controller,
            runtime: PluginRuntime::Python,
            entry_class: Some("UpdatedDriver".to_string()),
            schema: vec![],
            source_file: Some("main.py".to_string()),
            source_code: Some("class UpdatedDriver:\n    pass".to_string()),
            dependencies: vec![],
            description: Some("updated".to_string()),
            version: None,
            author: None,
        },
    );

    assert!(matches!(
        result,
        Err(AppError::InvalidArgument(message)) if message == "Tipo do plugin não pode ser alterado"
    ));
}

#[test]
fn test_get_plugin_returns_source_code_from_disk() {
    let store = PluginStore::new();
    let created = PluginService::create(&store, create_valid_request()).unwrap();

    let retrieved = PluginService::get(&store, &created.id).unwrap();
    assert_eq!(
        retrieved.source_code.as_deref(),
        Some("class TestDriver:\n    pass")
    );
}

#[test]
fn test_update_plugin_without_source_code_keeps_existing_file_contents() {
    let store = PluginStore::new();
    let created = PluginService::create(&store, create_valid_request()).unwrap();
    let updated_name = format!("updated_driver_{}", Uuid::new_v4().simple());

    let original_source_path = test_workspace_drivers_dir()
        .join(&created.name)
        .join("main.py");
    let original_source = fs::read_to_string(&original_source_path).unwrap();

    let updated = PluginService::update(
        &store,
        UpdatePluginRequest {
            id: created.id.clone(),
            name: updated_name.clone(),
            plugin_type: PluginType::Driver,
            runtime: PluginRuntime::Python,
            entry_class: Some("TestDriver".to_string()),
            schema: vec![],
            source_file: Some("main.py".to_string()),
            source_code: None,
            dependencies: vec![],
            description: Some("updated".to_string()),
            version: None,
            author: None,
        },
    )
    .unwrap();

    let updated_source_path = test_workspace_drivers_dir()
        .join(&updated_name)
        .join("main.py");
    let updated_source = fs::read_to_string(&updated_source_path).unwrap();

    assert_eq!(updated.name, updated_name);
    assert_eq!(updated_source.trim(), original_source.trim());
    assert!(!original_source_path.exists());
    assert_eq!(updated.source_code, None);
}

#[test]
fn test_remove_plugin_cleans_workspace_directory() {
    let store = PluginStore::new();
    let created = PluginService::create(&store, create_valid_request()).unwrap();

    let plugin_dir = test_workspace_drivers_dir().join(&created.name);
    assert!(plugin_dir.exists());

    let removed = PluginService::remove(&store, &created.id).unwrap();

    assert_eq!(removed.id, created.id);
    assert!(!plugin_dir.exists());
}
