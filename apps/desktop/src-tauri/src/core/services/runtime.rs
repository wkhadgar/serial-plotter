mod bootstrap;
mod environment;
mod process;
mod validation;

use self::bootstrap::{
    build_bootstrap_payload, build_runtime_controller_payloads, collect_runtime_setpoints,
    resolve_plugin_for_runtime, resolve_runtime_components_for_connect,
};
use self::environment::{
    collect_runtime_plugins, compute_env_hash, ensure_python_env, prepare_runtime_directory,
    prepare_runtime_scaffold, write_bootstrap_files, write_runner_script,
};
use self::process::{
    emit_status_event, send_command, spawn_driver_process, spawn_stderr_task, spawn_stdout_task,
    wait_for_handshake,
};
use self::validation::{
    ensure_driver_supports_write, validate_controller_plugin_source,
    validate_plugin_workspace_files, validate_python_source_file,
};
use crate::core::error::{AppError, AppResult};
use crate::core::models::plant::{
    Plant, PlantController, RemovePlantControllerRequest, SavePlantControllerConfigRequest,
    SavePlantSetpointRequest,
};
use crate::core::models::plugin::{PluginRegistry, PluginRuntime, PluginType};
use crate::state::{PlantStore, PluginStore};
use parking_lot::{Condvar, Mutex};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::{Child, ChildStdin};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tauri::AppHandle;
use uuid::Uuid;

const RUNNER_SCRIPT: &str = include_str!("../../../runtime/python/runner.py");
const PYTHON_SYNTAX_CHECK_SCRIPT: &str = r#"
import sys
import tokenize

path = sys.argv[1]
with tokenize.open(path) as handle:
    compile(handle.read(), path, "exec")
"#;
const PYTHON_CLASS_METHOD_CHECK_SCRIPT: &str = r#"
import ast
import json
import sys
import tokenize

path = sys.argv[1]
class_name = sys.argv[2]
required_methods = json.loads(sys.argv[3])

with tokenize.open(path) as handle:
    tree = ast.parse(handle.read(), path)

target_class = None
for node in tree.body:
    if isinstance(node, ast.ClassDef) and node.name == class_name:
        target_class = node
        break

if target_class is None:
    print(f"Classe '{class_name}' não encontrada em '{path}'", file=sys.stderr)
    sys.exit(2)

declared_methods = {
    node.name
    for node in target_class.body
    if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
}

missing = [method for method in required_methods if method not in declared_methods]
if missing:
    print(
        f"Classe '{class_name}' não implementa os métodos obrigatórios: {', '.join(missing)}",
        file=sys.stderr,
    )
    sys.exit(3)
