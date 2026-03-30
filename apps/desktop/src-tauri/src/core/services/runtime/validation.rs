use super::{
    environment::prepare_python_command, PYTHON_CLASS_METHOD_CHECK_SCRIPT,
    PYTHON_IMPORT_CLASS_CHECK_SCRIPT, PYTHON_SYNTAX_CHECK_SCRIPT,
};
use crate::core::error::{AppError, AppResult};
use crate::core::models::plugin::{PluginRegistry, PluginRuntime, PluginType};
use crate::core::services::workspace::WorkspaceService;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(super) fn validate_plugin_workspace_files(
    plugin_dir: &Path,
    component_label: &str,
) -> AppResult<()> {
    let registry_path = plugin_dir.join("registry.json");
    if !registry_path.exists() {
        return Err(AppError::NotFound(format!(
            "registry.json do {component_label} não encontrado em '{}'",
            registry_path.display()
        )));
    }

    let main_path = plugin_dir.join("main.py");
    if !main_path.exists() {
        return Err(AppError::NotFound(format!(
            "main.py do {component_label} não encontrado em '{}'",
            main_path.display()
        )));
    }

    Ok(())
}

pub(super) fn validate_python_source_file(
    python_path: &Path,
    source_path: &Path,
    component_label: &str,
) -> AppResult<()> {
    let mut command = Command::new(python_path);
    prepare_python_command(
        command
            .arg("-c")
            .arg(PYTHON_SYNTAX_CHECK_SCRIPT)
            .arg(source_path),
    );
    let output = command.output().map_err(|error| {
        AppError::IoError(format!(
            "Falha ao validar código Python do {component_label} '{}': {error}",
            source_path.display()
        ))
    })?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let detail = extract_python_error_detail(
        &stderr,
        &stdout,
        "Erro de sintaxe desconhecido no plugin Python",
    );

    Err(AppError::InvalidArgument(format!(
        "Código Python inválido no {component_label}: {detail}"
    )))
}

fn validate_python_class_methods(
    python_path: &Path,
    source_path: &Path,
    class_name: &str,
    required_methods: &[&str],
    component_label: &str,
) -> AppResult<()> {
    let methods_json = serde_json::to_string(required_methods).map_err(|error| {
        AppError::IoError(format!(
            "Falha ao serializar métodos obrigatórios do {component_label}: {error}"
        ))
    })?;

    let mut command = Command::new(python_path);
    prepare_python_command(
        command
            .arg("-c")
            .arg(PYTHON_CLASS_METHOD_CHECK_SCRIPT)
            .arg(source_path)
            .arg(class_name)
            .arg(methods_json),
    );
    let output = command.output().map_err(|error| {
        AppError::IoError(format!(
            "Falha ao validar métodos Python do {component_label} '{}': {error}",
            source_path.display()
        ))
    })?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let detail =
        extract_python_error_detail(&stderr, &stdout, "Métodos obrigatórios não encontrados");

    Err(AppError::InvalidArgument(format!(
        "Contrato inválido no {component_label}: {detail}"
    )))
}

fn validate_python_importable_class(
    python_path: &Path,
    source_path: &Path,
    class_name: &str,
    required_methods: &[&str],
    component_label: &str,
) -> AppResult<()> {
    let mut command = Command::new(python_path);
    prepare_python_command(
        command
            .arg("-c")
            .arg(PYTHON_IMPORT_CLASS_CHECK_SCRIPT)
            .arg(source_path)
            .arg(class_name),
    );

    for method in required_methods {
        command.arg(method);
    }

    let output = command.output().map_err(|error| {
        AppError::IoError(format!(
            "Falha ao validar carregamento Python do {component_label} '{}': {error}",
            source_path.display()
        ))
    })?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let detail = extract_python_error_detail(&stderr, &stdout, "Falha ao carregar classe Python");

    Err(AppError::InvalidArgument(format!(
        "Falha ao carregar {component_label}: {detail}"
    )))
}

