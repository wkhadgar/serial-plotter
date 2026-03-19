use super::{
    DriverBootstrapController, DriverBootstrapDriver, DriverBootstrapIoGroup,
    DriverBootstrapPayload, DriverBootstrapPlant, DriverBootstrapRuntime,
    DriverBootstrapRuntimePaths, DriverBootstrapRuntimeSupervision, DriverBootstrapRuntimeTiming,
    ResolvedRuntimeController,
};
use crate::core::error::{AppError, AppResult};
use crate::core::models::plant::{Plant, VariableType};
use crate::core::models::plugin::{PluginRegistry, PluginRuntime, PluginType};
use crate::core::services::plant::PlantService;
use crate::core::services::plugin::PluginService;
use crate::core::services::workspace::WorkspaceService;
use crate::state::{PlantStore, PluginStore};
use std::collections::HashMap;
use std::path::Path;

pub(super) fn resolve_runtime_components_for_connect(
    plants: &PlantStore,
    plugins: &PluginStore,
    mut plant: Plant,
) -> AppResult<(
    Plant,
    PluginRegistry,
    Vec<ResolvedRuntimeController>,
    Vec<PluginRegistry>,
)> {
    let mut loaded_from_workspace = false;
    let driver = resolve_plugin_for_runtime(
        plugins,
        &plant.driver.plugin_id,
        &plant.driver.plugin_name,
        PluginType::Driver,
        &mut loaded_from_workspace,
    )?
    .ok_or_else(|| {
        AppError::NotFound(format!(
            "Driver da planta '{}' não foi encontrado",
            plant.name
        ))
    })?;

    let mut plant_changed = plant.driver.plugin_id != driver.id
        || plant.driver.plugin_name != driver.name
        || plant.driver.runtime != driver.runtime
        || plant.driver.source_file != driver.source_file;

    if driver.runtime != PluginRuntime::Python {
        return Err(AppError::InvalidArgument(
            "A runtime atual suporta apenas drivers Python".into(),
        ));
    }

    if plant_changed {
        plant.driver.plugin_id = driver.id.clone();
        plant.driver.plugin_name = driver.name.clone();
        plant.driver.runtime = driver.runtime;
        plant.driver.source_file = driver.source_file.clone();
        plant.driver.source_code = None;
    }

    let mut active_controllers = Vec::new();
    let mut runtime_plugins = vec![driver.clone()];
    for controller in &mut plant.controllers {
        let resolved_plugin = resolve_plugin_for_runtime(
            plugins,
            &controller.plugin_id,
            &controller.plugin_name,
            PluginType::Controller,
            &mut loaded_from_workspace,
        )?;

        match resolved_plugin {
            Some(plugin) => {
                if PlantService::fill_missing_controller_params(&mut controller.params, &plugin) {
                    plant_changed = true;
                }

                if !runtime_plugins
                    .iter()
                    .any(|runtime_plugin| runtime_plugin.id == plugin.id)
                {
                    runtime_plugins.push(plugin.clone());
                }

                let controller_changed =
                    controller.plugin_id != plugin.id || controller.plugin_name != plugin.name;
                if controller_changed {
                    controller.plugin_id = plugin.id.clone();
                    controller.plugin_name = plugin.name.clone();
                    plant_changed = true;
                }

                if controller.active {
                    if plugin.runtime != PluginRuntime::Python {
                        return Err(AppError::InvalidArgument(format!(
                            "O controlador '{}' precisa ser Python para executar na runtime atual",
                            controller.name
                        )));
                    }

                    let plugin_dir =
                        WorkspaceService::plugin_directory(&plugin.name, PluginType::Controller)?;
                    active_controllers.push(ResolvedRuntimeController {
                        instance: controller.clone(),
                        plugin,
                        plugin_dir,
                    });
                }
            }
            None if controller.active => {
                return Err(AppError::NotFound(format!(
                    "Controlador '{}' da planta '{}' não está carregado",
                    controller.name, plant.name
                )));
            }
            None => {}
        }
    }

    if plant_changed {
        WorkspaceService::update_plant_registry(&plant, &plant.name)?;
        plants.replace(&plant.id, plant.clone())?;
    }

    Ok((plant, driver, active_controllers, runtime_plugins))
}

pub(super) fn resolve_plugin_for_runtime(
    plugins: &PluginStore,
    plugin_id: &str,
    plugin_name: &str,
    expected_type: PluginType,
    loaded_from_workspace: &mut bool,
) -> AppResult<Option<PluginRegistry>> {
    let find_by_name = |plugins: &PluginStore, plugin_name: &str| {
        plugins
            .list_by_type(expected_type)
            .into_iter()
            .find(|plugin| plugin.name.eq_ignore_ascii_case(plugin_name.trim()))
    };

    let try_resolve = |plugins: &PluginStore| -> AppResult<Option<PluginRegistry>> {
        match plugins.get(plugin_id) {
            Ok(plugin) if plugin.plugin_type == expected_type => Ok(Some(plugin)),
            Ok(_) | Err(AppError::NotFound(_)) if !plugin_name.trim().is_empty() => {
                Ok(find_by_name(plugins, plugin_name))
            }
            Ok(_) | Err(AppError::NotFound(_)) => Ok(None),
            Err(error) => Err(error),
        }
    };

    if let Some(plugin) = try_resolve(plugins)? {
        return Ok(Some(plugin));
    }

    if !*loaded_from_workspace {
        PluginService::load_all(plugins)?;
        *loaded_from_workspace = true;
        return try_resolve(plugins);
    }

    Ok(None)
}

