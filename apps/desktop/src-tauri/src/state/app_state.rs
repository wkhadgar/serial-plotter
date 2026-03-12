use crate::state::{plant_store::PlantStore, PluginStore};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    plant_store: Arc<PlantStore>,
    plugin_store: Arc<PluginStore>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            plant_store: Arc::new(PlantStore::new()),
            plugin_store: Arc::new(PluginStore::new()),
        }
    }

    pub fn plants(&self) -> &PlantStore {
        &self.plant_store
    }

    pub fn plugins(&self) -> &PluginStore {
        &self.plugin_store
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
