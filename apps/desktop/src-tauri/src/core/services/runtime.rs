use crate::core::error::{AppError, AppResult};
use crate::core::models::plant::Plant;
use crate::core::models::plugin::{PluginRegistry, PluginRuntime, PluginType};
use crate::core::services::workspace::WorkspaceService;
use crate::state::{PlantStore, PluginStore};
use parking_lot::{Condvar, Mutex};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

const RUNNER_SCRIPT: &str = include_str!("../../../runtime/python/runner.py");
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

#[derive(Debug, Serialize)]
struct DriverBootstrapPayload {
    runtime_id: String,
    plant_id: String,
    plant_name: String,
    sample_time_ms: u64,
    driver_class_name: String,
    driver_config: Value,
    variables: Value,
    sensors: Vec<String>,
    actuators: Vec<String>,
    setpoints: HashMap<String, f64>,
    driver_dir: String,
    runtime_dir: String,
    venv_python_path: String,
    runner_path: String,
}

#[derive(Debug)]
struct RuntimeHandle {
    plant_id: String,
    runtime_id: String,
    #[allow(dead_code)]
    env_hash: String,
    #[allow(dead_code)]
    venv_python_path: PathBuf,
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

    pub fn is_running(&self, plant_id: &str) -> bool {
        self.handles.lock().contains_key(plant_id)
    }

