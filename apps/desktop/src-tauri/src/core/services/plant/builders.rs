use super::controller_params::fill_missing_controller_params;
use super::validation::resolve_plugin;
use crate::core::error::AppResult;
use crate::core::models::plant::{
    ControllerRuntimeStatus, CreatePlantControllerRequest, CreatePlantDriverRequest,
    CreatePlantRequest, CreatePlantVariableRequest, Plant, PlantController, PlantDriver,
    PlantStats, PlantVariable,
};
use crate::core::models::plugin::{PluginRegistry, PluginType};
use crate::state::PluginStore;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub(super) struct PlantRuntimeSnapshot {
    pub previous_name: String,
    pub previous_sample_time_ms: u64,
    pub connected: bool,
    pub paused: bool,
    pub stats: PlantStats,
}

pub(super) fn build_plant(request: CreatePlantRequest, plugins: &PluginStore) -> AppResult<Plant> {
    let plant_id = format!("plant_{}", Uuid::new_v4());
    let sample_time_ms = request.sample_time_ms;
    let driver_plugin = resolve_plugin(plugins, &request.driver.plugin_id, PluginType::Driver)?;
    let variables = build_variables(request.variables);
    let controllers = request
        .controllers
        .into_iter()
        .map(|controller| build_controller(controller, plugins))
        .collect::<AppResult<Vec<_>>>()?;

    Ok(Plant {
        id: plant_id,
        name: request.name.trim().to_string(),
        sample_time_ms,
        variables,
        driver: build_driver(request.driver, &driver_plugin),
        controllers,
        connected: false,
        paused: false,
        stats: PlantStats {
            dt: sample_time_ms as f64 / 1000.0,
            uptime: 0,
        },
    })
}

pub(super) fn build_updated_plant(
    request: crate::core::models::plant::UpdatePlantRequest,
    plugins: &PluginStore,
    runtime: PlantRuntimeSnapshot,
) -> AppResult<Plant> {
    let driver_plugin = resolve_plugin(plugins, &request.driver.plugin_id, PluginType::Driver)?;
    let variables = build_variables(request.variables);
    let controllers = request
        .controllers
        .into_iter()
        .map(|controller| build_controller(controller, plugins))
        .collect::<AppResult<Vec<_>>>()?;

    let mut stats = runtime.stats;
    if !runtime.connected || stats.dt == runtime.previous_sample_time_ms as f64 / 1000.0 {
        stats.dt = request.sample_time_ms as f64 / 1000.0;
    }

    Ok(Plant {
        id: request.id,
        name: request.name.trim().to_string(),
        sample_time_ms: request.sample_time_ms,
        variables,
        driver: build_driver(request.driver, &driver_plugin),
        controllers,
        connected: runtime.connected,
        paused: runtime.paused,
        stats,
    })
}

fn build_variables(variables: Vec<CreatePlantVariableRequest>) -> Vec<PlantVariable> {
    variables
        .into_iter()
        .enumerate()
        .map(|(idx, var)| PlantVariable {
            id: format!("var_{}", idx),
            name: var.name,
            var_type: var.var_type,
            unit: var.unit,
            setpoint: var.setpoint,
            pv_min: var.pv_min,
            pv_max: var.pv_max,
            linked_sensor_ids: var.linked_sensor_ids,
        })
        .collect()
}

fn build_driver(request: CreatePlantDriverRequest, plugin: &PluginRegistry) -> PlantDriver {
    PlantDriver {
        plugin_id: plugin.id.clone(),
        plugin_name: plugin.name.clone(),
        runtime: plugin.runtime,
        source_file: plugin.source_file.clone(),
        source_code: None,
        config: request.config,
    }
}

fn build_controller(
    request: CreatePlantControllerRequest,
    plugins: &PluginStore,
) -> AppResult<PlantController> {
    let plugin = resolve_plugin(plugins, &request.plugin_id, PluginType::Controller)?;
    let mut params = request.params;
    fill_missing_controller_params(&mut params, &plugin);

    Ok(PlantController {
        id: request
            .id
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| format!("ctrl_{}", Uuid::new_v4().simple())),
        plugin_id: plugin.id,
        plugin_name: plugin.name,
        name: request.name.trim().to_string(),
        controller_type: request.controller_type.trim().to_string(),
        active: request.active,
        input_variable_ids: request.input_variable_ids,
        output_variable_ids: request.output_variable_ids,
        params,
        runtime_status: ControllerRuntimeStatus::Synced,
    })
}
