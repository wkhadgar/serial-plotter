use crate::core::error::{AppError, AppResult};
use crate::core::models::plant::{
    CreatePlantControllerRequest, CreatePlantDriverRequest, CreatePlantVariableRequest,
    VariableType,
};
use crate::core::models::plugin::{PluginRegistry, PluginType};
use crate::state::{PlantStore, PluginStore};
use std::collections::HashMap;

pub(super) fn validate_payload(
    current_id: Option<&str>,
    name: &str,
    sample_time_ms: u64,
    variables: &[CreatePlantVariableRequest],
    driver: &CreatePlantDriverRequest,
    controllers: &[CreatePlantControllerRequest],
    store: &PlantStore,
    plugins: &PluginStore,
) -> AppResult<()> {
    if name.trim().is_empty() {
        return Err(AppError::InvalidArgument(
            "Nome da planta é obrigatório".into(),
        ));
    }

    let has_duplicate_name = current_id
        .map(|id| store.exists_by_name_except(id, name))
        .unwrap_or_else(|| store.exists_by_name(name));

    if has_duplicate_name {
        return Err(AppError::InvalidArgument(format!(
            "Planta com NOME '{}' já existe",
            name
        )));
    }

    if variables.is_empty() {
        return Err(AppError::InvalidArgument(
            "Pelo menos uma variável deve ser definida".into(),
        ));
    }

    if sample_time_ms == 0 {
        return Err(AppError::InvalidArgument(
            "Tempo de amostragem deve ser maior que 0 ms".into(),
        ));
    }

    if driver.plugin_id.trim().is_empty() {
        return Err(AppError::InvalidArgument(
            "Um driver de comunicação é obrigatório".into(),
        ));
    }

    resolve_plugin(plugins, &driver.plugin_id, PluginType::Driver)?;

    for (idx, var) in variables.iter().enumerate() {
        validate_variable(var).map_err(|error| {
            AppError::InvalidArgument(format!("Variável {} inválida: {}", idx + 1, error))
        })?;
    }

    for (idx, controller) in controllers.iter().enumerate() {
        validate_controller(controller, variables, plugins).map_err(|error| {
            AppError::InvalidArgument(format!("Controlador {} inválido: {}", idx + 1, error))
        })?;
    }

    Ok(())
}

pub(super) fn validate_variable(var: &CreatePlantVariableRequest) -> AppResult<()> {
    if var.name.trim().is_empty() {
        return Err(AppError::InvalidArgument(
            "Nome da variável é obrigatório".into(),
        ));
    }

    if var.pv_min >= var.pv_max {
        return Err(AppError::InvalidArgument(
            "pv_min deve ser menor que pv_max".into(),
        ));
    }

    if var.setpoint < var.pv_min || var.setpoint > var.pv_max {
        return Err(AppError::InvalidArgument(
            "setpoint deve estar entre pv_min e pv_max".into(),
        ));
    }

    Ok(())
}

pub(super) fn validate_controller(
    controller: &CreatePlantControllerRequest,
    variables: &[CreatePlantVariableRequest],
    plugins: &PluginStore,
) -> AppResult<()> {
    if controller.plugin_id.trim().is_empty() {
        return Err(AppError::InvalidArgument(
            "Plugin do controlador é obrigatório".into(),
        ));
    }

    if controller.name.trim().is_empty() {
        return Err(AppError::InvalidArgument(
            "Nome do controlador é obrigatório".into(),
        ));
    }

    if controller.controller_type.trim().is_empty() {
        return Err(AppError::InvalidArgument(
            "Tipo do controlador é obrigatório".into(),
        ));
    }

    if controller.input_variable_ids.is_empty() {
        return Err(AppError::InvalidArgument(
            "O controlador precisa de pelo menos uma variável de entrada".into(),
        ));
    }

    if controller.output_variable_ids.is_empty() {
        return Err(AppError::InvalidArgument(
            "O controlador precisa de pelo menos uma variável de saída".into(),
        ));
    }

    let variable_types = build_variable_type_map(variables);

    for input_id in &controller.input_variable_ids {
        match variable_types.get(input_id) {
            Some(VariableType::Sensor) => {}
            Some(VariableType::Atuador) => {
                return Err(AppError::InvalidArgument(format!(
                    "A variável '{}' não pode ser usada como entrada",
                    input_id
                )));
            }
            None => {
                return Err(AppError::InvalidArgument(format!(
                    "Variável de entrada '{}' não existe",
                    input_id
                )));
            }
        }
    }

    for output_id in &controller.output_variable_ids {
        match variable_types.get(output_id) {
            Some(VariableType::Atuador) => {}
            Some(VariableType::Sensor) => {
                return Err(AppError::InvalidArgument(format!(
                    "A variável '{}' não pode ser usada como saída",
                    output_id
                )));
            }
            None => {
                return Err(AppError::InvalidArgument(format!(
                    "Variável de saída '{}' não existe",
                    output_id
                )));
            }
        }
    }

    resolve_plugin(plugins, &controller.plugin_id, PluginType::Controller)?;
    Ok(())
}

pub(super) fn validate_active_controller_conflicts(
    controllers: &[CreatePlantControllerRequest],
) -> AppResult<()> {
    let mut ownership: HashMap<&str, &str> = HashMap::new();

    for controller in controllers.iter().filter(|controller| controller.active) {
        for output_id in &controller.output_variable_ids {
            if let Some(existing_controller) =
                ownership.insert(output_id.as_str(), controller.name.trim())
            {
                return Err(AppError::InvalidArgument(format!(
                    "A saída '{}' não pode ser controlada ao mesmo tempo por '{}' e '{}'",
                    output_id, existing_controller, controller.name
                )));
            }
        }
    }

    Ok(())
}

pub(super) fn resolve_plugin(
    plugins: &PluginStore,
    plugin_id: &str,
    expected_type: PluginType,
) -> AppResult<PluginRegistry> {
    plugins.read(plugin_id, |plugin| {
        if plugin.plugin_type != expected_type {
            Err(AppError::InvalidArgument(format!(
                "Plugin '{}' não é do tipo {}",
                plugin.name,
                expected_type.as_label()
            )))
        } else {
            Ok(plugin.clone())
        }
    })?
}

fn build_variable_type_map(
    variables: &[CreatePlantVariableRequest],
) -> HashMap<String, VariableType> {
    variables
        .iter()
        .enumerate()
        .map(|(idx, variable)| (format!("var_{}", idx), variable.var_type))
        .collect()
}
