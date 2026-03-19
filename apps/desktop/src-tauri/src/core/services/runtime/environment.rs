use super::{DriverBootstrapPayload, RUNNER_SCRIPT};
use crate::core::error::{AppError, AppResult};
use crate::core::models::plugin::PluginRegistry;
use crate::core::services::workspace::WorkspaceService;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn prepare_runtime_directory() -> AppResult<PathBuf> {
    let seed_runtime_root = WorkspaceService::runtime_directory("seed_runtime")?;
    let runtime_root = seed_runtime_root
        .parent()
        .map(Path::to_path_buf)
        .ok_or(AppError::InternalError)?;
    fs::create_dir_all(&runtime_root).map_err(|error| {
        AppError::IoError(format!(
            "Falha ao criar diretório de runtimes '{}': {error}",
            runtime_root.display()
        ))
    })?;
    Ok(runtime_root)
}

pub(super) fn prepare_runtime_scaffold(runtime_dir: &Path) -> AppResult<()> {
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

pub(super) fn write_bootstrap_files(
    runtime_dir: &Path,
    bootstrap: &DriverBootstrapPayload,
    bootstrap_path: &Path,
) -> AppResult<()> {
    let runtime_path = runtime_dir.join("runtime.json");
    let runtime_payload = json!({
        "runtime": bootstrap.runtime,
        "created_at": SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|time| time.as_secs())
            .unwrap_or(0),
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
        serde_json::to_string_pretty(&bootstrap.plant).map_err(|error| {
            AppError::IoError(format!("Falha ao serializar plant.json: {error}"))
        })?,
    )
    .map_err(|error| {
        AppError::IoError(format!(
            "Falha ao gravar plant.json em '{}': {error}",
            plant_path.display()
        ))
    })?;

    fs::write(
        bootstrap_path,
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

pub(super) fn write_runner_script(runtime_dir: &Path) -> AppResult<PathBuf> {
    let runner_path = runtime_dir.join("runner.py");
    fs::write(&runner_path, RUNNER_SCRIPT).map_err(|error| {
        AppError::IoError(format!(
            "Falha ao gravar runner Python em '{}': {error}",
            runner_path.display()
        ))
    })?;
    Ok(runner_path)
}

pub(super) fn ensure_python_env(
    runtime_plugins: &[PluginRegistry],
    env_hash: &str,
) -> AppResult<PathBuf> {
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

        let specs = collect_runtime_dependency_specs(runtime_plugins);
        if !specs.is_empty() {
            run_command(
                Command::new(&venv_python)
                    .arg("-m")
                    .arg("pip")
                    .arg("install")
                    .arg("--disable-pip-version-check")
                    .args(specs.clone()),
                "Falha ao instalar dependências da runtime da planta",
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
        "runtime": "python",
        "plugins": runtime_plugins
            .iter()
            .map(|plugin| json!({
                "id": plugin.id,
                "name": plugin.name,
                "type": plugin.plugin_type,
                "runtime": plugin.runtime,
                "entry_class": plugin.entry_class,
            }))
            .collect::<Vec<_>>(),
        "dependencies": runtime_plugins
            .iter()
            .flat_map(|plugin| plugin.dependencies.iter().map(|dependency| json!({
                "plugin_id": plugin.id,
                "name": dependency.name,
                "version": dependency.version,
            })))
            .collect::<Vec<_>>(),
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
        if Command::new(candidate).arg("--version").output().is_ok() {
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

pub(super) fn collect_runtime_plugins(runtime_plugins: &[PluginRegistry]) -> Vec<PluginRegistry> {
    let mut plugins: Vec<PluginRegistry> = Vec::with_capacity(runtime_plugins.len());
    for plugin in runtime_plugins {
        if !plugins.iter().any(|existing| existing.id == plugin.id) {
            plugins.push(plugin.clone());
        }
    }
    plugins
}

fn collect_runtime_dependency_specs(runtime_plugins: &[PluginRegistry]) -> Vec<String> {
    let mut specs = runtime_plugins
        .iter()
        .flat_map(|plugin| plugin.dependencies.iter())
        .map(|dependency| {
            if dependency.version.trim().is_empty() {
                dependency.name.clone()
            } else {
                format!("{}=={}", dependency.name, dependency.version)
            }
        })
        .collect::<Vec<_>>();
    specs.sort();
    specs.dedup();
    specs
}

pub(super) fn compute_env_hash(runtime_plugins: &[PluginRegistry]) -> String {
    let mut dependencies = runtime_plugins
        .iter()
        .flat_map(|plugin| {
            plugin
                .dependencies
                .iter()
                .map(|dependency| (dependency.name.clone(), dependency.version.clone()))
        })
        .collect::<Vec<_>>();
    dependencies.sort();
    dependencies.dedup();

    let mut material = "runtime=python\nformat=v2\n".to_string();
    for plugin in runtime_plugins {
        material.push_str("plugin=");
        material.push_str(&plugin.id);
        material.push('|');
        material.push_str(&plugin.entry_class);
        material.push('\n');
    }
    for (name, version) in dependencies {
        material.push_str(&name);
        material.push('=');
        material.push_str(&version);
        material.push('\n');
    }

    let hash = fnv1a_64(material.as_bytes());
    format!("{hash:016x}")
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
