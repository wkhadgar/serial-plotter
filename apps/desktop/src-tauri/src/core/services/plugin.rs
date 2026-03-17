use crate::core::error::{AppError, AppResult};
use crate::core::models::plugin::{
    CreatePluginRequest, PluginRegistry, PluginRuntime, PluginType, UpdatePluginRequest,
};
use crate::core::services::workspace::WorkspaceService;
use crate::state::PluginStore;
use uuid::Uuid;

const PYTHON_SOURCE_FILE_NAME: &str = "main.py";

pub struct PluginService;

#[derive(Debug, Clone)]
struct ExistingPluginMeta {
    name: String,
    plugin_type: PluginType,
}

impl PluginService {
    pub fn create(store: &PluginStore, request: CreatePluginRequest) -> AppResult<PluginRegistry> {
        Self::validate_request(
            store,
            None,
            &request.name,
            request.runtime,
            request.source_file.as_deref(),
            request.source_code.as_deref(),
            true,
        )?;

        let source_code = Self::normalize_source_code(request.source_code.as_deref())
            .ok_or_else(|| AppError::InvalidArgument("Código fonte Python é obrigatório".into()))?;
        let plugin = Self::build_plugin(request);

        WorkspaceService::save_plugin_registry(&plugin, &source_code)?;

        if let Err(error) = store.insert(plugin.clone()) {
            let _ = WorkspaceService::delete_plugin_registry(&plugin.name, plugin.plugin_type);
            return Err(error);
        }

        Ok(plugin)
    }

    pub fn get(store: &PluginStore, id: &str) -> AppResult<PluginRegistry> {
        let mut plugin = store.get(id)?;
        let source_code = WorkspaceService::read_plugin_source(&plugin.name, plugin.plugin_type)?;
        plugin.source_code = Some(source_code);
        Ok(plugin)
    }

    pub fn list(store: &PluginStore) -> Vec<PluginRegistry> {
        store.list()
    }

    pub fn list_by_type(store: &PluginStore, plugin_type: PluginType) -> Vec<PluginRegistry> {
        store.list_by_type(plugin_type)
    }

    pub fn update(store: &PluginStore, request: UpdatePluginRequest) -> AppResult<PluginRegistry> {
        Self::validate_request(
            store,
            Some(request.id.as_str()),
            &request.name,
            request.runtime,
            request.source_file.as_deref(),
            request.source_code.as_deref(),
            false,
        )?;

        let existing = store.with_registry(&request.id, |plugin| ExistingPluginMeta {
            name: plugin.name.clone(),
            plugin_type: plugin.plugin_type,
        })?;

        if request.plugin_type != existing.plugin_type {
            return Err(AppError::InvalidArgument(
                "Tipo do plugin não pode ser alterado".into(),
            ));
        }

        let resolved_source_code = match Self::normalize_source_code(request.source_code.as_deref())
        {
            Some(code) => code,
            None => WorkspaceService::read_plugin_source(&existing.name, existing.plugin_type)?,
        };

        let next_plugin = PluginRegistry {
            id: request.id.clone(),
            name: request.name.trim().to_string(),
            plugin_type: existing.plugin_type,
            runtime: request.runtime,
            schema: request.schema,
            source_file: Some(PYTHON_SOURCE_FILE_NAME.to_string()),
            source_code: None,
            dependencies: request.dependencies,
            description: Self::normalize_optional_text(request.description),
            version: Self::normalize_optional_text(request.version),
            author: Self::normalize_optional_text(request.author),
        };

        WorkspaceService::update_plugin_registry(
            &next_plugin,
            resolved_source_code.as_str(),
            &existing.name,
            existing.plugin_type,
        )?;
        store.replace(&request.id, next_plugin.clone())?;

        Ok(next_plugin)
    }