"#;
const STARTUP_TIMEOUT: Duration = Duration::from_secs(12);
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(4);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeLifecycleState {
    Created,
    Bootstrapping,
    Ready,
    Connecting,
    Running,
    Stopping,
    Stopped,
    Faulted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeCyclePhase {
    CycleStarted,
    ReadInputs,
    ComputeControllers,
    WriteOutputs,
    PublishTelemetry,
    SleepUntilDeadline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStatusEvent {
    pub plant_id: String,
    pub runtime_id: String,
    pub lifecycle_state: RuntimeLifecycleState,
    pub cycle_phase: RuntimeCyclePhase,
    pub configured_sample_time_ms: u64,
    pub effective_dt_ms: f64,
    pub cycle_late: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeTelemetryEvent {
    pub plant_id: String,
    pub runtime_id: String,
    pub lifecycle_state: RuntimeLifecycleState,
    pub cycle_phase: RuntimeCyclePhase,
    pub configured_sample_time_ms: u64,
    pub effective_dt_ms: f64,
    pub cycle_late: bool,
    pub payload: Value,
}

#[derive(Debug, Default, Clone)]
struct RuntimeMetrics {
    lifecycle_state: RuntimeLifecycleState,
    cycle_phase: RuntimeCyclePhase,
    effective_dt_ms: f64,
    cycle_duration_ms: f64,
    read_duration_ms: f64,
    cycle_late: bool,
    late_cycle_count: u64,
    last_telemetry_at: Option<u64>,
}

impl Default for RuntimeLifecycleState {
    fn default() -> Self {
        Self::Created
    }
}

impl Default for RuntimeCyclePhase {
    fn default() -> Self {
        Self::CycleStarted
    }
}

#[derive(Debug, Default)]
struct HandshakeState {
    ready: bool,
    connected: bool,
    error: Option<String>,
}

type SharedHandshake = Arc<(Mutex<HandshakeState>, Condvar)>;
type SharedMetrics = Arc<Mutex<RuntimeMetrics>>;

#[derive(Debug, Deserialize)]
struct RuntimeEnvelope {
    #[serde(rename = "type")]
    msg_type: String,
    #[serde(default)]
    payload: Value,
}

#[derive(Debug, Serialize, Clone)]
struct DriverBootstrapIoGroup {
    ids: Vec<String>,
    count: usize,
    variables: Vec<Value>,
    variables_by_id: HashMap<String, Value>,
}

#[derive(Debug, Serialize, Clone)]
struct DriverBootstrapPlant {
    id: String,
    name: String,
    variables: Vec<Value>,
    variables_by_id: HashMap<String, Value>,
    sensors: DriverBootstrapIoGroup,
    actuators: DriverBootstrapIoGroup,
    setpoints: HashMap<String, f64>,
}

#[derive(Debug, Serialize, Clone)]
struct DriverBootstrapRuntimeTiming {
    owner: &'static str,
    clock: &'static str,
    strategy: &'static str,
    sample_time_ms: u64,
}

#[derive(Debug, Serialize, Clone)]
struct DriverBootstrapRuntimeSupervision {
    owner: &'static str,
    startup_timeout_ms: u64,
    shutdown_timeout_ms: u64,
}

#[derive(Debug, Serialize, Clone)]
struct DriverBootstrapRuntimePaths {
    runtime_dir: String,
    venv_python_path: String,
    runner_path: String,
    bootstrap_path: String,
}

#[derive(Debug, Serialize, Clone)]
struct DriverBootstrapRuntime {
    id: String,
    timing: DriverBootstrapRuntimeTiming,
    supervision: DriverBootstrapRuntimeSupervision,
    paths: DriverBootstrapRuntimePaths,
}

#[derive(Debug, Serialize, Clone)]
struct DriverBootstrapDriver {
    plugin_id: String,
    plugin_name: String,
    plugin_dir: String,
    source_file: String,
    class_name: String,
    config: Value,
}

#[derive(Debug, Serialize, Clone)]
struct DriverBootstrapController {
    id: String,
    plugin_id: String,
    plugin_name: String,
    plugin_dir: String,
    source_file: String,
    class_name: String,
    name: String,
    controller_type: String,
    active: bool,
    input_variable_ids: Vec<String>,
    output_variable_ids: Vec<String>,
    params: Value,
}

#[derive(Debug, Serialize, Clone)]
struct DriverBootstrapPayload {
    driver: DriverBootstrapDriver,
    controllers: Vec<DriverBootstrapController>,
    plant: DriverBootstrapPlant,
    runtime: DriverBootstrapRuntime,
}

#[derive(Debug, Clone)]
struct ResolvedRuntimeController {
    instance: PlantController,
    plugin: PluginRegistry,
    plugin_dir: PathBuf,
}

#[derive(Debug)]
struct RuntimeHandle {
    plant_id: String,
    runtime_id: String,
    runtime_dir: PathBuf,
    configured_sample_time_ms: u64,
    stdin: Arc<Mutex<ChildStdin>>,
    child: Child,
    stdout_task: Option<thread::JoinHandle<()>>,
    stderr_task: Option<thread::JoinHandle<()>>,
    metrics: SharedMetrics,
}

#[derive(Debug)]
pub struct PlantRuntimeManager {
    handles: Mutex<HashMap<String, RuntimeHandle>>,
    plant_store: Arc<PlantStore>,
}

impl PlantRuntimeManager {
    pub fn new(plant_store: Arc<PlantStore>) -> Self {
        Self {
            handles: Mutex::new(HashMap::new()),
            plant_store,
        }
    }

    fn start_runtime(
        &self,
        app: &AppHandle,
        plant: &Plant,
        driver_plugin: &PluginRegistry,
        active_controllers: &[ResolvedRuntimeController],
        runtime_plugins: &[PluginRegistry],
    ) -> AppResult<()> {
        if self.handles.lock().contains_key(&plant.id) {
            return Err(AppError::InvalidArgument(format!(
                "Planta '{}' já está em execução",
                plant.id
            )));
        }

        if driver_plugin.plugin_type != PluginType::Driver {
            return Err(AppError::InvalidArgument(
                "Plugin selecionado não é um driver".into(),
            ));
        }

        if driver_plugin.runtime != PluginRuntime::Python {
            return Err(AppError::InvalidArgument(
                "A runtime de driver atual suporta apenas plugins Python".into(),
            ));
        }

        let runtime_id = format!("rt_{}", Uuid::new_v4().simple());
        let runtime_plugins = collect_runtime_plugins(runtime_plugins);
        let env_hash = compute_env_hash(&runtime_plugins);
        let venv_python_path = ensure_python_env(&runtime_plugins, &env_hash)?;

        let driver_dir = crate::core::services::workspace::WorkspaceService::plugin_directory(
            &driver_plugin.name,
            PluginType::Driver,
        )?;
        validate_plugin_workspace_files(&driver_dir, "driver")?;
        validate_python_source_file(&venv_python_path, &driver_dir.join("main.py"), "driver")?;
        if !active_controllers.is_empty() {
            ensure_driver_supports_write(driver_plugin)?;
        }
        for controller in active_controllers {
            validate_plugin_workspace_files(&controller.plugin_dir, "controlador")?;
            validate_python_source_file(
                &venv_python_path,
                &controller.plugin_dir.join("main.py"),
                &format!("controlador '{}'", controller.instance.name),
            )?;
        }

        let runtime_root = prepare_runtime_directory()?;
        let runtime_dir = runtime_root.join(&runtime_id);
        prepare_runtime_scaffold(&runtime_dir)?;

        let startup_result = (|| -> AppResult<RuntimeHandle> {
            let runner_path = write_runner_script(&runtime_dir)?;
            let bootstrap_path = runtime_dir.join("bootstrap.json");
            let bootstrap = build_bootstrap_payload(
                &runtime_id,
                plant,
                driver_plugin,
                &driver_dir,
                active_controllers,
                &runtime_dir,
                &venv_python_path,
                &runner_path,
                &bootstrap_path,
                STARTUP_TIMEOUT.as_millis() as u64,
                SHUTDOWN_TIMEOUT.as_millis() as u64,
            )?;
            write_bootstrap_files(&runtime_dir, &bootstrap, &bootstrap_path)?;

            let mut child = spawn_driver_process(
                &venv_python_path,
                &runner_path,
                &runtime_dir,
                &bootstrap_path,
            )?;

            let stdin = child.stdin.take().ok_or(AppError::InternalError)?;
            let stdout = child.stdout.take().ok_or(AppError::InternalError)?;
            let stderr = child.stderr.take().ok_or(AppError::InternalError)?;

            let shared_stdin = Arc::new(Mutex::new(stdin));
            let metrics = Arc::new(Mutex::new(RuntimeMetrics {
                lifecycle_state: RuntimeLifecycleState::Bootstrapping,
                cycle_phase: RuntimeCyclePhase::CycleStarted,
                ..RuntimeMetrics::default()
            }));
            let handshake = Arc::new((Mutex::new(HandshakeState::default()), Condvar::new()));

            let stdout_task = spawn_stdout_task(
                app.clone(),
                self.plant_store.clone(),
                plant.id.clone(),
                runtime_id.clone(),
                plant.sample_time_ms,
                stdout,
                handshake.clone(),
                metrics.clone(),
            );
            let stderr_task = spawn_stderr_task(plant.id.clone(), runtime_id.clone(), stderr);

            let startup = (|| -> AppResult<()> {
                send_command(
                    &shared_stdin,
                    "init",
                    Some(serde_json::to_value(&bootstrap).map_err(|error| {
                        AppError::IoError(format!("Falha ao serializar payload init: {error}"))
                    })?),
                )?;
                send_command(&shared_stdin, "start", None)?;
                wait_for_handshake(&handshake, STARTUP_TIMEOUT)
            })();

            if let Err(error) = startup {
                let _ = send_command(&shared_stdin, "shutdown", None);
                let _ = child.kill();
                let _ = child.wait();
                let _ = stdout_task.join();
                let _ = stderr_task.join();
                return Err(error);
            }

            Ok(RuntimeHandle {
                plant_id: plant.id.clone(),
                runtime_id: runtime_id.clone(),
                runtime_dir: runtime_dir.clone(),
                configured_sample_time_ms: plant.sample_time_ms,
                stdin: shared_stdin,
                child,
                stdout_task: Some(stdout_task),
                stderr_task: Some(stderr_task),
                metrics,
            })
        })();

        let handle = match startup_result {
            Ok(handle) => handle,
            Err(error) => {
                let _ = fs::remove_dir_all(&runtime_dir);
                return Err(error);
            }
        };

        self.handles.lock().insert(plant.id.clone(), handle);
        Ok(())
    }

    pub fn stop_runtime(&self, app: &AppHandle, plant_id: &str) -> AppResult<()> {
        let handle = {
            let mut handles = self.handles.lock();
            handles.remove(plant_id)
        };

        let mut handle = match handle {
            Some(handle) => handle,
            None => return Ok(()),
        };

        let _ = send_command(&handle.stdin, "shutdown", None);

        let started_wait = std::time::Instant::now();
        loop {
            match handle.child.try_wait() {
                Ok(Some(_status)) => break,
                Ok(None) => {
                    if started_wait.elapsed() > SHUTDOWN_TIMEOUT {
                        let _ = handle.child.kill();
                        let _ = handle.child.wait();
                        break;
                    }
                    thread::sleep(Duration::from_millis(100));
                }
                Err(_) => break,
            }
        }

        if let Some(task) = handle.stdout_task.take() {
            let _ = task.join();
        }
        if let Some(task) = handle.stderr_task.take() {
            let _ = task.join();
        }

        let _ = fs::remove_dir_all(&handle.runtime_dir);

        emit_status_event(
            app,
            RuntimeStatusEvent {
                plant_id: handle.plant_id,
                runtime_id: handle.runtime_id,
                lifecycle_state: RuntimeLifecycleState::Stopped,
                cycle_phase: RuntimeCyclePhase::SleepUntilDeadline,
                configured_sample_time_ms: handle.configured_sample_time_ms,
                effective_dt_ms: handle.metrics.lock().effective_dt_ms,
                cycle_late: false,
            },
        );

        Ok(())
    }

    pub fn pause_runtime(&self, plant_id: &str) -> AppResult<()> {
        self.send_runtime_command(plant_id, "pause")
    }

    pub fn resume_runtime(&self, plant_id: &str) -> AppResult<()> {
        self.send_runtime_command(plant_id, "resume")
    }

    pub fn update_setpoints(
        &self,
        plant_id: &str,
        setpoints: HashMap<String, f64>,
    ) -> AppResult<()> {
        let payload = serde_json::to_value(json!({ "setpoints": setpoints })).map_err(|error| {
            AppError::IoError(format!(
                "Falha ao serializar atualização de setpoints: {error}"
            ))
        })?;
        self.send_runtime_command_with_payload(plant_id, "update_setpoints", payload)
    }

    fn update_controllers(
        &self,
        plant_id: &str,
        controllers: Vec<DriverBootstrapController>,
    ) -> AppResult<()> {
        let payload =
            serde_json::to_value(json!({ "controllers": controllers })).map_err(|error| {
                AppError::IoError(format!(
                    "Falha ao serializar atualização de controladores: {error}"
                ))
            })?;
        self.send_runtime_command_with_payload(plant_id, "update_controllers", payload)
    }

    fn send_runtime_command(&self, plant_id: &str, msg_type: &str) -> AppResult<()> {
        self.send_runtime_command_with_payload(plant_id, msg_type, Value::Null)
    }

    fn send_runtime_command_with_payload(
        &self,
        plant_id: &str,
        msg_type: &str,
        payload: Value,
    ) -> AppResult<()> {
        let stdin = {
            let handles = self.handles.lock();
            let handle = handles.get(plant_id).ok_or_else(|| {
                AppError::NotFound(format!(
                    "Runtime da planta '{}' não está em execução",
                    plant_id
                ))
            })?;
            handle.stdin.clone()
        };

        let payload = if payload.is_null() {
            None
        } else {
            Some(payload)
        };
        send_command(&stdin, msg_type, payload)
    }
}

pub struct DriverRuntimeService;

impl DriverRuntimeService {
    pub fn connect(
        app: &AppHandle,
        plants: &PlantStore,
        plugins: &PluginStore,
        manager: &PlantRuntimeManager,
        plant_id: &str,
    ) -> AppResult<Plant> {
        let plant = plants.get(plant_id)?;

        if plant.connected {
            return Err(AppError::InvalidArgument(
                "Planta já está conectada".to_string(),
            ));
        }

        let (plant, driver, active_controllers, runtime_plugins) =
            resolve_runtime_components_for_connect(plants, plugins, plant)?;
        manager.start_runtime(app, &plant, &driver, &active_controllers, &runtime_plugins)?;

        plants.update(plant_id, |plant| {
            plant.connected = true;
            plant.paused = false;
        })
    }

    pub fn disconnect(
        app: &AppHandle,
        plants: &PlantStore,
        manager: &PlantRuntimeManager,
        plant_id: &str,
    ) -> AppResult<Plant> {
        manager.stop_runtime(app, plant_id)?;

        plants.update(plant_id, |plant| {
            plant.connected = false;
            plant.paused = false;
        })
    }

    pub fn pause(
        plants: &PlantStore,
        manager: &PlantRuntimeManager,
        plant_id: &str,
    ) -> AppResult<Plant> {
        let plant = plants.get(plant_id)?;

        if !plant.connected {
            return Err(AppError::InvalidArgument(
                "Planta precisa estar conectada para pausar".to_string(),
            ));
        }

        if plant.paused {
            return Ok(plant);
        }

        manager.pause_runtime(plant_id)?;
        plants.update(plant_id, |plant| {
            plant.paused = true;
        })
    }

    pub fn resume(
        plants: &PlantStore,
        manager: &PlantRuntimeManager,
        plant_id: &str,
    ) -> AppResult<Plant> {
        let plant = plants.get(plant_id)?;

        if !plant.connected {
            return Err(AppError::InvalidArgument(
                "Planta precisa estar conectada para retomar".to_string(),
            ));
        }

        if !plant.paused {
            return Ok(plant);
        }

        manager.resume_runtime(plant_id)?;
        plants.update(plant_id, |plant| {
            plant.paused = false;
        })
    }

    pub fn save_setpoint(
        plants: &PlantStore,
        manager: &PlantRuntimeManager,
        request: SavePlantSetpointRequest,
    ) -> AppResult<Plant> {
        let plant = crate::core::services::plant::PlantService::save_setpoint(plants, request)?;

        if plant.connected {
            manager.update_setpoints(&plant.id, collect_runtime_setpoints(&plant))?;
        }

        Ok(plant)
    }

    pub fn save_controller_config(
        plants: &PlantStore,
        plugins: &PluginStore,
        manager: &PlantRuntimeManager,
        request: SavePlantControllerConfigRequest,
    ) -> AppResult<Plant> {
        let current_plant = plants.get(&request.plant_id)?;
        let existing_controller = current_plant
            .controllers
            .iter()
            .find(|controller| controller.id == request.controller_id)
            .cloned();
        let will_have_active_controllers = current_plant
            .controllers
            .iter()
            .any(|controller| controller.id != request.controller_id && controller.active)
            || request.active;

        if will_have_active_controllers {
            let mut loaded_from_workspace = false;
            let driver_plugin = resolve_plugin_for_runtime(
                plugins,
                &current_plant.driver.plugin_id,
                &current_plant.driver.plugin_name,
                PluginType::Driver,
                &mut loaded_from_workspace,
            )?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "Driver da planta '{}' não foi encontrado",
                    current_plant.name
                ))
            })?;
            ensure_driver_supports_write(&driver_plugin)?;
        }

        if current_plant.connected && request.active {
            let mut loaded_from_workspace = false;
            let controller_plugin_id = request
                .plugin_id
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .or_else(|| {
                    existing_controller
                        .as_ref()
                        .map(|controller| controller.plugin_id.as_str())
                })
                .unwrap_or_default();
            let controller_plugin_name = existing_controller
                .as_ref()
                .map(|controller| controller.plugin_name.as_str())
                .unwrap_or_default();
            let controller_plugin = resolve_plugin_for_runtime(
                plugins,
                controller_plugin_id,
                controller_plugin_name,
                PluginType::Controller,
                &mut loaded_from_workspace,
            )?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "Plugin do controlador '{}' não foi encontrado",
                    request.name.trim()
                ))
            })?;
            validate_controller_plugin_source(&controller_plugin, request.name.trim())?;
        }

        let plant = crate::core::services::plant::PlantService::save_controller_config(
            plants, plugins, request,
        )?;

        if plant.connected {
            let (resolved_plant, _driver, active_controllers, _runtime_plugins) =
                resolve_runtime_components_for_connect(plants, plugins, plant)?;
            for controller in &active_controllers {
                validate_controller_plugin_source(&controller.plugin, &controller.instance.name)?;
            }
            manager.update_controllers(
                &resolved_plant.id,
                build_runtime_controller_payloads(&active_controllers)?,
            )?;
            return Ok(resolved_plant);
        }

        Ok(plant)
    }

    pub fn remove_controller(
        plants: &PlantStore,
        plugins: &PluginStore,
        manager: &PlantRuntimeManager,
        request: RemovePlantControllerRequest,
    ) -> AppResult<Plant> {
        let plant = crate::core::services::plant::PlantService::remove_controller(plants, request)?;

        if plant.connected {
            let (resolved_plant, _driver, active_controllers, _runtime_plugins) =
                resolve_runtime_components_for_connect(plants, plugins, plant)?;
            manager.update_controllers(
                &resolved_plant.id,
                build_runtime_controller_payloads(&active_controllers)?,
            )?;
            return Ok(resolved_plant);
        }

        Ok(plant)
    }
}