pub(super) fn build_bootstrap_payload(
    runtime_id: &str,
    plant: &Plant,
    driver_plugin: &PluginRegistry,
    driver_dir: &Path,
    active_controllers: &[ResolvedRuntimeController],
    runtime_dir: &Path,
    venv_python_path: &Path,
    runner_path: &Path,
    bootstrap_path: &Path,
    startup_timeout_ms: u64,
    shutdown_timeout_ms: u64,
) -> AppResult<DriverBootstrapPayload> {
    let mut variables = Vec::new();
    let mut variables_by_id = HashMap::new();
    let mut sensor_ids = Vec::new();
    let mut actuator_ids = Vec::new();
    let mut sensor_variables = Vec::new();
    let mut sensor_variables_by_id = HashMap::new();
    let mut actuator_variables = Vec::new();
    let mut actuator_variables_by_id = HashMap::new();
    let mut setpoints = HashMap::new();

    for variable in &plant.variables {
        let serialized_variable = serde_json::to_value(variable).map_err(|error| {
            AppError::IoError(format!(
                "Falha ao serializar variável '{}' da planta: {error}",
                variable.id
            ))
        })?;

        variables.push(serialized_variable.clone());
        variables_by_id.insert(variable.id.clone(), serialized_variable.clone());
        setpoints.insert(variable.id.clone(), variable.setpoint);

        match variable.var_type {
            VariableType::Sensor => {
                sensor_ids.push(variable.id.clone());
                sensor_variables.push(serialized_variable.clone());
                sensor_variables_by_id.insert(variable.id.clone(), serialized_variable);
            }
            VariableType::Atuador => {
                actuator_ids.push(variable.id.clone());
                actuator_variables.push(serialized_variable.clone());
                actuator_variables_by_id.insert(variable.id.clone(), serialized_variable);
            }
        }
    }

    Ok(DriverBootstrapPayload {
        driver: DriverBootstrapDriver {
            plugin_id: driver_plugin.id.clone(),
            plugin_name: driver_plugin.name.clone(),
            plugin_dir: driver_dir.display().to_string(),
            source_file: driver_plugin
                .source_file
                .clone()
                .unwrap_or_else(|| "main.py".to_string()),
            class_name: driver_plugin.entry_class.clone(),
            config: serde_json::to_value(&plant.driver.config).map_err(|error| {
                AppError::IoError(format!("Falha ao serializar config do driver: {error}"))
            })?,
        },
        controllers: build_runtime_controller_payloads(active_controllers)?,
        plant: DriverBootstrapPlant {
            id: plant.id.clone(),
            name: plant.name.clone(),
            variables,
            variables_by_id,
            sensors: DriverBootstrapIoGroup {
                ids: sensor_ids,
                count: sensor_variables.len(),
                variables: sensor_variables,
                variables_by_id: sensor_variables_by_id,
            },
            actuators: DriverBootstrapIoGroup {
                ids: actuator_ids,
                count: actuator_variables.len(),
                variables: actuator_variables,
                variables_by_id: actuator_variables_by_id,
            },
            setpoints,
        },
        runtime: DriverBootstrapRuntime {
            id: runtime_id.to_string(),
            timing: DriverBootstrapRuntimeTiming {
                owner: "runtime",
                clock: "monotonic",
                strategy: "deadline",
                sample_time_ms: plant.sample_time_ms,
            },
            supervision: DriverBootstrapRuntimeSupervision {
                owner: "rust",
                startup_timeout_ms,
                shutdown_timeout_ms,
            },
            paths: DriverBootstrapRuntimePaths {
                runtime_dir: runtime_dir.display().to_string(),
                venv_python_path: venv_python_path.display().to_string(),
                runner_path: runner_path.display().to_string(),
                bootstrap_path: bootstrap_path.display().to_string(),
            },
        },
    })
}

pub(super) fn build_runtime_controller_payloads(
    active_controllers: &[ResolvedRuntimeController],
) -> AppResult<Vec<DriverBootstrapController>> {
    active_controllers
        .iter()
        .map(|controller| {
            Ok(DriverBootstrapController {
                id: controller.instance.id.clone(),
                plugin_id: controller.plugin.id.clone(),
                plugin_name: controller.plugin.name.clone(),
                plugin_dir: controller.plugin_dir.display().to_string(),
                source_file: controller
                    .plugin
                    .source_file
                    .clone()
                    .unwrap_or_else(|| "main.py".to_string()),
                class_name: controller.plugin.entry_class.clone(),
                name: controller.instance.name.clone(),
                controller_type: controller.instance.controller_type.clone(),
                active: controller.instance.active,
                input_variable_ids: controller.instance.input_variable_ids.clone(),
                output_variable_ids: controller.instance.output_variable_ids.clone(),
                params: serde_json::to_value(&controller.instance.params).map_err(|error| {
                    AppError::IoError(format!(
                        "Falha ao serializar parâmetros do controlador '{}': {error}",
                        controller.instance.name
                    ))
                })?,
            })
        })
        .collect()
}

pub(super) fn collect_runtime_setpoints(plant: &Plant) -> HashMap<String, f64> {
    plant
        .variables
        .iter()
        .map(|variable| (variable.id.clone(), variable.setpoint))
        .collect()
}