    pub fn start_runtime(
        &self,
        app: &AppHandle,
        plant: &Plant,
        driver_plugin: &PluginRegistry,
    ) -> AppResult<()> {
        if self.is_running(&plant.id) {
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
        let env_hash = compute_env_hash(driver_plugin);
        let venv_python_path = ensure_python_env(driver_plugin, &env_hash)?;
        let runtime_dir = prepare_runtime_directory()?;
        let runtime_dir = runtime_dir.join(&runtime_id);
        prepare_runtime_scaffold(&runtime_dir)?;

        let driver_dir =
            WorkspaceService::plugin_directory(&driver_plugin.name, PluginType::Driver)?;
        validate_driver_files(&driver_dir)?;

        let runner_path = write_runner_script(&runtime_dir)?;
        let bootstrap = build_bootstrap_payload(
            &runtime_id,
            plant,
            &driver_dir,
            &runtime_dir,
            &venv_python_path,
            &runner_path,
        )?;
        write_bootstrap_files(&runtime_dir, &bootstrap, plant)?;

        let mut child = spawn_driver_process(
            &venv_python_path,
            &runner_path,
            &runtime_dir,
            &driver_dir,
            &runtime_dir.join("driver").join("bootstrap.json"),
        )?;

        let stdin = child.stdin.take().ok_or_else(|| AppError::InternalError)?;
        let stdout = child.stdout.take().ok_or_else(|| AppError::InternalError)?;
        let stderr = child.stderr.take().ok_or_else(|| AppError::InternalError)?;

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
        let stderr_task =
            spawn_stderr_task(app.clone(), plant.id.clone(), runtime_id.clone(), stderr);

        send_command(
            &shared_stdin,
            "init",
            Some(serde_json::to_value(&bootstrap).map_err(|error| {
                AppError::IoError(format!("Falha ao serializar payload init: {error}"))
            })?),
        )?;
        send_command(&shared_stdin, "start", None)?;

        wait_for_handshake(&handshake, STARTUP_TIMEOUT)?;
        emit_status_event(
            app,
            RuntimeStatusEvent {
                plant_id: plant.id.clone(),
                runtime_id: runtime_id.clone(),
                lifecycle_state: RuntimeLifecycleState::Running,
                cycle_phase: RuntimeCyclePhase::ReadInputs,
                configured_sample_time_ms: plant.sample_time_ms,
                effective_dt_ms: plant.sample_time_ms as f64,
                cycle_late: false,
            },
        );

        let mut handles = self.handles.lock();
        handles.insert(
            plant.id.clone(),
            RuntimeHandle {
                plant_id: plant.id.clone(),
                runtime_id,
                env_hash,
                venv_python_path,
                runtime_dir,
                configured_sample_time_ms: plant.sample_time_ms,
                stdin: shared_stdin,
                child,
                stdout_task: Some(stdout_task),
                stderr_task: Some(stderr_task),
                metrics,
            },
        );

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

        let started_wait = Instant::now();
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

        let driver = plugins.get(&plant.driver.plugin_id)?;
        manager.start_runtime(app, &plant, &driver)?;

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
}

fn prepare_runtime_directory() -> AppResult<PathBuf> {
    let seed_runtime_root = WorkspaceService::runtime_directory("seed_runtime")?;
    let runtime_root = seed_runtime_root
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| AppError::InternalError)?;
    fs::create_dir_all(&runtime_root).map_err(|error| {
        AppError::IoError(format!(
            "Falha ao criar diretório de runtimes '{}': {error}",
            runtime_root.display()
        ))
    })?;
    Ok(runtime_root)
}

fn prepare_runtime_scaffold(runtime_dir: &Path) -> AppResult<()> {
    fs::create_dir_all(runtime_dir.join("driver")).map_err(|error| {
        AppError::IoError(format!(
            "Falha ao criar diretório driver da runtime '{}': {error}",
            runtime_dir.display()
        ))
    })?;
    fs::create_dir_all(runtime_dir.join("logs")).map_err(|error| {
        AppError::IoError(format!(
            "Falha ao criar diretório logs da runtime '{}': {error}",
            runtime_dir.display()
        ))
    })?;
    fs::create_dir_all(runtime_dir.join("ipc")).map_err(|error| {
        AppError::IoError(format!(
            "Falha ao criar diretório ipc da runtime '{}': {error}",
            runtime_dir.display()
        ))
    })?;
    Ok(())
}

fn validate_driver_files(driver_dir: &Path) -> AppResult<()> {
    let registry_path = driver_dir.join("registry.json");
    if !registry_path.exists() {
        return Err(AppError::NotFound(format!(
            "registry.json do driver não encontrado em '{}'",
            registry_path.display()
        )));
    }

    let main_path = driver_dir.join("main.py");
    if !main_path.exists() {
        return Err(AppError::NotFound(format!(
            "main.py do driver não encontrado em '{}'",
            main_path.display()
        )));
    }

    Ok(())
}

fn build_bootstrap_payload(
    runtime_id: &str,
    plant: &Plant,
    driver_dir: &Path,
    runtime_dir: &Path,
    venv_python_path: &Path,
    runner_path: &Path,
) -> AppResult<DriverBootstrapPayload> {
    let mut sensors = Vec::new();
    let mut actuators = Vec::new();
    let mut setpoints = HashMap::new();
    for variable in &plant.variables {
        setpoints.insert(variable.id.clone(), variable.setpoint);
        match variable.var_type {
            crate::core::models::plant::VariableType::Sensor => sensors.push(variable.id.clone()),
            crate::core::models::plant::VariableType::Atuador => {
                actuators.push(variable.id.clone())
            }
        }
    }

    Ok(DriverBootstrapPayload {
        runtime_id: runtime_id.to_string(),
        plant_id: plant.id.clone(),
        plant_name: plant.name.clone(),
        sample_time_ms: plant.sample_time_ms,
        driver_class_name: to_driver_class_name(&plant.driver.plugin_name),
        driver_config: serde_json::to_value(&plant.driver.config).map_err(|error| {
            AppError::IoError(format!("Falha ao serializar config do driver: {error}"))
        })?,
        variables: serde_json::to_value(&plant.variables).map_err(|error| {
            AppError::IoError(format!("Falha ao serializar variáveis da planta: {error}"))
        })?,
        sensors,
        actuators,
        setpoints,
        driver_dir: driver_dir.display().to_string(),
        runtime_dir: runtime_dir.display().to_string(),
        venv_python_path: venv_python_path.display().to_string(),
        runner_path: runner_path.display().to_string(),
    })
}

fn write_bootstrap_files(
    runtime_dir: &Path,
    bootstrap: &DriverBootstrapPayload,
    plant: &Plant,
) -> AppResult<()> {
    let runtime_path = runtime_dir.join("runtime.json");
    let runtime_payload = json!({
        "runtime_id": bootstrap.runtime_id,
        "plant_id": bootstrap.plant_id,
        "sample_time_ms": bootstrap.sample_time_ms,
        "created_at": SystemTime::now().duration_since(UNIX_EPOCH).map(|t| t.as_secs()).unwrap_or(0),
    });
    fs::write(
        &runtime_path,
        serde_json::to_string_pretty(&runtime_payload).map_err(|error| {
            AppError::IoError(format!("Falha ao serializar runtime.json: {error}"))
        })?,
    )
    .map_err(|error| {
        AppError::IoError(format!(
            "Falha ao gravar runtime.json em '{}': {error}",
            runtime_path.display()
        ))
    })?;

    let plant_path = runtime_dir.join("plant.json");
    fs::write(
        &plant_path,
        serde_json::to_string_pretty(plant).map_err(|error| {
            AppError::IoError(format!("Falha ao serializar plant.json: {error}"))
        })?,
    )
    .map_err(|error| {
        AppError::IoError(format!(
            "Falha ao gravar plant.json em '{}': {error}",
            plant_path.display()
        ))
    })?;

    let bootstrap_path = runtime_dir.join("driver").join("bootstrap.json");
    fs::write(
        &bootstrap_path,
        serde_json::to_string_pretty(bootstrap).map_err(|error| {
            AppError::IoError(format!("Falha ao serializar bootstrap.json: {error}"))
        })?,
    )
    .map_err(|error| {
        AppError::IoError(format!(
            "Falha ao gravar bootstrap.json em '{}': {error}",
            bootstrap_path.display()
        ))
    })?;

    Ok(())
}

fn write_runner_script(runtime_dir: &Path) -> AppResult<PathBuf> {
    let runner_path = runtime_dir.join("driver").join("runner.py");
    fs::write(&runner_path, RUNNER_SCRIPT).map_err(|error| {
        AppError::IoError(format!(
            "Falha ao gravar runner Python em '{}': {error}",
            runner_path.display()
        ))
    })?;
    Ok(runner_path)
}

fn spawn_driver_process(
    venv_python_path: &Path,
    runner_path: &Path,
    runtime_dir: &Path,
    driver_dir: &Path,
    bootstrap_path: &Path,
) -> AppResult<Child> {
    Command::new(venv_python_path)
        .arg("-u")
        .arg(runner_path)
        .arg("--runtime-dir")
        .arg(runtime_dir)
        .arg("--driver-dir")
        .arg(driver_dir)
        .arg("--bootstrap")
        .arg(bootstrap_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            AppError::IoError(format!(
                "Falha ao iniciar processo Python do driver '{}': {error}",
                venv_python_path.display()
            ))
        })
}

