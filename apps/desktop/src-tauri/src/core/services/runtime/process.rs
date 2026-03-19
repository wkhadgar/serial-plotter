use super::{
    RuntimeCyclePhase, RuntimeEnvelope, RuntimeLifecycleState, RuntimeStatusEvent,
    RuntimeTelemetryEvent, SharedHandshake, SharedMetrics,
};
use crate::core::error::{AppError, AppResult};
use crate::state::PlantStore;
use parking_lot::Mutex;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};

pub(super) fn spawn_driver_process(
    venv_python_path: &std::path::Path,
    runner_path: &std::path::Path,
    runtime_dir: &std::path::Path,
    bootstrap_path: &std::path::Path,
) -> AppResult<Child> {
    Command::new(venv_python_path)
        .arg("-u")
        .arg(runner_path)
        .arg("--runtime-dir")
        .arg(runtime_dir)
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

pub(super) fn spawn_stdout_task(
    app: AppHandle,
    plant_store: Arc<PlantStore>,
    plant_id: String,
    runtime_id: String,
    configured_sample_time_ms: u64,
    stdout: ChildStdout,
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
                "ready" => handle_ready_event(
                    &app,
                    &plant_id,
                    &runtime_id,
                    configured_sample_time_ms,
                    &handshake,
                    &metrics,
                ),
                "connected" => handle_connected_event(
                    &app,
                    &plant_id,
                    &runtime_id,
                    configured_sample_time_ms,
                    &handshake,
                    &metrics,
                ),
                "error" => handle_runtime_error_event(
                    &app,
                    &plant_id,
                    &runtime_id,
                    configured_sample_time_ms,
                    envelope.payload,
                    &handshake,
                    &metrics,
                ),
                "telemetry" => process_telemetry(
                    &app,
                    &plant_store,
                    &plant_id,
                    &runtime_id,
                    configured_sample_time_ms,
                    envelope.payload,
                    &metrics,
                ),
                "cycle_overrun" => {
                    let mut lock = metrics.lock();
                    lock.cycle_late = true;
                    lock.late_cycle_count = lock.late_cycle_count.saturating_add(1);
                }
                "stopped" => {
                    handle_stopped_event(
                        &app,
                        &plant_id,
                        &runtime_id,
                        configured_sample_time_ms,
                        &metrics,
                    );
                    break;
                }
                _ => {}
            }
        }
    })
}

pub(super) fn spawn_stderr_task(
    plant_id: String,
    runtime_id: String,
    stderr: ChildStderr,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            if let Ok(line) = line {
                if !line.trim().is_empty() {
                    eprintln!(
                        "[driver-runtime][plant={}][runtime={}] {}",
                        plant_id, runtime_id, line
                    );
                }
            }
        }
    })
}

fn handle_ready_event(
    app: &AppHandle,
    plant_id: &str,
    runtime_id: &str,
    configured_sample_time_ms: u64,
    handshake: &SharedHandshake,
    metrics: &SharedMetrics,
) {
    {
        let mut lock = handshake.0.lock();
        lock.ready = true;
    }
    handshake.1.notify_all();

    let mut lock = metrics.lock();
    lock.lifecycle_state = RuntimeLifecycleState::Ready;
    emit_status_event(
        app,
        RuntimeStatusEvent {
            plant_id: plant_id.to_string(),
            runtime_id: runtime_id.to_string(),
            lifecycle_state: RuntimeLifecycleState::Ready,
            cycle_phase: RuntimeCyclePhase::CycleStarted,
            configured_sample_time_ms,
            effective_dt_ms: configured_sample_time_ms as f64,
            cycle_late: false,
        },
    );
}

fn handle_connected_event(
    app: &AppHandle,
    plant_id: &str,
    runtime_id: &str,
    configured_sample_time_ms: u64,
    handshake: &SharedHandshake,
    metrics: &SharedMetrics,
) {
    {
        let mut lock = handshake.0.lock();
        lock.connected = true;
    }
    handshake.1.notify_all();

    let mut lock = metrics.lock();
    lock.lifecycle_state = RuntimeLifecycleState::Running;
    lock.cycle_phase = RuntimeCyclePhase::ReadInputs;
    emit_status_event(
        app,
        RuntimeStatusEvent {
            plant_id: plant_id.to_string(),
            runtime_id: runtime_id.to_string(),
            lifecycle_state: RuntimeLifecycleState::Running,
            cycle_phase: RuntimeCyclePhase::ReadInputs,
            configured_sample_time_ms,
            effective_dt_ms: configured_sample_time_ms as f64,
            cycle_late: false,
        },
    );
}

fn handle_runtime_error_event(
    app: &AppHandle,
    plant_id: &str,
    runtime_id: &str,
    configured_sample_time_ms: u64,
    payload: Value,
    handshake: &SharedHandshake,
    metrics: &SharedMetrics,
) {
    let message = payload
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("Erro na runtime Python")
        .to_string();

    {
        let mut lock = handshake.0.lock();
        lock.error = Some(message.clone());
    }
    handshake.1.notify_all();

    let mut lock = metrics.lock();
    lock.lifecycle_state = RuntimeLifecycleState::Faulted;
    emit_status_event(
        app,
        RuntimeStatusEvent {
            plant_id: plant_id.to_string(),
            runtime_id: runtime_id.to_string(),
            lifecycle_state: RuntimeLifecycleState::Faulted,
            cycle_phase: lock.cycle_phase,
            configured_sample_time_ms,
            effective_dt_ms: lock.effective_dt_ms,
            cycle_late: lock.cycle_late,
        },
    );
    let _ = emit_error_event(app, plant_id, runtime_id, message);
}

fn handle_stopped_event(
    app: &AppHandle,
    plant_id: &str,
    runtime_id: &str,
    configured_sample_time_ms: u64,
    metrics: &SharedMetrics,
) {
    let mut lock = metrics.lock();
    lock.lifecycle_state = RuntimeLifecycleState::Stopped;
    emit_status_event(
        app,
        RuntimeStatusEvent {
            plant_id: plant_id.to_string(),
            runtime_id: runtime_id.to_string(),
            lifecycle_state: RuntimeLifecycleState::Stopped,
            cycle_phase: RuntimeCyclePhase::SleepUntilDeadline,
            configured_sample_time_ms,
            effective_dt_ms: lock.effective_dt_ms,
            cycle_late: false,
        },
    );
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

pub(super) fn wait_for_handshake(handshake: &SharedHandshake, timeout: Duration) -> AppResult<()> {
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

pub(super) fn send_command(
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

pub(super) fn emit_status_event(app: &AppHandle, event: RuntimeStatusEvent) {
    let _ = app.emit("plant://status", event);
}

pub(super) fn emit_error_event(
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
