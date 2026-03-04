pub struct AppState {
    pub plants: Arc<RwLock<HashMap<String, Plant>>>,
    pub sessions: Arc<RwLock<HashMap<String, SessionHandle>>>,
    pub plugin_registry: Arc<RwLock<HashMap<String, PluginInfo>>>,
}

pub struct SessionHandle {
    pub stop: tokio_util::sync::CancellationToken,
    pub paused: Arc<std::sync::atomic::AtomicBool>,
}

pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub plugin_type: PluginType,
    pub description: String,
    pub path: PathBuf,
    pub schema: PluginSchema,
}

pub enum PluginType {
    Driver,
    Controller,
}