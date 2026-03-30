use crate::core::error::{AppError, AppResult};
use crate::core::models::plugin::PluginType;
use crate::core::services::workspace::WorkspaceService;
use std::path::PathBuf;

pub(super) fn resolve_source_code_for_create(
    source_code: Option<&str>,
    source_file: Option<&str>,
    plugin_name: &str,
    plugin_type: PluginType,
) -> AppResult<String> {
    if let Some(code) = normalize_source_code(source_code) {
        return Ok(code);
    }

    if let Some(file_name) = source_file {
        let maybe_path = PathBuf::from(file_name);
        if maybe_path.is_absolute() && maybe_path.exists() {
            let code = std::fs::read_to_string(&maybe_path).map_err(|error| {
                AppError::IoError(format!(
                    "Falha ao ler código fonte do arquivo '{}': {error}",
                    maybe_path.display()
                ))
            })?;
            if let Some(normalized) = normalize_source_code(Some(&code)) {
                return Ok(normalized);
            }
        }
    }

    if let Ok(code) = WorkspaceService::read_plugin_source(plugin_name, plugin_type) {
        if let Some(normalized) = normalize_source_code(Some(&code)) {
            return Ok(normalized);
        }
    }

    Err(AppError::InvalidArgument(
        "Código fonte Python é obrigatório. Informe sourceCode ou use um source_file absoluto válido."
            .into(),
    ))
}

pub(super) fn normalize_source_code(source_code: Option<&str>) -> Option<String> {
    source_code.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}