fn spawn_stdout_task(
    app: AppHandle,
    plant_store: Arc<PlantStore>,
    plant_id: String,
    runtime_id: String,
    configured_sample_time_ms: u64,
    stdout: std::process::ChildStdout,
    handshake: SharedHandshake,
    metrics: SharedMetrics,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            let line = match line {
                Ok(line) => line,
                Err(error) => {
                    let _ = emit_error_event(
                        &app,
                        &plant_id,
                        &runtime_id,
                        format!("Falha ao ler stdout do driver: {error}"),
                    );
                    break;
                }
            };

            let envelope = match serde_json::from_str::<RuntimeEnvelope>(&line) {
                Ok(message) => message,
                Err(error) => {
                    let _ = emit_error_event(
                        &app,
                        &plant_id,
                        &runtime_id,
                        format!("Mensagem inválida recebida do driver: {error}"),
                    );
                    continue;
                }
            };

            match envelope.msg_type.as_str() {
                "ready" => {
                    {
                        let mut lock = handshake.0.lock();
                        lock.ready = true;
                    }
                    handshake.1.notify_all();
                    let mut lock = metrics.lock();
                    lock.lifecycle_state = RuntimeLifecycleState::Ready;
                }
                "connected" => {
                    {
                        let mut lock = handshake.0.lock();
                        lock.connected = true;
                    }
                    handshake.1.notify_all();
                    let mut lock = metrics.lock();
                    lock.lifecycle_state = RuntimeLifecycleState::Running;
                    lock.cycle_phase = RuntimeCyclePhase::ReadInputs;
                }
                "error" => {
                    {
                        let mut lock = handshake.0.lock();
                        lock.error = Some(
                            envelope
                                .payload
                                .get("message")
                                .and_then(Value::as_str)
                                .unwrap_or("Erro na runtime Python")
                                .to_string(),
                        );
                    }
                    handshake.1.notify_all();
                    let mut lock = metrics.lock();
                    lock.lifecycle_state = RuntimeLifecycleState::Faulted;
                    let message = envelope
                        .payload
                        .get("message")
                        .and_then(Value::as_str)
                        .unwrap_or("Erro na runtime Python")
                        .to_string();
                    let _ = emit_error_event(&app, &plant_id, &runtime_id, message);
                }
                "telemetry" => {
                    process_telemetry(
                        &app,
                        &plant_store,
                        &plant_id,
                        &runtime_id,
                        configured_sample_time_ms,
                        envelope.payload,
                        &metrics,
                    );
                }
                "cycle_overrun" => {
                    let mut lock = metrics.lock();
                    lock.cycle_late = true;
                    lock.late_cycle_count = lock.late_cycle_count.saturating_add(1);
                }
                "stopped" => {
                    let mut lock = metrics.lock();
                    lock.lifecycle_state = RuntimeLifecycleState::Stopped;
                    break;
                }
                _ => {}
            }
        }
    })
}

