mod bootstrap;
mod environment;
mod process;
mod validation;

use self::bootstrap::{
    build_bootstrap_payload, build_runtime_controller_payloads, collect_runtime_setpoints,
    resolve_plugin_for_runtime, resolve_runtime_components_for_connect,
};
use self::environment::{
    compute_env_hash, dedupe_runtime_plugins, ensure_python_env, prepare_runtime_directory,
    prepare_runtime_scaffold, write_bootstrap_files, write_runner_script,
};
use self::process::{
    emit_status_event, send_command, spawn_driver_process, spawn_stderr_task, spawn_stdout_task,
    wait_for_handshake,
};
use self::validation::{
    ensure_driver_supports_write, validate_driver_write_support, validate_plugin_workspace_files,
    validate_python_source_file, validate_runtime_controller,
};
use crate::core::error::{AppError, AppResult};
use crate::core::models::plant::{
    ControllerRuntimeStatus, Plant, PlantController, RemovePlantControllerRequest,
    SavePlantControllerConfigRequest, SavePlantSetpointRequest,
};
use crate::core::models::plugin::{PluginRegistry, PluginRuntime, PluginType};
use crate::core::services::plant::PlantService;
use crate::state::{PlantStore, PluginStore};
use parking_lot::{Condvar, Mutex};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::process::{Child, ChildStdin};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Runtime};
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
const PYTHON_IMPORT_CLASS_CHECK_SCRIPT: &str = r#"
import importlib.util
import pathlib
import sys

source_path = pathlib.Path(sys.argv[1])
class_name = sys.argv[2]
required_methods = [name for name in sys.argv[3:] if name]

sys.path.insert(0, str(source_path.parent))
module_name = f"senamby_runtime_check_{source_path.stem}"
spec = importlib.util.spec_from_file_location(module_name, source_path)
if spec is None or spec.loader is None:
    print(f"Nao foi possivel carregar o modulo '{source_path}'", file=sys.stderr)
    sys.exit(2)

module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

target_class = getattr(module, class_name, None)
if target_class is None:
    print(f"Classe '{class_name}' nao encontrada em '{source_path}'", file=sys.stderr)
    sys.exit(3)

missing = [method for method in required_methods if not callable(getattr(target_class, method, None))]
if missing:
    print(
        f"Classe '{class_name}' nao implementa os metodos obrigatorios: {', '.join(missing)}",
        file=sys.stderr,
    )
    sys.exit(4)
