use super::{DriverBootstrapPayload, RUNNER_SCRIPT};
use crate::core::error::{AppError, AppResult};
use crate::core::models::plugin::PluginRegistry;
use crate::core::services::workspace::WorkspaceService;
use serde_json::json;
use std::collections::{BTreeSet, HashSet};
use std::fs;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

const PYTHON_ENV_VARS_TO_CLEAR: &[&str] = &[
    "PYTHONHOME",
    "PYTHONPATH",
    "VIRTUAL_ENV",
    "__PYVENV_LAUNCHER__",
    "PYTHONEXECUTABLE",
];
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub(super) fn prepare_python_command(command: &mut Command) -> &mut Command {
    for key in PYTHON_ENV_VARS_TO_CLEAR {
        command.env_remove(key);
    }
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

pub(super) fn prepare_runtime_directory() -> AppResult<PathBuf> {
    let runtime_root = WorkspaceService::runtime_root_directory()?;
    fs::create_dir_all(&runtime_root).map_err(|error| {
        AppError::IoError(format!(
            "Falha ao criar diretório de runtimes '{}': {error}",
            runtime_root.display()
        ))
    })?;
    Ok(runtime_root)
}

pub(super) fn prepare_runtime_scaffold(runtime_dir: &Path) -> AppResult<()> {
    fs::create_dir_all(runtime_dir).map_err(|error| {
        AppError::IoError(format!(
            "Falha ao criar diretório da runtime '{}': {error}",
            runtime_dir.display()
        ))
    })?;
    Ok(())
}

pub(super) fn write_bootstrap_files(
    bootstrap: &DriverBootstrapPayload,
    bootstrap_path: &Path,
) -> AppResult<()> {
    write_if_changed(
        bootstrap_path,
        &serde_json::to_string(bootstrap).map_err(|error| {
            AppError::IoError(format!("Falha ao serializar bootstrap.json: {error}"))
        })?,
        &format!(
            "Falha ao gravar bootstrap.json em '{}'",
            bootstrap_path.display()
        ),
    )?;

    Ok(())
}

pub(super) fn write_runner_script(runtime_root: &Path) -> AppResult<PathBuf> {
    let runner_path = runtime_root.join("runner.py");
    write_if_changed(
        &runner_path,
        RUNNER_SCRIPT,
        &format!(
            "Falha ao gravar runner Python em '{}'",
            runner_path.display()
        ),
    )?;
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
    let should_recreate_env = !venv_python.exists() || check_python_runtime(&venv_python).is_err();

    if should_recreate_env {
        if venv_dir.exists() {
            fs::remove_dir_all(&venv_dir).map_err(|error| {
                AppError::IoError(format!(
                    "Falha ao recriar ambiente Python isolado '{}': {error}",
                    venv_dir.display()
                ))
            })?;
        }

        let python_cmd = find_system_python()?;
        let mut create_venv = Command::new(&python_cmd);
        prepare_python_command(create_venv.arg("-m").arg("venv").arg(&venv_dir));
        run_command(&mut create_venv, "Falha ao criar ambiente Python isolado")?;
        check_python_runtime(&venv_python)?;

        let specs = collect_dependency_specs(runtime_plugins);
        if !specs.is_empty() {
            let mut install_deps = Command::new(&venv_python);
            prepare_python_command(
                install_deps
                    .arg("-m")
                    .arg("pip")
                    .arg("install")
                    .arg("--disable-pip-version-check")
                    .args(specs.clone()),
            );
            run_command(
                &mut install_deps,
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
    let metadata_contents = serde_json::to_string_pretty(&metadata_payload).map_err(|error| {
        AppError::IoError(format!("Falha ao serializar metadata.json: {error}"))
    })?;
    write_if_changed(
        &metadata_path,
        &metadata_contents,
        &format!(
            "Falha ao gravar metadata.json em '{}'",
            metadata_path.display()
        ),
    )?;

    Ok(venv_python)
}

fn venv_python_path(venv_dir: &Path) -> PathBuf {
    if cfg!(target_os = "windows") {
        venv_dir.join("Scripts").join("python.exe")
    } else {
        venv_dir.join("bin").join("python")
    }
}

fn find_system_python() -> AppResult<String> {
    for candidate in ["python3", "python"] {
        let mut command = Command::new(candidate);
        prepare_python_command(command.arg("--version"));
        if command
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
        {
            return Ok(candidate.to_string());
        }
    }

    Err(AppError::IoError(
        "Python não encontrado no sistema para criação da runtime".into(),
    ))
}

fn check_python_runtime(python_path: &Path) -> AppResult<()> {
    let mut command = Command::new(python_path);
    prepare_python_command(command.arg("-c").arg("import encodings, tokenize, venv"));
    run_command(
        &mut command,
        &format!(
            "Falha ao validar interpretador Python isolado '{}'",
            python_path.display()
        ),
    )
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

pub(super) fn dedupe_runtime_plugins(runtime_plugins: Vec<PluginRegistry>) -> Vec<PluginRegistry> {
    let mut seen = HashSet::new();
    let mut plugins: Vec<PluginRegistry> = Vec::with_capacity(runtime_plugins.len());
    for plugin in runtime_plugins {
        if seen.insert(plugin.id.clone()) {
            plugins.push(plugin);
        }
    }
    plugins
}

fn collect_dependency_specs(runtime_plugins: &[PluginRegistry]) -> Vec<String> {
    runtime_plugins
        .iter()
        .flat_map(|plugin| plugin.dependencies.iter())
        .map(|dependency| {
            if dependency.version.trim().is_empty() {
                dependency.name.clone()
            } else {
                format!("{}=={}", dependency.name, dependency.version)
            }
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
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
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0100_0000_01b3;

    let mut hash = OFFSET;
    for byte in data {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }

    hash
}

fn write_if_changed(path: &Path, contents: &str, context: &str) -> AppResult<()> {
    let should_write = match fs::read_to_string(path) {
        Ok(existing) => existing != contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => true,
        Err(error) => {
            return Err(AppError::IoError(format!(
                "Falha ao ler '{}' antes da atualização: {error}",
                path.display()
            )))
        }
    };

    if !should_write {
        return Ok(());
    }

    fs::write(path, contents).map_err(|error| AppError::IoError(format!("{context}: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::plugin::{PluginRegistry, PluginRuntime, PluginType};
    use std::fs;
    use std::process::Command;
    use std::thread;
    use std::time::Duration;

    fn create_plugin(id: &str, name: &str) -> PluginRegistry {
        PluginRegistry {
            id: id.to_string(),
            name: name.to_string(),
            plugin_type: PluginType::Driver,
            runtime: PluginRuntime::Python,
            entry_class: "Driver".to_string(),
            schema: vec![],
            source_file: Some("main.py".to_string()),
            source_code: None,
            dependencies: vec![],
            description: None,
            version: None,
            author: None,
        }
    }

    #[test]
    fn dedupe_runtime_plugins_preserves_single_instance_per_id() {
        let deduped = dedupe_runtime_plugins(vec![
            create_plugin("plugin_a", "Driver A"),
            create_plugin("plugin_a", "Driver A"),
            create_plugin("plugin_b", "Driver B"),
        ]);

        assert_eq!(deduped.len(), 2);
        assert_eq!(deduped[0].id, "plugin_a");
        assert_eq!(deduped[1].id, "plugin_b");
    }

    #[test]
    fn write_if_changed_preserves_timestamp_when_contents_match() {
        let test_dir = std::env::temp_dir().join("senamby-runtime-env-tests");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let path = test_dir.join("metadata.json");
        write_if_changed(&path, "{\"runtime\":true}", "falha de teste").unwrap();
        let initial_modified = fs::metadata(&path).unwrap().modified().unwrap();

        thread::sleep(Duration::from_millis(20));
        write_if_changed(&path, "{\"runtime\":true}", "falha de teste").unwrap();
        let second_modified = fs::metadata(&path).unwrap().modified().unwrap();

        assert_eq!(initial_modified, second_modified);

        thread::sleep(Duration::from_millis(20));
        write_if_changed(&path, "{\"runtime\":false}", "falha de teste").unwrap();
        let third_modified = fs::metadata(&path).unwrap().modified().unwrap();

        assert!(third_modified > second_modified);
    }

    #[test]
    fn prepare_python_command_ignores_invalid_pythonhome() {
        let python = find_system_python().expect("python nao encontrado para teste");
        let mut command = Command::new(&python);
        command.env("PYTHONHOME", "/tmp/senamby-invalid-python-home");
        prepare_python_command(command.arg("-c").arg("import encodings"));

        let output = command.output().expect("falha ao executar python saneado");
        assert!(
            output.status.success(),
            "stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
