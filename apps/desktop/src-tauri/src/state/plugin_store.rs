use crate::core::error::{AppError, AppResult};

use crate::core::models::plugin::PluginInstance;
use crate::core::models::plugin::PluginRegistry;

use parking_lot::RwLock;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct PluginStore {
    registries: RwLock<HashMap<String, PluginRegistry>>,
    instances: RwLock<HashMap<String, PluginInstance>>,
}

impl PluginStore {
    pub fn new() -> Self {
        Self {
            registries: RwLock::new(HashMap::new()),
            instances: RwLock::new(HashMap::new()),
        }
    }

    pub fn insert(&self, registry: PluginRegistry) -> AppResult<()> {
        let mut plugins = self.registries.write();

        if plugins.contains_key(&registry.id) {
            return Err(AppError::InvalidArgument(format!(
                "Plugin com ID {} já existe!",
                registry.id
            )));
        }

        plugins.insert(registry.id.clone(), registry);
        Ok(())
    }

    pub fn get(&self, id: &str) -> AppResult<PluginRegistry> {
        let plugins = self.registries.read();

        plugins
            .get(id)
            .cloned()
            .ok_or_else(|| AppError::NotFound(format!("Plugin '{}' não encontrado", id)))
    }

    pub fn list(&self) -> Vec<PluginRegistry> {
        self.registries.read().values().cloned().collect()
    }

    pub fn list_by_type(&self, plugin_type: crate::core::models::plugin::PluginType) -> Vec<PluginRegistry> {
        self.registries
            .read()
            .values()
            .filter(|p| p.plugin_type == plugin_type)
            .cloned()
            .collect()
    }

    pub fn update<F>(&self, id: &str, updater: F) -> AppResult<PluginRegistry>
    where
        F: FnOnce(&mut PluginRegistry),
    {
        let mut plugins = self.registries.write();

        let plugin = plugins
            .get_mut(id)
            .ok_or_else(|| AppError::NotFound(format!("Plugin '{}' não encontrado", id)))?;

        updater(plugin);
        Ok(plugin.clone())
    }

    pub fn exists_by_name(&self, name: &str) -> bool {
        self.registries
            .read()
            .values()
            .any(|plugin| plugin.name.eq_ignore_ascii_case(name))
    }

    pub fn exists_by_name_except(&self, id: &str, name: &str) -> bool {
        self.registries
            .read()
            .values()
            .any(|plugin| plugin.id != id && plugin.name.eq_ignore_ascii_case(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::plugin::{PluginRegistry, PluginRuntime, PluginType};

    fn create_plugin_test(id: &str, name: &str, description: Option<&str>) -> PluginRegistry {
        PluginRegistry {
            id: id.to_string(),
            name: name.to_string(),
            plugin_type: PluginType::Driver,
            runtime: PluginRuntime::Python,
            schema: vec![],
            source_file: None,
            source_code: None,
            dependencies: vec![],
            description: description.map(str::to_string),
            version: None,
            author: None,
        }
    }

    #[test]
    fn test_registry_insert() {
        let store = PluginStore::new();

        let plugin1_registry =
            create_plugin_test("plugin_test_1", "Plugin 1", Some("Plugin Description"));
        let plugin2_registry = create_plugin_test("plugin_test_2", "Plugin 2", None);

        assert!(store.insert(plugin1_registry).is_ok());
        assert!(store.insert(plugin2_registry).is_ok());

        let retrieved_plugin1 = store.get("plugin_test_1").unwrap();
        let retrieved_plugin2 = store.get("plugin_test_2").unwrap();

        assert_eq!(retrieved_plugin1.name, "Plugin 1");
        assert_eq!(retrieved_plugin2.name, "Plugin 2");
        assert_eq!(retrieved_plugin2.description, None);
    }

    #[test]
    fn test_registry_duplicated_id() {
        let store = PluginStore::new();

        let plugin1_registry =
            create_plugin_test("plugin_test_1", "Plugin 1", Some("Plugin Description"));
        let plugin2_registry = create_plugin_test("plugin_test_1", "Plugin 2", None);

        assert!(store.insert(plugin1_registry).is_ok());
        assert!(store.insert(plugin2_registry).is_err());
    }
}