"#;
const STARTUP_TIMEOUT: Duration = Duration::from_secs(12);
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(4);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeLifecycleState {
    #[default]
    Created,
    Bootstrapping,
    Ready,
    Connecting,
    Running,
    Stopping,
    Stopped,
    Faulted,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeCyclePhase {
    #[default]
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
pub struct RuntimeTelemetryPayload {
    #[serde(default)]
    pub timestamp: f64,
    #[serde(default)]
    pub cycle_id: u64,
    pub configured_sample_time_ms: u64,
    #[serde(default)]
    pub effective_dt_ms: f64,
    #[serde(default)]
    pub cycle_duration_ms: f64,
    #[serde(default)]
    pub read_duration_ms: f64,
    #[serde(default)]
    pub control_duration_ms: f64,
    #[serde(default)]
    pub write_duration_ms: f64,
    #[serde(default)]
    pub publish_duration_ms: f64,
    #[serde(default)]
    pub cycle_late: bool,
    #[serde(default)]
    pub late_by_ms: f64,
    #[serde(default)]
    pub phase: String,
    #[serde(default)]
    pub uptime_s: f64,
    #[serde(default)]
    pub sensors: HashMap<String, f64>,
    #[serde(default)]
    pub actuators: HashMap<String, f64>,
    #[serde(default)]
    pub actuators_read: HashMap<String, f64>,
    #[serde(default)]
    pub setpoints: HashMap<String, f64>,
    #[serde(default)]
    pub controller_outputs: HashMap<String, f64>,
    #[serde(default)]
    pub written_outputs: HashMap<String, f64>,
    #[serde(default)]
    pub controller_durations_ms: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeTelemetryEvent {
    pub plant_id: String,
    pub runtime_id: String,
    pub lifecycle_state: RuntimeLifecycleState,
    pub cycle_phase: RuntimeCyclePhase,
    pub timestamp: f64,
    pub cycle_id: u64,
    pub configured_sample_time_ms: u64,
    pub effective_dt_ms: f64,
    pub cycle_duration_ms: f64,
    pub read_duration_ms: f64,
    pub control_duration_ms: f64,
    pub write_duration_ms: f64,
    pub publish_duration_ms: f64,
    pub cycle_late: bool,
    pub late_by_ms: f64,
    pub phase: String,
    pub uptime_s: f64,
    pub sensors: HashMap<String, f64>,
    pub actuators: HashMap<String, f64>,
    pub actuators_read: HashMap<String, f64>,
    pub setpoints: HashMap<String, f64>,
    pub controller_outputs: HashMap<String, f64>,
    pub written_outputs: HashMap<String, f64>,
    pub controller_durations_ms: HashMap<String, f64>,
}

#[derive(Debug, Default, Clone)]
struct RuntimeMetrics {
    lifecycle_state: RuntimeLifecycleState,
    cycle_phase: RuntimeCyclePhase,
    effective_dt_ms: f64,
    cycle_duration_ms: f64,
    read_duration_ms: f64,
    uptime_s: f64,
    cycle_late: bool,
    late_cycle_count: u64,
    last_telemetry_at: Option<u64>,
}

#[derive(Debug, Default)]
struct HandshakeState {
    ready: bool,
    connected: bool,
    error: Option<String>,
}

type SharedHandshake = Arc<(Mutex<HandshakeState>, Condvar)>;
type SharedMetrics = Arc<Mutex<RuntimeMetrics>>;

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
fn saturating_f64_to_u64(value: f64) -> u64 {
    if !value.is_finite() || value <= 0.0 {
        return 0;
    }

    value.floor().clamp(0.0, u64::MAX as f64) as u64
}

fn duration_millis_u64(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

#[derive(Debug, Deserialize)]
struct RuntimeEnvelope {
    #[serde(rename = "type")]
    msg_type: String,
    #[serde(default)]
    payload: Value,
}

#[derive(Debug, Serialize, Clone)]
struct DriverBootstrapVariable {
    id: String,
    name: String,
    #[serde(rename = "type")]
    var_type: crate::core::models::plant::VariableType,
    unit: String,
    setpoint: f64,
    pv_min: f64,
    pv_max: f64,
    #[serde(default)]
    linked_sensor_ids: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
struct DriverBootstrapPlant {
    id: String,
    name: String,
    variables: Vec<DriverBootstrapVariable>,
    sensor_ids: Vec<String>,
    actuator_ids: Vec<String>,
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
    venv_python_path: PathBuf,
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
}

impl PlantRuntimeManager {
    pub fn new() -> Self {
        Self {
            handles: Mutex::new(HashMap::new()),
        }
    }

    pub fn apply_live_stats(&self, mut plant: Plant) -> Plant {
        let handles = self.handles.lock();
        let Some(handle) = handles.get(&plant.id) else {
            return plant;
        };
        let metrics = handle.metrics.lock();
        plant.stats.dt = (metrics.effective_dt_ms / 1000.0).max(0.0);
        plant.stats.uptime = saturating_f64_to_u64(metrics.uptime_s);
        plant
    }

    pub fn apply_live_stats_batch(&self, plants: Vec<Plant>) -> Vec<Plant> {
        plants
            .into_iter()
            .map(|plant| self.apply_live_stats(plant))
            .collect()
    }

    #[allow(clippy::too_many_lines)]
    fn start_runtime<R: Runtime + 'static>(
        &self,
        app: &AppHandle<R>,
        plant: &Plant,
        driver_plugin: &PluginRegistry,
        active_controllers: &[ResolvedRuntimeController],
        runtime_plugins: Vec<PluginRegistry>,
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
        let runtime_plugins = dedupe_runtime_plugins(runtime_plugins);
        let env_hash = compute_env_hash(&runtime_plugins);
        let venv_python_path = ensure_python_env(&runtime_plugins, &env_hash)?;

        let driver_dir = crate::core::services::workspace::WorkspaceService::plugin_directory(
            &driver_plugin.name,
            PluginType::Driver,
        )?;
        validate_plugin_workspace_files(&driver_dir, "driver")?;
        validate_python_source_file(&venv_python_path, &driver_dir.join("main.py"), "driver")?;
        if !active_controllers.is_empty() {
            validate_driver_write_support(&venv_python_path, driver_plugin)?;
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
        let runtime_dir =
            crate::core::services::workspace::WorkspaceService::runtime_directory(&runtime_id)?;
        prepare_runtime_scaffold(&runtime_dir)?;

        let startup_result = (|| -> AppResult<RuntimeHandle> {
            let runner_path = write_runner_script(&runtime_root)?;
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
                duration_millis_u64(STARTUP_TIMEOUT),
                duration_millis_u64(SHUTDOWN_TIMEOUT),
            )?;
            write_bootstrap_files(&bootstrap, &bootstrap_path)?;

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
                venv_python_path: venv_python_path.clone(),
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

    pub fn stop_runtime<R: Runtime>(&self, app: &AppHandle<R>, plant_id: &str) {
        let handle = {
            let mut handles = self.handles.lock();
            handles.remove(plant_id)
        };

        let Some(mut handle) = handle else {
            return;
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
    }

    pub fn update_setpoints(
        &self,
        plant_id: &str,
        setpoints: &HashMap<String, f64>,
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
        controllers: &[DriverBootstrapController],
    ) -> AppResult<()> {
        let payload =
            serde_json::to_value(json!({ "controllers": controllers })).map_err(|error| {
                AppError::IoError(format!(
                    "Falha ao serializar atualização de controladores: {error}"
                ))
            })?;
        self.send_runtime_command_with_payload(plant_id, "update_controllers", payload)
    }

    fn venv_python_path(&self, plant_id: &str) -> AppResult<PathBuf> {
        let handles = self.handles.lock();
        let handle = handles.get(plant_id).ok_or_else(|| {
            AppError::NotFound(format!(
                "Runtime da planta '{plant_id}' não está em execução"
            ))
        })?;
        Ok(handle.venv_python_path.clone())
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
                    "Runtime da planta '{plant_id}' não está em execução"
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

fn persist_runtime_statuses(plant: &Plant) -> AppResult<()> {
    crate::core::services::workspace::WorkspaceService::update_plant_registry(plant, &plant.name)
}

fn set_all_controller_runtime_statuses(
    plants: &PlantStore,
    plant_id: &str,
    status: ControllerRuntimeStatus,
) -> AppResult<Plant> {
    let updated = plants.update(plant_id, |plant| {
        for controller in &mut plant.controllers {
            controller.runtime_status = status;
        }
    })?;
    persist_runtime_statuses(&updated)?;
    Ok(updated)
}

fn set_pending_controller_runtime_statuses(
    plants: &PlantStore,
    plant_id: &str,
    pending_ids: &[String],
) -> AppResult<Plant> {
    let pending_lookup = pending_ids
        .iter()
        .cloned()
        .collect::<std::collections::HashSet<_>>();
    let updated = plants.update(plant_id, |plant| {
        for controller in &mut plant.controllers {
            controller.runtime_status =
                if controller.active && pending_lookup.contains(&controller.id) {
                    ControllerRuntimeStatus::PendingRestart
                } else {
                    ControllerRuntimeStatus::Synced
                };
        }
    })?;
    persist_runtime_statuses(&updated)?;
    Ok(updated)
}

fn collect_incompatible_active_controller_ids(
    python_path: &std::path::Path,
    active_controllers: &[ResolvedRuntimeController],
) -> Vec<String> {
    active_controllers
        .iter()
        .filter_map(|controller| {
            validate_runtime_controller(python_path, &controller.plugin, &controller.instance.name)
                .err()
                .map(|_| controller.instance.id.clone())
        })
        .collect()
}

fn collect_incompatible_controller_ids_for_target_ids(
    python_path: &std::path::Path,
    active_controllers: &[ResolvedRuntimeController],
    target_ids: &[String],
) -> Vec<String> {
    let target_ids = target_ids.iter().cloned().collect::<HashSet<_>>();
    active_controllers
        .iter()
        .filter(|controller| target_ids.contains(&controller.instance.id))
        .filter_map(|controller| {
            validate_runtime_controller(python_path, &controller.plugin, &controller.instance.name)
                .err()
                .map(|_| controller.instance.id.clone())
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RuntimeControllerValidationPlan {
    None,
    Targeted(String),
    All,
}

fn should_validate_driver_write_on_controller_save(
    plant: &Plant,
    request: &SavePlantControllerConfigRequest,
) -> bool {
    request.active && !plant.controllers.iter().any(|controller| controller.active)
}

fn plan_runtime_controller_validation(
    plant: &Plant,
    existing_controller: Option<&PlantController>,
    request: &SavePlantControllerConfigRequest,
) -> RuntimeControllerValidationPlan {
    if !plant.connected {
        return RuntimeControllerValidationPlan::None;
    }

    if plant.controllers.iter().any(|controller| {
        controller.active && controller.runtime_status != ControllerRuntimeStatus::Synced
    }) {
        return RuntimeControllerValidationPlan::All;
    }

    let controller_already_running = existing_controller.is_some_and(|controller| {
        controller.active && controller.runtime_status == ControllerRuntimeStatus::Synced
    });

    if request.active && !controller_already_running {
        return RuntimeControllerValidationPlan::Targeted(request.controller_id.clone());
    }

    RuntimeControllerValidationPlan::None
}

pub struct DriverRuntimeService;

impl DriverRuntimeService {
    pub fn connect<R: Runtime + 'static>(
        app: &AppHandle<R>,
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
        manager.start_runtime(app, &plant, &driver, &active_controllers, runtime_plugins)?;

        let updated = plants.update(plant_id, |plant| {
            plant.connected = true;
            plant.paused = false;
            for controller in &mut plant.controllers {
                controller.runtime_status = ControllerRuntimeStatus::Synced;
            }
        })?;
        persist_runtime_statuses(&updated)?;
        Ok(updated)
    }

    pub fn disconnect<R: Runtime>(
        app: &AppHandle<R>,
        plants: &PlantStore,
        manager: &PlantRuntimeManager,
        plant_id: &str,
    ) -> AppResult<Plant> {
        manager.stop_runtime(app, plant_id);

        plants.update(plant_id, |plant| {
            plant.connected = false;
            plant.paused = false;
        })
    }

    pub fn close<R: Runtime>(
        app: &AppHandle<R>,
        plants: &PlantStore,
        manager: &PlantRuntimeManager,
        plant_id: &str,
    ) -> AppResult<Plant> {
        plants.read(plant_id, |_| ())?;
        manager.stop_runtime(app, plant_id);
        let mut plant = PlantService::close(plants, plant_id)?;
        plant.connected = false;
        plant.paused = false;
        Ok(plant)
    }

    pub fn remove<R: Runtime>(
        app: &AppHandle<R>,
        plants: &PlantStore,
        manager: &PlantRuntimeManager,
        plant_id: &str,
    ) -> AppResult<Plant> {
        plants.read(plant_id, |_| ())?;
        manager.stop_runtime(app, plant_id);
        let mut plant = PlantService::remove(plants, plant_id)?;
        plant.connected = false;
        plant.paused = false;
        Ok(plant)
    }

    pub fn pause(
        plants: &PlantStore,
        _manager: &PlantRuntimeManager,
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

        plants.update(plant_id, |plant| {
            plant.paused = true;
        })
    }

    pub fn resume(
        plants: &PlantStore,
        _manager: &PlantRuntimeManager,
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

        plants.update(plant_id, |plant| {
            plant.paused = false;
        })
    }

    pub fn save_setpoint(
        plants: &PlantStore,
        manager: &PlantRuntimeManager,
        request: &SavePlantSetpointRequest,
    ) -> AppResult<Plant> {
        let plant = crate::core::services::plant::PlantService::save_setpoint(plants, request)?;

        if plant.connected {
            let setpoints = collect_runtime_setpoints(&plant);
            manager.update_setpoints(&plant.id, &setpoints)?;
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
        let request_controller_id = request.controller_id.clone();
        let runtime_python_path = if current_plant.connected {
            manager.venv_python_path(&request.plant_id).ok()
        } else {
            None
        };
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
        let validation_plan = plan_runtime_controller_validation(
            &current_plant,
            existing_controller.as_ref(),
            &request,
        );

        if will_have_active_controllers
            && should_validate_driver_write_on_controller_save(&current_plant, &request)
        {
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
            if let Some(python_path) = runtime_python_path.as_deref() {
                validate_driver_write_support(python_path, &driver_plugin)?;
            } else {
                ensure_driver_supports_write(&driver_plugin)?;
            }
        }

        let plant = crate::core::services::plant::PlantService::save_controller_config(
            plants, plugins, request,
        )?;

        if plant.connected {
            let (resolved_plant, _driver, active_controllers, _runtime_plugins) =
                resolve_runtime_components_for_connect(plants, plugins, plant)?;
            let incompatible_controller_ids = match validation_plan {
                RuntimeControllerValidationPlan::None => Vec::new(),
                RuntimeControllerValidationPlan::All => match runtime_python_path.as_deref() {
                    Some(python_path) => {
                        collect_incompatible_active_controller_ids(python_path, &active_controllers)
                    }
                    None => active_controllers
                        .iter()
                        .map(|controller| controller.instance.id.clone())
                        .collect(),
                },
                RuntimeControllerValidationPlan::Targeted(_) => {
                    match runtime_python_path.as_deref() {
                        Some(python_path) => collect_incompatible_controller_ids_for_target_ids(
                            python_path,
                            &active_controllers,
                            std::slice::from_ref(&request_controller_id),
                        ),
                        None => active_controllers
                            .iter()
                            .filter(|controller| controller.instance.id == request_controller_id)
                            .map(|controller| controller.instance.id.clone())
                            .collect(),
                    }
                }
            };

            if incompatible_controller_ids.is_empty() {
                let controller_payloads = build_runtime_controller_payloads(&active_controllers)?;
                manager.update_controllers(&resolved_plant.id, &controller_payloads)?;
                return set_all_controller_runtime_statuses(
                    plants,
                    &resolved_plant.id,
                    ControllerRuntimeStatus::Synced,
                );
            }

            return set_pending_controller_runtime_statuses(
                plants,
                &resolved_plant.id,
                &incompatible_controller_ids,
            );
        }

        Ok(plant)
    }

    pub fn remove_controller(
        plants: &PlantStore,
        plugins: &PluginStore,
        manager: &PlantRuntimeManager,
        request: &RemovePlantControllerRequest,
    ) -> AppResult<Plant> {
        plants.read(&request.plant_id, |plant| {
            let controller = plant
                .controllers
                .iter()
                .find(|controller| controller.id == request.controller_id)
                .ok_or_else(|| {
                    AppError::NotFound(format!(
                        "Controlador '{}' não encontrado na planta '{}'",
                        request.controller_id, plant.name
                    ))
                })?;

            if plant.connected
                && controller.active
                && controller.runtime_status == ControllerRuntimeStatus::Synced
            {
                return Err(AppError::InvalidArgument(
                    "Não é permitido remover um controlador em execução. Desative-o antes.".into(),
                ));
            }

            Ok(())
        })??;

        let plant = crate::core::services::plant::PlantService::remove_controller(plants, request)?;

        if plant.connected {
            let (resolved_plant, _driver, active_controllers, _runtime_plugins) =
                resolve_runtime_components_for_connect(plants, plugins, plant)?;
            let incompatible_controller_ids = match manager.venv_python_path(&resolved_plant.id) {
                Ok(python_path) => {
                    collect_incompatible_active_controller_ids(&python_path, &active_controllers)
                }
                Err(_) => active_controllers
                    .iter()
                    .map(|controller| controller.instance.id.clone())
                    .collect(),
            };

            if incompatible_controller_ids.is_empty() {
                let controller_payloads = build_runtime_controller_payloads(&active_controllers)?;
                manager.update_controllers(&resolved_plant.id, &controller_payloads)?;
                return set_all_controller_runtime_statuses(
                    plants,
                    &resolved_plant.id,
                    ControllerRuntimeStatus::Synced,
                );
            }

            return set_pending_controller_runtime_statuses(
                plants,
                &resolved_plant.id,
                &incompatible_controller_ids,
            );
        }

        Ok(plant)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::plant::{
        ControllerParam, ControllerParamType, ControllerRuntimeStatus, PlantDriver, PlantStats,
        PlantVariable, RemovePlantControllerRequest, SavePlantControllerConfigRequest,
        SavePlantControllerParamRequest, VariableType,
    };
    use crate::core::models::plugin::{
        PluginRegistry, PluginRuntime, PluginType, SchemaFieldValue,
    };
    use crate::core::services::workspace::{test_workspace_root, WorkspaceService};
    use parking_lot::Mutex;
    use std::collections::HashMap;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};

    fn plant_registry_path(name: &str) -> PathBuf {
        test_workspace_root()
            .join("plants")
            .join(name)
            .join("registry.json")
    }

    fn create_test_plant(id: &str, name: &str) -> Plant {
        Plant {
            id: id.to_string(),
            name: name.to_string(),
            sample_time_ms: 100,
            variables: vec![
                PlantVariable {
                    id: "var_0".to_string(),
                    name: "Temperatura".to_string(),
                    var_type: VariableType::Sensor,
                    unit: "C".to_string(),
                    setpoint: 42.0,
                    pv_min: 0.0,
                    pv_max: 100.0,
                    linked_sensor_ids: None,
                },
                PlantVariable {
                    id: "var_1".to_string(),
                    name: "Valvula".to_string(),
                    var_type: VariableType::Atuador,
                    unit: "%".to_string(),
                    setpoint: 0.0,
                    pv_min: 0.0,
                    pv_max: 100.0,
                    linked_sensor_ids: Some(vec!["var_0".to_string()]),
                },
            ],
            driver: PlantDriver {
                plugin_id: "driver_plugin".to_string(),
                plugin_name: "Driver Python".to_string(),
                runtime: PluginRuntime::Python,
                source_file: Some("main.py".to_string()),
                source_code: None,
                config: HashMap::new(),
            },
            controllers: Vec::new(),
            connected: true,
            paused: false,
            stats: PlantStats::default(),
        }
    }

    fn find_test_python() -> String {
        ["python3", "python"]
            .into_iter()
            .find(|candidate| Command::new(candidate).arg("--version").output().is_ok())
            .expect("python nao encontrado para teste de runtime")
            .to_string()
    }

    fn spawn_runtime_test_child() -> std::process::Child {
        Command::new(find_test_python())
            .arg("-c")
            .arg("import sys; sys.stdin.readline()")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("falha ao criar processo de teste da runtime")
    }

    fn spawn_runtime_capture_child(commands_path: &Path) -> std::process::Child {
        Command::new(find_test_python())
            .arg("-c")
            .arg(
                "import pathlib, sys\npath = pathlib.Path(sys.argv[1])\nwith path.open('w', encoding='utf-8') as handle:\n    for line in sys.stdin:\n        handle.write(line)\n        handle.flush()\n",
            )
            .arg(commands_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("falha ao criar processo de captura da runtime")
    }

    fn read_captured_commands(commands_path: &Path) -> String {
        std::thread::sleep(Duration::from_millis(50));
        fs::read_to_string(commands_path).unwrap_or_default()
    }

    fn create_test_plugin(
        id: &str,
        name: &str,
        plugin_type: PluginType,
        entry_class: &str,
    ) -> PluginRegistry {
        PluginRegistry {
            id: id.to_string(),
            name: name.to_string(),
            plugin_type,
            runtime: PluginRuntime::Python,
            entry_class: entry_class.to_string(),
            schema: vec![],
            source_file: Some("main.py".to_string()),
            source_code: None,
            dependencies: vec![],
            description: None,
            version: None,
            author: None,
        }
    }

    fn save_test_plugin(plugin: &PluginRegistry, source_code: &str) {
        WorkspaceService::save_plugin_registry(plugin, source_code).unwrap();
    }

    fn insert_runtime_handle(
        manager: &PlantRuntimeManager,
        plant: &Plant,
        runtime_id: &str,
        commands_path: &Path,
    ) -> PathBuf {
        let runtime_dir = WorkspaceService::runtime_directory(runtime_id).unwrap();
        fs::create_dir_all(&runtime_dir).unwrap();

        let mut child = spawn_runtime_capture_child(commands_path);
        let stdin = child
            .stdin
            .take()
            .expect("stdin ausente no processo de captura");

        manager.handles.lock().insert(
            plant.id.clone(),
            RuntimeHandle {
                plant_id: plant.id.clone(),
                runtime_id: runtime_id.to_string(),
                runtime_dir: runtime_dir.clone(),
                venv_python_path: PathBuf::from(find_test_python()),
                configured_sample_time_ms: plant.sample_time_ms,
                stdin: Arc::new(Mutex::new(stdin)),
                child,
                stdout_task: None,
                stderr_task: None,
                metrics: Arc::new(Mutex::new(RuntimeMetrics::default())),
            },
        );

        runtime_dir
    }

    #[test]
    fn close_plant_stops_runtime_and_preserves_registry() {
        let suffix = Uuid::new_v4().simple().to_string();
        let store = Arc::new(PlantStore::new());
        let manager = PlantRuntimeManager::new();
        let plant = create_test_plant(
            &format!("plant_close_{suffix}"),
            &format!("Plant Close Runtime {suffix}"),
        );
        WorkspaceService::save_plant_registry(&plant).unwrap();
        store.insert(plant.clone()).unwrap();

        let runtime_id = format!("rt_close_test_{suffix}");
        let runtime_dir = WorkspaceService::runtime_directory(&runtime_id).unwrap();
        fs::create_dir_all(&runtime_dir).unwrap();
        fs::write(runtime_dir.join("marker.txt"), "runtime-alive").unwrap();

        let mut child = spawn_runtime_test_child();
        let stdin = child
            .stdin
            .take()
            .expect("stdin ausente no processo de teste");

        manager.handles.lock().insert(
            plant.id.clone(),
            RuntimeHandle {
                plant_id: plant.id.clone(),
                runtime_id,
                runtime_dir: runtime_dir.clone(),
                venv_python_path: PathBuf::from(find_test_python()),
                configured_sample_time_ms: plant.sample_time_ms,
                stdin: Arc::new(Mutex::new(stdin)),
                child,
                stdout_task: None,
                stderr_task: None,
                metrics: Arc::new(Mutex::new(RuntimeMetrics::default())),
            },
        );

        let app = tauri::test::mock_app();
        let closed =
            DriverRuntimeService::close(app.handle(), store.as_ref(), &manager, &plant.id).unwrap();

        assert_eq!(closed.id, plant.id);
        assert!(!closed.connected);
        assert!(!closed.paused);
        assert!(!store.exists(&plant.id));
        assert!(plant_registry_path(&plant.name).exists());
        assert!(!runtime_dir.exists());
        assert!(manager.handles.lock().is_empty());
    }

    #[test]
    fn save_controller_config_hot_updates_running_runtime_when_controller_is_loadable() {
        let suffix = Uuid::new_v4().simple().to_string();
        let store = Arc::new(PlantStore::new());
        let plugins = PluginStore::new();
        let manager = PlantRuntimeManager::new();
        let app = tauri::test::mock_app();

        let driver_plugin = create_test_plugin(
            &format!("driver_plugin_{suffix}"),
            &format!("Driver Python {suffix}"),
            PluginType::Driver,
            "DriverPython",
        );
        save_test_plugin(
            &driver_plugin,
            r#"
class DriverPython:
    def __init__(self, context):
        self.context = context
    def connect(self):
        return True
    def stop(self):
        return True
    def read(self):
        return {"sensors": {"sensor_1": 1.0}, "actuators": {"actuator_1": 0.0}}
    def write(self, outputs):
        return True
"#,
        );
        plugins.insert(driver_plugin.clone()).unwrap();

        let controller_plugin = create_test_plugin(
            &format!("controller_plugin_{suffix}"),
            &format!("Controller Python {suffix}"),
            PluginType::Controller,
            "ControllerPython",
        );
        save_test_plugin(
            &controller_plugin,
            r#"
class ControllerPython:
    def __init__(self, context):
        self.context = context
    def compute(self, snapshot):
        return {"actuator_1": 1.0}
"#,
        );
        plugins.insert(controller_plugin.clone()).unwrap();

        let mut plant = create_test_plant(
            &format!("plant_sync_{suffix}"),
            &format!("Plant Sync {suffix}"),
        );
        plant.driver.plugin_id = driver_plugin.id.clone();
        plant.driver.plugin_name = driver_plugin.name.clone();
        WorkspaceService::save_plant_registry(&plant).unwrap();
        store.insert(plant.clone()).unwrap();

        let commands_path =
            std::env::temp_dir().join(format!("senamby_runtime_sync_commands_{suffix}.jsonl"));
        let _ = fs::remove_file(&commands_path);
        insert_runtime_handle(
            &manager,
            &plant,
            &format!("rt_sync_test_{suffix}"),
            &commands_path,
        );

        let saved = DriverRuntimeService::save_controller_config(
            store.as_ref(),
            &plugins,
            &manager,
            SavePlantControllerConfigRequest {
                plant_id: plant.id.clone(),
                controller_id: "ctrl_sync".to_string(),
                plugin_id: Some(controller_plugin.id.clone()),
                name: "Controller Sync".to_string(),
                controller_type: "PID".to_string(),
                active: true,
                input_variable_ids: vec!["var_0".to_string()],
                output_variable_ids: vec!["var_1".to_string()],
                params: vec![SavePlantControllerParamRequest {
                    key: "kp".to_string(),
                    param_type: ControllerParamType::Number,
                    value: SchemaFieldValue::Float(1.2),
                    label: "Kp".to_string(),
                }],
            },
        )
        .unwrap();

        assert_eq!(saved.controllers.len(), 1);
        assert!(saved.controllers[0].active);
        assert_eq!(
            saved.controllers[0].runtime_status,
            ControllerRuntimeStatus::Synced
        );
        assert!(read_captured_commands(&commands_path).contains("\"type\":\"update_controllers\""));

        manager.stop_runtime(app.handle(), &plant.id);
    }

    #[test]
    fn save_controller_config_marks_controller_pending_restart_when_current_env_cannot_load_it() {
        let suffix = Uuid::new_v4().simple().to_string();
        let store = Arc::new(PlantStore::new());
        let plugins = PluginStore::new();
        let manager = PlantRuntimeManager::new();
        let app = tauri::test::mock_app();

        let driver_plugin = create_test_plugin(
            &format!("driver_plugin_{suffix}"),
            &format!("Driver Python {suffix}"),
            PluginType::Driver,
            "DriverPython",
        );
        save_test_plugin(
            &driver_plugin,
            r#"
class DriverPython:
    def __init__(self, context):
        self.context = context
    def connect(self):
        return True
    def stop(self):
        return True
    def read(self):
        return {"sensors": {"sensor_1": 1.0}, "actuators": {"actuator_1": 0.0}}
    def write(self, outputs):
        return True
"#,
        );
        plugins.insert(driver_plugin.clone()).unwrap();

        let controller_plugin = create_test_plugin(
            &format!("controller_plugin_pending_{suffix}"),
            &format!("Controller Pending {suffix}"),
            PluginType::Controller,
            "ControllerPending",
        );
        save_test_plugin(
            &controller_plugin,
            r#"
import dependency_that_does_not_exist_for_senamby_test

class ControllerPending:
    def __init__(self, context):
        self.context = context
    def compute(self, snapshot):
        return {"actuator_1": 1.0}
"#,
        );
        plugins.insert(controller_plugin.clone()).unwrap();

        let mut plant = create_test_plant(
            &format!("plant_pending_{suffix}"),
            &format!("Plant Pending {suffix}"),
        );
        plant.driver.plugin_id = driver_plugin.id.clone();
        plant.driver.plugin_name = driver_plugin.name.clone();
        WorkspaceService::save_plant_registry(&plant).unwrap();
        store.insert(plant.clone()).unwrap();

        let commands_path =
            std::env::temp_dir().join(format!("senamby_runtime_pending_commands_{suffix}.jsonl"));
        let _ = fs::remove_file(&commands_path);
        insert_runtime_handle(
            &manager,
            &plant,
            &format!("rt_pending_test_{suffix}"),
            &commands_path,
        );

        let saved = DriverRuntimeService::save_controller_config(
            store.as_ref(),
            &plugins,
            &manager,
            SavePlantControllerConfigRequest {
                plant_id: plant.id.clone(),
                controller_id: "ctrl_pending".to_string(),
                plugin_id: Some(controller_plugin.id.clone()),
                name: "Controller Pending".to_string(),
                controller_type: "PID".to_string(),
                active: true,
                input_variable_ids: vec!["var_0".to_string()],
                output_variable_ids: vec!["var_1".to_string()],
                params: vec![SavePlantControllerParamRequest {
                    key: "kp".to_string(),
                    param_type: ControllerParamType::Number,
                    value: SchemaFieldValue::Float(1.2),
                    label: "Kp".to_string(),
                }],
            },
        )
        .unwrap();

        assert_eq!(saved.controllers.len(), 1);
        assert_eq!(
            saved.controllers[0].runtime_status,
            ControllerRuntimeStatus::PendingRestart
        );
        assert!(!read_captured_commands(&commands_path).contains("\"type\":\"update_controllers\""));

        manager.stop_runtime(app.handle(), &plant.id);
    }

    #[test]
    fn remove_controller_rejects_active_synced_controller_while_runtime_is_running() {
        let suffix = Uuid::new_v4().simple().to_string();
        let store = Arc::new(PlantStore::new());
        let plugins = PluginStore::new();
        let manager = PlantRuntimeManager::new();

        let mut plant = create_test_plant(
            &format!("plant_remove_active_{suffix}"),
            &format!("Plant Remove Active {suffix}"),
        );
        plant.controllers.push(PlantController {
            id: "ctrl_running".to_string(),
            plugin_id: "controller_plugin".to_string(),
            plugin_name: "Controller Running".to_string(),
            name: "Controller Running".to_string(),
            controller_type: "PID".to_string(),
            active: true,
            input_variable_ids: vec!["var_0".to_string()],
            output_variable_ids: vec!["var_1".to_string()],
            params: HashMap::from([(
                "kp".to_string(),
                ControllerParam {
                    param_type: ControllerParamType::Number,
                    value: SchemaFieldValue::Float(1.0),
                    label: "Kp".to_string(),
                },
            )]),
            runtime_status: ControllerRuntimeStatus::Synced,
        });
        WorkspaceService::save_plant_registry(&plant).unwrap();
        store.insert(plant.clone()).unwrap();

        let error = DriverRuntimeService::remove_controller(
            store.as_ref(),
            &plugins,
            &manager,
            &RemovePlantControllerRequest {
                plant_id: plant.id.clone(),
                controller_id: "ctrl_running".to_string(),
            },
        )
        .unwrap_err();

        assert!(matches!(error, AppError::InvalidArgument(_)));
        assert!(error.to_string().contains("Desative-o antes"));
    }

    #[test]
    fn remove_controller_allows_inactive_controller_and_updates_runtime() {
        let suffix = Uuid::new_v4().simple().to_string();
        let store = Arc::new(PlantStore::new());
        let plugins = PluginStore::new();
        let manager = PlantRuntimeManager::new();
        let app = tauri::test::mock_app();

        let driver_plugin = create_test_plugin(
            &format!("driver_plugin_{suffix}"),
            &format!("Driver Python {suffix}"),
            PluginType::Driver,
            "DriverPython",
        );
        save_test_plugin(
            &driver_plugin,
            r#"
class DriverPython:
    def __init__(self, context):
        self.context = context
    def connect(self):
        return True
    def stop(self):
        return True
    def read(self):
        return {"sensors": {"sensor_1": 1.0}, "actuators": {"actuator_1": 0.0}}
    def write(self, outputs):
        return True
"#,
        );
        plugins.insert(driver_plugin.clone()).unwrap();

        let mut plant = create_test_plant(
            &format!("plant_remove_inactive_{suffix}"),
            &format!("Plant Remove Inactive {suffix}"),
        );
        plant.driver.plugin_id = driver_plugin.id.clone();
        plant.driver.plugin_name = driver_plugin.name.clone();
        plant.controllers.push(PlantController {
            id: "ctrl_idle".to_string(),
            plugin_id: "controller_idle".to_string(),
            plugin_name: "Controller Idle".to_string(),
            name: "Controller Idle".to_string(),
            controller_type: "PID".to_string(),
            active: false,
            input_variable_ids: vec!["var_0".to_string()],
            output_variable_ids: vec!["var_1".to_string()],
            params: HashMap::new(),
            runtime_status: ControllerRuntimeStatus::Synced,
        });
        WorkspaceService::save_plant_registry(&plant).unwrap();
        store.insert(plant.clone()).unwrap();

        let commands_path =
            std::env::temp_dir().join(format!("senamby_runtime_remove_inactive_{suffix}.jsonl"));
        let _ = fs::remove_file(&commands_path);
        insert_runtime_handle(
            &manager,
            &plant,
            &format!("rt_remove_inactive_test_{suffix}"),
            &commands_path,
        );

        let updated = DriverRuntimeService::remove_controller(
            store.as_ref(),
            &plugins,
            &manager,
            &RemovePlantControllerRequest {
                plant_id: plant.id.clone(),
                controller_id: "ctrl_idle".to_string(),
            },
        )
        .unwrap();

        assert!(updated.controllers.is_empty());
        assert!(read_captured_commands(&commands_path).contains("\"type\":\"update_controllers\""));

        manager.stop_runtime(app.handle(), &plant.id);
    }

    #[test]
    fn plan_runtime_controller_validation_skips_reloading_for_synced_active_controller_updates() {
        let mut plant = create_test_plant("plant_validation_none", "Plant Validation None");
        plant.connected = true;
        plant.controllers.push(PlantController {
            id: "ctrl_running".to_string(),
            plugin_id: "controller_plugin".to_string(),
            plugin_name: "Controller Running".to_string(),
            name: "Controller Running".to_string(),
            controller_type: "PID".to_string(),
            active: true,
            input_variable_ids: vec!["var_0".to_string()],
            output_variable_ids: vec!["var_1".to_string()],
            params: HashMap::new(),
            runtime_status: ControllerRuntimeStatus::Synced,
        });

        let plan = plan_runtime_controller_validation(
            &plant,
            plant.controllers.first(),
            &SavePlantControllerConfigRequest {
                plant_id: plant.id.clone(),
                controller_id: "ctrl_running".to_string(),
                plugin_id: Some("controller_plugin".to_string()),
                name: "Controller Running".to_string(),
                controller_type: "PID".to_string(),
                active: true,
                input_variable_ids: vec!["var_0".to_string()],
                output_variable_ids: vec!["var_1".to_string()],
                params: vec![],
            },
        );

        assert_eq!(plan, RuntimeControllerValidationPlan::None);
    }

    #[test]
    fn plan_runtime_controller_validation_targets_newly_activated_controller() {
        let mut plant = create_test_plant("plant_validation_targeted", "Plant Validation Targeted");
        plant.connected = true;
        plant.controllers.push(PlantController {
            id: "ctrl_idle".to_string(),
            plugin_id: "controller_plugin".to_string(),
            plugin_name: "Controller Idle".to_string(),
            name: "Controller Idle".to_string(),
            controller_type: "PID".to_string(),
            active: false,
            input_variable_ids: vec!["var_0".to_string()],
            output_variable_ids: vec!["var_1".to_string()],
            params: HashMap::new(),
            runtime_status: ControllerRuntimeStatus::Synced,
        });

        let plan = plan_runtime_controller_validation(
            &plant,
            plant.controllers.first(),
            &SavePlantControllerConfigRequest {
                plant_id: plant.id.clone(),
                controller_id: "ctrl_idle".to_string(),
                plugin_id: Some("controller_plugin".to_string()),
                name: "Controller Idle".to_string(),
                controller_type: "PID".to_string(),
                active: true,
                input_variable_ids: vec!["var_0".to_string()],
                output_variable_ids: vec!["var_1".to_string()],
                params: vec![],
            },
        );

        assert_eq!(
            plan,
            RuntimeControllerValidationPlan::Targeted("ctrl_idle".to_string())
        );
    }

    #[test]
    fn plan_runtime_controller_validation_requires_full_check_when_any_active_controller_is_pending(
    ) {
        let mut plant = create_test_plant("plant_validation_all", "Plant Validation All");
        plant.connected = true;
        plant.controllers.push(PlantController {
            id: "ctrl_pending".to_string(),
            plugin_id: "controller_plugin".to_string(),
            plugin_name: "Controller Pending".to_string(),
            name: "Controller Pending".to_string(),
            controller_type: "PID".to_string(),
            active: true,
            input_variable_ids: vec!["var_0".to_string()],
            output_variable_ids: vec!["var_1".to_string()],
            params: HashMap::new(),
            runtime_status: ControllerRuntimeStatus::PendingRestart,
        });

        let plan = plan_runtime_controller_validation(
            &plant,
            plant.controllers.first(),
            &SavePlantControllerConfigRequest {
                plant_id: plant.id.clone(),
                controller_id: "ctrl_pending".to_string(),
                plugin_id: Some("controller_plugin".to_string()),
                name: "Controller Pending".to_string(),
                controller_type: "PID".to_string(),
                active: true,
                input_variable_ids: vec!["var_0".to_string()],
                output_variable_ids: vec!["var_1".to_string()],
                params: vec![],
            },
        );

        assert_eq!(plan, RuntimeControllerValidationPlan::All);
    }

    #[test]
    fn pause_and_resume_only_toggle_visual_state_without_sending_runtime_commands() {
        let suffix = Uuid::new_v4().simple().to_string();
        let store = Arc::new(PlantStore::new());
        let manager = PlantRuntimeManager::new();
        let app = tauri::test::mock_app();
        let plant = create_test_plant(
            &format!("plant_pause_{suffix}"),
            &format!("Plant Pause {suffix}"),
        );
        WorkspaceService::save_plant_registry(&plant).unwrap();
        store.insert(plant.clone()).unwrap();

        let commands_path =
            std::env::temp_dir().join(format!("senamby_runtime_pause_resume_{suffix}.jsonl"));
        let _ = fs::remove_file(&commands_path);
        insert_runtime_handle(
            &manager,
            &plant,
            &format!("rt_pause_test_{suffix}"),
            &commands_path,
        );

        let paused = DriverRuntimeService::pause(store.as_ref(), &manager, &plant.id).unwrap();
        assert!(paused.paused);

        let resumed = DriverRuntimeService::resume(store.as_ref(), &manager, &plant.id).unwrap();
        assert!(!resumed.paused);

        let captured_commands = read_captured_commands(&commands_path);
        assert!(!captured_commands.contains("\"type\":\"pause\""));
        assert!(!captured_commands.contains("\"type\":\"resume\""));

        manager.stop_runtime(app.handle(), &plant.id);
    }
}
