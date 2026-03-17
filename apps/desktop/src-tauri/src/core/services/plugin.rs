use crate::core::error::{AppError, AppResult};
use crate::core::models::plugin::{
    CreatePluginRequest, PluginRegistry, PluginRuntime, PluginType, UpdatePluginRequest,
};
use crate::core::services::workspace::WorkspaceService;
use crate::state::PluginStore;
use uuid::Uuid;

const PYTHON_SOURCE_FILE_NAME: &str = "main.py";

pub struct PluginService;

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

        let source_code = request
            .source_code
            .as_ref()
            .map(|value| value.trim().to_string())
            .unwrap_or_default();
        let plugin = Self::build_plugin(request);

        if plugin.plugin_type == PluginType::Driver {
            WorkspaceService::save_driver_registry(&plugin, &source_code)?;
        }

        store.insert(plugin.clone())?;

        Ok(plugin)
    }

    pub fn get(store: &PluginStore, id: &str) -> AppResult<PluginRegistry> {
        store.get(id)
    }

    pub fn list(store: &PluginStore) -> Vec<PluginRegistry> {
        store.list()
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

        let source_code = request
            .source_code
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        let updated = store.update(&request.id, |plugin| {
            plugin.name = request.name.trim().to_string();
            plugin.plugin_type = request.plugin_type;
            plugin.runtime = request.runtime;
            plugin.schema = request.schema;
            plugin.source_file = Some(PYTHON_SOURCE_FILE_NAME.to_string());
            plugin.source_code = None;
            plugin.dependencies = request.dependencies;
            plugin.description = request.description.map(|value| value.trim().to_string());
            plugin.version = request.version.map(|value| value.trim().to_string());
            plugin.author = request.author.map(|value| value.trim().to_string());
        })?;

        if updated.plugin_type == PluginType::Driver {
            if let Some(code) = source_code.as_deref() {
                WorkspaceService::save_driver_registry(&updated, code)?;
            }
        }

        Ok(updated)
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
            description: request.description.map(|value| value.trim().to_string()),
            version: request.version.map(|value| value.trim().to_string()),
            author: request.author.map(|value| value.trim().to_string()),
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::plugin::{PluginRuntime, PluginType};

    fn create_valid_request() -> CreatePluginRequest {
        CreatePluginRequest {
            name: "test_driver".to_string(),
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
        let result = PluginService::create(&store, request);

        assert!(result.is_ok());
        let plugin = result.unwrap();

        assert!(plugin.id.starts_with("plugin_"));
        assert_eq!(plugin.name, "test_driver");
        assert_eq!(plugin.source_file.as_deref(), Some("main.py"));
        assert_eq!(plugin.source_code, None);
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
}
