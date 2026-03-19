use super::metadata::is_valid_entry_class;
use crate::core::error::{AppError, AppResult};
use crate::core::models::plugin::PluginRuntime;
use crate::state::PluginStore;

pub(super) fn validate_request(
    store: &PluginStore,
    current_id: Option<&str>,
    name: &str,
    runtime: PluginRuntime,
    entry_class: Option<&str>,
    source_file: Option<&str>,
    source_code: Option<&str>,
) -> AppResult<()> {
    if name.trim().is_empty() {
        return Err(AppError::InvalidArgument(
            "Nome do plugin é obrigatório".into(),
        ));
    }

    let has_duplicate_name = current_id
        .map(|id| store.exists_by_name_except(id, name))
        .unwrap_or_else(|| store.exists_by_name(name));

    if has_duplicate_name {
        return Err(AppError::InvalidArgument(format!(
            "Plugin com nome '{}' já existe",
            name.trim()
        )));
    }

    if runtime != PluginRuntime::Python {
        return Err(AppError::InvalidArgument(
            "Somente plugins Python podem ser criados no momento".into(),
        ));
    }

    let normalized_entry_class = entry_class
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or_default();
    if !normalized_entry_class.is_empty() && !is_valid_entry_class(normalized_entry_class) {
        return Err(AppError::InvalidArgument(
            "Classe principal do plugin é inválida".into(),
        ));
    }

    if source_code.is_some_and(|code| code.trim().is_empty()) {
        return Err(AppError::InvalidArgument(
            "Código fonte Python é obrigatório".into(),
        ));
    }

    if source_file.is_some_and(|file_name| file_name.trim().is_empty()) {
        return Err(AppError::InvalidArgument(
            "Nome do arquivo fonte inválido".into(),
        ));
    }

    Ok(())
}