fn spawn_stderr_task(
    app: AppHandle,
    plant_id: String,
    runtime_id: String,
    stderr: std::process::ChildStderr,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            if let Ok(line) = line {
                if !line.trim().is_empty() {
                    let _ = emit_error_event(&app, &plant_id, &runtime_id, line);
                }
            }
        }
    })
}

fn process_telemetry(
    app: &AppHandle,
    plant_store: &PlantStore,
    plant_id: &str,
    runtime_id: &str,
    configured_sample_time_ms: u64,
    payload: Value,
    metrics: &SharedMetrics,
) {
    let effective_dt_ms = payload
        .get("effective_dt_ms")
        .and_then(Value::as_f64)
        .unwrap_or(configured_sample_time_ms as f64);
    let cycle_duration_ms = payload
        .get("cycle_duration_ms")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let read_duration_ms = payload
        .get("read_duration_ms")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let cycle_late = payload
        .get("cycle_late")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let uptime = payload
        .get("uptime_s")
        .and_then(Value::as_f64)
        .unwrap_or(0.0)
        .max(0.0) as u64;

    {
        let mut lock = metrics.lock();
        lock.lifecycle_state = RuntimeLifecycleState::Running;
        lock.cycle_phase = RuntimeCyclePhase::PublishTelemetry;
        lock.effective_dt_ms = effective_dt_ms;
        lock.cycle_duration_ms = cycle_duration_ms;
        lock.read_duration_ms = read_duration_ms;
        lock.cycle_late = cycle_late;
        if cycle_late {
            lock.late_cycle_count = lock.late_cycle_count.saturating_add(1);
        }
        lock.last_telemetry_at = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|time| time.as_secs())
                .unwrap_or(0),
        );
    }

    let _ = plant_store.update(plant_id, |plant| {
        plant.stats.dt = (effective_dt_ms / 1000.0).max(0.0);
        plant.stats.uptime = uptime;
    });

    let event = RuntimeTelemetryEvent {
        plant_id: plant_id.to_string(),
        runtime_id: runtime_id.to_string(),
        lifecycle_state: RuntimeLifecycleState::Running,
        cycle_phase: RuntimeCyclePhase::PublishTelemetry,
        configured_sample_time_ms,
        effective_dt_ms,
        cycle_late,
        payload,
    };
    let _ = app.emit("plant://telemetry", event);
}