    fn build_plugin(request: CreatePluginRequest) -> PluginRegistry {
        let plugin_id = format!("plugin_{}", Uuid::new_v4());

        PluginRegistry {
            id: plugin_id,
            name: request.name.trim().to_string(),
            plugin_type: request.plugin_type,
            runtime: request.runtime,
            schema: request.schema,
            source_file: Some(PYTHON_SOURCE_FILE_NAME.to_string()),
            source_code: None,
            dependencies: request.dependencies,
            description: Self::normalize_optional_text(request.description),
            version: Self::normalize_optional_text(request.version),
            author: Self::normalize_optional_text(request.author),
        }
    }

    fn validate_request(
        store: &PluginStore,
        current_id: Option<&str>,
        name: &str,
        runtime: PluginRuntime,
        source_file: Option<&str>,
        source_code: Option<&str>,
        require_source_code: bool,
    ) -> AppResult<()> {
        if name.trim().is_empty() {
            return Err(AppError::InvalidArgument(
                "Nome do plugin é obrigatório".into(),
            ));
        }

        let has_duplicate_name = current_id
            .map(|id| store.exists_by_name_except(id, name))
            .unwrap_or_else(|| store.exists_by_name(name));

        if has_duplicate_name {
            return Err(AppError::InvalidArgument(format!(
                "Plugin com nome '{}' já existe",
                name.trim()
            )));
        }

        if runtime != PluginRuntime::Python {
            return Err(AppError::InvalidArgument(
                "Somente plugins Python podem ser criados no momento".into(),
            ));
        }

        if require_source_code
            && source_code
                .map(|code| code.trim().is_empty())
                .unwrap_or(true)
        {
            return Err(AppError::InvalidArgument(
                "Código fonte Python é obrigatório".into(),
            ));
        }

        if source_code.is_some_and(|code| code.trim().is_empty()) {
            return Err(AppError::InvalidArgument(
                "Código fonte Python é obrigatório".into(),
            ));
        }

        if source_file.is_some_and(|file_name| file_name.trim().is_empty()) {
            return Err(AppError::InvalidArgument(
                "Nome do arquivo fonte inválido".into(),
            ));
        }

        Ok(())
    }

    fn normalize_source_code(source_code: Option<&str>) -> Option<String> {
        source_code.and_then(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
    }

    fn normalize_optional_text(value: Option<String>) -> Option<String> {
        value.and_then(|raw| {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::plugin::{
        PluginRuntime, PluginSchemaField, PluginType, SchemaFieldType, SchemaFieldValue,
    };
    use std::fs;
    use std::path::PathBuf;
    use uuid::Uuid;

    fn create_valid_request() -> CreatePluginRequest {
        CreatePluginRequest {
            name: format!("test_driver_{}", Uuid::new_v4().simple()),
            plugin_type: PluginType::Driver,
            runtime: PluginRuntime::Python,
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

        let registry_path = std::env::temp_dir()
            .join("Senamby/workspace")
            .join("drivers")
            .join(&plugin.name)
            .join("registry.json");
        let raw_registry = fs::read_to_string(&registry_path).unwrap();
        let persisted: PluginRegistry = serde_json::from_str(&raw_registry).unwrap();

        assert_eq!(persisted.schema.len(), 1);
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
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::temp_dir());
        let _ = fs::remove_dir_all(driver_dir);
    }

    #[test]
    fn test_empty_name_should_fail() {
        let store = PluginStore::new();
        let mut request = create_valid_request();

        request.name = "".to_string();

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
            Err(AppError::InvalidArgument(message))
            if message == "Tipo do plugin não pode ser alterado"
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

        let original_source_path = std::env::temp_dir()
            .join("Senamby/workspace")
            .join("drivers")
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

        let updated_source_path = std::env::temp_dir()
            .join("Senamby/workspace")
            .join("drivers")
            .join(&updated_name)
            .join("main.py");
        let updated_source = fs::read_to_string(&updated_source_path).unwrap();

        assert_eq!(updated.name, updated_name);
        assert_eq!(updated_source.trim(), original_source.trim());
        assert!(!original_source_path.exists());
        assert_eq!(updated.source_code, None);
    }
}
