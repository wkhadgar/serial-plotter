use crate::core::error::{AppError, AppResult};
use crate::core::models::plugin::{CreatePluginRequest, PluginRegistry};
use crate::state::PluginStore;
use uuid::Uuid;

pub struct PluginService;

impl PluginService {
    pub fn create(store: &PluginStore, request: CreatePluginRequest) -> AppResult<PluginRegistry> {
        Self::validate_create_request(&request)?;

        let plugin = Self::build_plugin(request);
        store.insert(plugin.clone())?;

        Ok(plugin)
    }

    fn build_plugin(request: CreatePluginRequest) -> PluginRegistry {
        let plugin_id = format!("plugin_{}", Uuid::new_v4());

        PluginRegistry {
            id: plugin_id,
            name: request.name,
            plugin_type: request.plugin_type,
            runtime: request.runtime,
            schema: request.schema,
            source_file: request.source_file,
            source_code: request.source_code,
            dependencies: request.dependencies,
            description: request.description,
            version: request.version,
            author: request.author,
        }
    }

    fn validate_create_request(request: &CreatePluginRequest) -> AppResult<()> {
        if request.name.trim().is_empty() {
            return Err(AppError::InvalidArgument(
                "Nome da planta é obrigatório".into(),
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
    }

    #[test]
    #[should_panic(expected = "ValidationError")]
    fn test_empty_name_should_fail() {
        let store = PluginStore::new();
        let mut request = create_valid_request();

        request.name = "".to_string();

        PluginService::create(&store, request).expect("ValidationError");
    }

    #[test]
    #[should_panic(expected = "ValidationError")]
    fn test_driver_without_source_should_fail() {
        let store = PluginStore::new();
        let mut request = create_valid_request();

        request.source_code = None;
        request.source_file = None;

        PluginService::create(&store, request).expect("ValidationError");
    }
}