fn wait_for_handshake(handshake: &SharedHandshake, timeout: Duration) -> AppResult<()> {
    let deadline = Instant::now() + timeout;

    let mut guard = handshake.0.lock();
    loop {
        if guard.connected {
            return Ok(());
        }
        if let Some(message) = guard.error.clone() {
            return Err(AppError::IoError(format!(
                "Falha durante handshake da runtime: {message}"
            )));
        }

        let now = Instant::now();
        if now >= deadline {
            return Err(AppError::IoError(
                "Timeout aguardando handshake da runtime Python".into(),
            ));
        }

        let wait_for = deadline.saturating_duration_since(now);
        if handshake.1.wait_for(&mut guard, wait_for).timed_out() {
            return Err(AppError::IoError(
                "Timeout aguardando handshake da runtime Python".into(),
            ));
        }
    }
}

fn send_command(
    stdin: &Arc<Mutex<ChildStdin>>,
    msg_type: &str,
    payload: Option<Value>,
) -> AppResult<()> {
    let mut writer = stdin.lock();
    let mut envelope = serde_json::Map::new();
    envelope.insert("type".to_string(), Value::String(msg_type.to_string()));
    if let Some(payload) = payload {
        envelope.insert("payload".to_string(), payload);
    }

    let line = serde_json::to_string(&envelope).map_err(|error| {
        AppError::IoError(format!("Falha ao serializar comando para runtime: {error}"))
    })?;
    writer.write_all(line.as_bytes()).map_err(|error| {
        AppError::IoError(format!("Falha ao enviar comando para runtime: {error}"))
    })?;
    writer.write_all(b"\n").map_err(|error| {
        AppError::IoError(format!("Falha ao finalizar comando para runtime: {error}"))
    })?;
    writer.flush().map_err(|error| {
        AppError::IoError(format!("Falha ao flush de comando para runtime: {error}"))
    })?;

    Ok(())
}

fn emit_status_event(app: &AppHandle, event: RuntimeStatusEvent) {
    let _ = app.emit("plant://status", event);
}

fn emit_error_event(
    app: &AppHandle,
    plant_id: &str,
    runtime_id: &str,
    message: String,
) -> AppResult<()> {
    app.emit(
        "plant://error",
        json!({
            "plant_id": plant_id,
            "runtime_id": runtime_id,
            "message": message,
        }),
    )
    .map_err(|error| AppError::IoError(format!("Falha ao emitir evento de erro: {error}")))?;

    Ok(())
}

