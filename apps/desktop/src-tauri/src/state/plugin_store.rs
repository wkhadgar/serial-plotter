use crate::core::error::{AppError, AppResult};
use crate::core::models::plugin::PluginRegistry;
use crate::state::normalized_name_key;
use parking_lot::RwLock;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct PluginStore {
    state: RwLock<PluginStoreState>,
}

#[derive(Debug, Default)]
struct PluginStoreState {
    registries: HashMap<String, PluginRegistry>,
    names: HashMap<String, String>,
}

impl PluginStore {
    pub fn new() -> Self {
        Self {
            state: RwLock::new(PluginStoreState::default()),
        }
    }

    pub fn insert(&self, registry: PluginRegistry) -> AppResult<()> {
        let mut state = self.state.write();
        Self::insert_into_state(&mut state, registry)?;
        Ok(())
    }

    pub fn get(&self, id: &str) -> AppResult<PluginRegistry> {
        self.state
            .read()
            .registries
            .get(id)
            .cloned()
            .ok_or_else(|| AppError::NotFound(format!("Plugin '{}' não encontrado", id)))
    }

    pub fn list(&self) -> Vec<PluginRegistry> {
        self.state.read().registries.values().cloned().collect()
    }

    pub fn list_by_type(
        &self,
        plugin_type: crate::core::models::plugin::PluginType,
    ) -> Vec<PluginRegistry> {
        self.state
            .read()
            .registries
            .values()
            .filter(|p| p.plugin_type == plugin_type)
            .cloned()
            .collect()
    }

    pub fn replace(&self, id: &str, registry: PluginRegistry) -> AppResult<()> {
        let mut state = self.state.write();
        let previous_name_key = {
            let current = state
                .registries
                .get(id)
                .ok_or_else(|| AppError::NotFound(format!("Plugin '{}' não encontrado", id)))?;
            Self::name_key(&current.name)
        };
        let next_name_key = Self::name_key(&registry.name);

        if next_name_key.is_empty() {
            return Err(AppError::InvalidArgument(
                "Nome do plugin é obrigatório".into(),
            ));
        }

        if let Some(existing_id) = state.names.get(&next_name_key) {
            if existing_id != id {
                return Err(AppError::InvalidArgument(format!(
                    "Plugin com nome '{}' já existe",
                    registry.name.trim()
                )));
            }
        }

        state.names.remove(&previous_name_key);
        state.names.insert(next_name_key, registry.id.clone());
        state.registries.insert(id.to_string(), registry);
        Ok(())
    }

    pub fn remove(&self, id: &str) -> AppResult<PluginRegistry> {
        let mut state = self.state.write();
        let registry = state
            .registries
            .remove(id)
            .ok_or_else(|| AppError::NotFound(format!("Plugin '{}' não encontrado", id)))?;
        state.names.remove(&Self::name_key(&registry.name));
        Ok(registry)
    }

    pub fn sync(&self, registries: Vec<PluginRegistry>) -> AppResult<Vec<PluginRegistry>> {
        let mut next_state = PluginStoreState::default();
        for registry in registries {
            Self::insert_into_state(&mut next_state, registry)?;
        }

        let items = next_state.registries.values().cloned().collect::<Vec<_>>();
        *self.state.write() = next_state;
        Ok(items)
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

    fn insert_into_state(state: &mut PluginStoreState, registry: PluginRegistry) -> AppResult<()> {
        if state.registries.contains_key(&registry.id) {
            return Err(AppError::InvalidArgument(format!(
                "Plugin com ID {} já existe!",
                registry.id
            )));
        }

        let name_key = Self::name_key(&registry.name);
        if name_key.is_empty() {
            return Err(AppError::InvalidArgument(
                "Nome do plugin é obrigatório".into(),
            ));
        }

        if state.names.contains_key(&name_key) {
            return Err(AppError::InvalidArgument(format!(
                "Plugin com nome '{}' já existe",
                registry.name.trim()
            )));
        }

        state.names.insert(name_key, registry.id.clone());
        state.registries.insert(registry.id.clone(), registry);
        Ok(())
    }

    fn name_key(name: &str) -> String {
        normalized_name_key(name)
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
            entry_class: "TestDriver".to_string(),
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

    #[test]
    fn test_registry_duplicate_name_ignores_case_and_whitespace() {
        let store = PluginStore::new();

        assert!(store
            .insert(create_plugin_test("plugin_test_1", "Driver Base", None))
            .is_ok());
        assert!(store
            .insert(create_plugin_test("plugin_test_2", "  driver base  ", None))
            .is_err());
    }
}