pub(super) fn ensure_driver_supports_write(driver_plugin: &PluginRegistry) -> AppResult<()> {
    validate_driver_write_support(Path::new("python3"), driver_plugin)
}

pub(super) fn validate_driver_write_support(
    python_path: &Path,
    driver_plugin: &PluginRegistry,
) -> AppResult<()> {
    if driver_plugin.runtime != PluginRuntime::Python {
        return Err(AppError::InvalidArgument(
            "A runtime atual suporta apenas drivers Python".into(),
        ));
    }

    let source_path = resolve_plugin_source_path(driver_plugin, PluginType::Driver)?;
    if !source_path.exists() {
        return Err(AppError::NotFound(format!(
            "Arquivo fonte do driver não encontrado em '{}'",
            source_path.display()
        )));
    }

    validate_python_class_methods(
        python_path,
        &source_path,
        &driver_plugin.entry_class,
        &["write"],
        "driver",
    )
    .map_err(|_| {
        AppError::InvalidArgument(
            "Driver precisa implementar write(outputs) para executar controladores ativos".into(),
        )
    })
}

pub(super) fn validate_runtime_controller(
    python_path: &Path,
    controller_plugin: &PluginRegistry,
    controller_name: &str,
) -> AppResult<()> {
    if controller_plugin.runtime != PluginRuntime::Python {
        return Err(AppError::InvalidArgument(format!(
            "O controlador '{controller_name}' precisa ser Python para executar na runtime atual"
        )));
    }

    let controller_dir =
        WorkspaceService::plugin_directory(&controller_plugin.name, PluginType::Controller)?;
    validate_plugin_workspace_files(&controller_dir, "controlador")?;

    let source_path = resolve_plugin_source_path(controller_plugin, PluginType::Controller)?;
    let component_label = format!("controlador '{controller_name}'");

    validate_python_source_file(python_path, &source_path, &component_label)?;
    validate_python_importable_class(
        python_path,
        &source_path,
        &controller_plugin.entry_class,
        &["compute"],
        &component_label,
    )?;

    Ok(())
}

fn extract_python_error_detail(stderr: &str, stdout: &str, fallback: &str) -> String {
    let lines = stderr
        .lines()
        .chain(stdout.lines())
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    let detail = lines
        .iter()
        .rev()
        .copied()
        .find(|line| {
            *line != "<no Python frame>"
                && !line.starts_with("Current thread ")
                && !line.starts_with("Python runtime state:")
        })
        .or_else(|| lines.iter().rev().copied().find(|line| !line.is_empty()))
        .unwrap_or(fallback);

    detail.to_string()
}

fn resolve_plugin_source_path(
    plugin: &PluginRegistry,
    plugin_type: PluginType,
) -> AppResult<PathBuf> {
    let plugin_dir = WorkspaceService::plugin_directory(&plugin.name, plugin_type)?;
    let source_file = plugin
        .source_file
        .clone()
        .unwrap_or_else(|| "main.py".to_string());
    Ok(plugin_dir.join(source_file))
}

#[cfg(test)]
mod tests {
    use super::extract_python_error_detail;

    #[test]
    fn extract_python_error_detail_skips_no_python_frame_noise() {
        let stderr = "\
Fatal Python error: Failed to import encodings module
Python runtime state: core initialized
ModuleNotFoundError: No module named 'encodings'

Current thread 0x0000000000000000 [python3] (most recent call first):
  <no Python frame>
";

        let detail = extract_python_error_detail(stderr, "", "fallback");
        assert_eq!(detail, "ModuleNotFoundError: No module named 'encodings'");
    }

    #[test]
    fn extract_python_error_detail_keeps_useful_syntax_error_tail() {
        let stderr = "\
Traceback (most recent call last):
  File \"<string>\", line 1, in <module>
  File \"/tmp/plugin.py\", line 4
    return (
           ^
SyntaxError: '(' was never closed
";

        let detail = extract_python_error_detail(stderr, "", "fallback");
        assert_eq!(detail, "SyntaxError: '(' was never closed");
    }
}