fn ensure_python_env(driver: &PluginRegistry, env_hash: &str) -> AppResult<PathBuf> {
    let env_dir = WorkspaceService::env_directory(env_hash)?;
    fs::create_dir_all(&env_dir).map_err(|error| {
        AppError::IoError(format!(
            "Falha ao criar diretório de ambiente '{}': {error}",
            env_dir.display()
        ))
    })?;

    let venv_dir = env_dir.join(".venv");
    let venv_python = venv_python_path(&venv_dir);
    if !venv_python.exists() {
        let python_cmd = resolve_system_python()?;
        run_command(
            Command::new(&python_cmd)
                .arg("-m")
                .arg("venv")
                .arg(&venv_dir),
            "Falha ao criar ambiente Python isolado",
        )?;

        if !driver.dependencies.is_empty() {
            let specs: Vec<String> = driver
                .dependencies
                .iter()
                .map(|dependency| {
                    if dependency.version.trim().is_empty() {
                        dependency.name.clone()
                    } else {
                        format!("{}=={}", dependency.name, dependency.version)
                    }
                })
                .collect();

            run_command(
                Command::new(&venv_python)
                    .arg("-m")
                    .arg("pip")
                    .arg("install")
                    .arg("--disable-pip-version-check")
                    .args(specs.clone()),
                "Falha ao instalar dependências do driver",
            )?;

            let lock_path = env_dir.join("requirements.lock.txt");
            fs::write(&lock_path, specs.join("\n")).map_err(|error| {
                AppError::IoError(format!(
                    "Falha ao gravar requirements.lock.txt em '{}': {error}",
                    lock_path.display()
                ))
            })?;
        }
    }

    let metadata_path = env_dir.join("metadata.json");
    let metadata_payload = json!({
        "env_hash": env_hash,
        "runtime": format!("{:?}", driver.runtime),
        "dependencies": driver.dependencies,
    });
    fs::write(
        &metadata_path,
        serde_json::to_string_pretty(&metadata_payload).map_err(|error| {
            AppError::IoError(format!("Falha ao serializar metadata.json: {error}"))
        })?,
    )
    .map_err(|error| {
        AppError::IoError(format!(
            "Falha ao gravar metadata.json em '{}': {error}",
            metadata_path.display()
        ))
    })?;

    Ok(venv_python)
}

fn venv_python_path(venv_dir: &Path) -> PathBuf {
    if cfg!(target_os = "windows") {
        venv_dir.join("Scripts").join("python.exe")
    } else {
        venv_dir.join("bin").join("python")
    }
}

fn resolve_system_python() -> AppResult<String> {
    for candidate in ["python3", "python"] {
        let output = Command::new(candidate).arg("--version").output();
        if output.is_ok() {
            return Ok(candidate.to_string());
        }
    }

    Err(AppError::IoError(
        "Python não encontrado no sistema para criação da runtime".into(),
    ))
}

fn run_command(command: &mut Command, context: &str) -> AppResult<()> {
    let output = command.output().map_err(|error| {
        AppError::IoError(format!("{context}: falha ao executar comando: {error}"))
    })?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    Err(AppError::IoError(format!(
        "{context}: status={} stdout='{}' stderr='{}'",
        output.status,
        stdout.trim(),
        stderr.trim()
    )))
}

fn compute_env_hash(driver: &PluginRegistry) -> String {
    let mut dependencies = driver.dependencies.clone();
    dependencies.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then(left.version.cmp(&right.version))
    });

    let mut material = format!("runtime={:?}\nformat=v1\n", driver.runtime);
    for dependency in dependencies {
        material.push_str(&dependency.name);
        material.push('=');
        material.push_str(&dependency.version);
        material.push('\n');
    }

    let hash = fnv1a_64(material.as_bytes());
    format!("{hash:016x}")
}

fn to_driver_class_name(plugin_name: &str) -> String {
    let trimmed = plugin_name.trim();
    if trimmed.is_empty() {
        return "MyDriver".to_string();
    }

    let filtered: String = trimmed
        .chars()
        .filter(|character| {
            character.is_ascii_alphanumeric()
                || character.is_ascii_whitespace()
                || *character == '_'
        })
        .collect();

    let mut class_name = String::new();
    for token in filtered
        .split(|character: char| character.is_ascii_whitespace() || character == '_')
        .filter(|token| !token.is_empty())
    {
        let mut chars = token.chars();
        if let Some(first) = chars.next() {
            class_name.push(first.to_ascii_uppercase());
            for character in chars {
                class_name.push(character.to_ascii_lowercase());
            }
        }
    }

    if class_name.is_empty() {
        "MyDriver".to_string()
    } else {
        class_name
    }
}

fn fnv1a_64(data: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;

    let mut hash = OFFSET;
    for byte in data {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(PRIME);
    }

    hash
}
