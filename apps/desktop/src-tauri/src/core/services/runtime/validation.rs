use super::{
    PYTHON_CLASS_METHOD_CHECK_SCRIPT, PYTHON_IMPORT_CLASS_CHECK_SCRIPT, PYTHON_SYNTAX_CHECK_SCRIPT,
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
    let output = Command::new(python_path)
        .arg("-c")
        .arg(PYTHON_SYNTAX_CHECK_SCRIPT)
        .arg(source_path)
        .output()
        .map_err(|error| {
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
    let detail = stderr
        .lines()
        .chain(stdout.lines())
        .map(str::trim)
        .rfind(|line| !line.is_empty())
        .unwrap_or("Erro de sintaxe desconhecido no plugin Python");

    Err(AppError::InvalidArgument(format!(
        "Código Python inválido no {component_label}: {detail}"
    )))
}

fn validate_python_class_methods(
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

    let output = Command::new("python3")
        .arg("-c")
        .arg(PYTHON_CLASS_METHOD_CHECK_SCRIPT)
        .arg(source_path)
        .arg(class_name)
        .arg(methods_json)
        .output()
        .map_err(|error| {
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
    let detail = stderr
        .lines()
        .chain(stdout.lines())
        .map(str::trim)
        .rfind(|line| !line.is_empty())
        .unwrap_or("Métodos obrigatórios não encontrados");

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
    command
        .arg("-c")
        .arg(PYTHON_IMPORT_CLASS_CHECK_SCRIPT)
        .arg(source_path)
        .arg(class_name);

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
    let detail = stderr
        .lines()
        .chain(stdout.lines())
        .map(str::trim)
        .rfind(|line| !line.is_empty())
        .unwrap_or("Falha ao carregar classe Python");

    Err(AppError::InvalidArgument(format!(
        "Falha ao carregar {component_label}: {detail}"
    )))
}

pub(super) fn ensure_driver_supports_write(driver_plugin: &PluginRegistry) -> AppResult<()> {
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

pub(super) fn validate_controller_plugin_source(
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

    validate_python_source_file(Path::new("python3"), &source_path, &component_label)?;
    validate_python_class_methods(
        &source_path,
        &controller_plugin.entry_class,
        &["compute"],
        &component_label,
    )?;

    Ok(())
}

pub(super) fn validate_controller_plugin_source_with_python(
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
