use crate::core::services::runtime::PlantRuntimeManager;
use crate::state::{plant_store::PlantStore, PluginStore};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    plant_store: Arc<PlantStore>,
    plugin_store: Arc<PluginStore>,
    runtime_manager: Arc<PlantRuntimeManager>,
}

impl AppState {
    pub fn new() -> Self {
        let plant_store = Arc::new(PlantStore::new());
        let plugin_store = Arc::new(PluginStore::new());
        let runtime_manager = Arc::new(PlantRuntimeManager::new(plant_store.clone()));

        Self {
            plant_store,
            plugin_store,
            runtime_manager,
        }
    }

    pub fn plants(&self) -> &PlantStore {
        &self.plant_store
    }

    pub fn plugins(&self) -> &PluginStore {
        &self.plugin_store
    }

    pub fn runtimes(&self) -> &PlantRuntimeManager {
        &self.runtime_manager
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
