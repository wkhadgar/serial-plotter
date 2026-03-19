mod builders;
mod controller_params;
mod validation;

use self::builders::{build_plant, build_updated_plant, PlantRuntimeSnapshot};
use self::validation::{
    resolve_plugin, validate_active_controller_conflicts, validate_controller, validate_payload,
};
use crate::core::error::{AppError, AppResult};
use crate::core::models::plant::{
    ControllerParam, CreatePlantControllerRequest, CreatePlantRequest, CreatePlantVariableRequest,
    Plant, PlantController, RemovePlantControllerRequest, SavePlantControllerConfigRequest,
    SavePlantSetpointRequest, UpdatePlantRequest, VariableType,
};
use crate::core::models::plugin::{PluginRegistry, PluginType};
use crate::core::services::workspace::WorkspaceService;
use crate::state::{PlantStore, PluginStore};

pub struct PlantService;

impl PlantService {
    pub fn create(
        store: &PlantStore,
        plugins: &PluginStore,
        request: CreatePlantRequest,
    ) -> AppResult<Plant> {
        validate_payload(
            None,
            &request.name,
            request.sample_time_ms,
            &request.variables,
            &request.driver,
            &request.controllers,
            store,
            plugins,
        )?;

        let plant = build_plant(request, plugins)?;
        WorkspaceService::save_plant_registry(&plant)?;

        if let Err(error) = store.insert(plant.clone()) {
            let _ = WorkspaceService::delete_plant_registry(&plant.name);
            return Err(error);
        }

        Ok(plant)
    }

    pub fn update(
        store: &PlantStore,
        plugins: &PluginStore,
        request: UpdatePlantRequest,
    ) -> AppResult<Plant> {
        validate_payload(
            Some(request.id.as_str()),
            &request.name,
            request.sample_time_ms,
            &request.variables,
            &request.driver,
            &request.controllers,
            store,
            plugins,
        )?;

        let existing = store.get(&request.id)?;
        let runtime = PlantRuntimeSnapshot {
            previous_name: existing.name.clone(),
            previous_sample_time_ms: existing.sample_time_ms,
            connected: existing.connected,
            paused: existing.paused,
            stats: existing.stats,
        };

        let previous_name = runtime.previous_name.clone();
        let updated_plant = build_updated_plant(request, plugins, runtime)?;
        WorkspaceService::update_plant_registry(&updated_plant, &previous_name)?;
        store.replace(&updated_plant.id, updated_plant.clone())?;

        Ok(updated_plant)
    }

    pub(crate) fn fill_missing_controller_params(
        params: &mut std::collections::HashMap<String, ControllerParam>,
        plugin: &PluginRegistry,
    ) -> bool {
        controller_params::fill_missing_controller_params(params, plugin)
    }

    pub fn get(store: &PlantStore, id: &str) -> AppResult<Plant> {
        store.get(id)
    }

    pub fn list(store: &PlantStore) -> Vec<Plant> {
        store.list()
    }

    pub fn remove(store: &PlantStore, id: &str) -> AppResult<Plant> {
        let plant = store.get(id)?;
        WorkspaceService::delete_plant_registry(&plant.name)?;
        store.remove(id)
    }

    pub fn save_controller_config(
        store: &PlantStore,
        plugins: &PluginStore,
        request: SavePlantControllerConfigRequest,
    ) -> AppResult<Plant> {
        let plant = store.get(&request.plant_id)?;
        let controller_index = plant
            .controllers
            .iter()
            .position(|controller| controller.id == request.controller_id);
        let existing_controller = controller_index.map(|index| plant.controllers[index].clone());

        if plant.connected && existing_controller.is_none() {
            return Err(AppError::InvalidArgument(
                "Não é permitido adicionar controladores com a planta ligada".into(),
            ));
        }

        let plugin_id = request
            .plugin_id
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .or_else(|| {
                existing_controller
                    .as_ref()
                    .map(|controller| controller.plugin_id.as_str())
            })
            .ok_or_else(|| {
                AppError::InvalidArgument("Plugin do controlador é obrigatório".into())
            })?;
        let plugin = resolve_plugin(plugins, plugin_id, PluginType::Controller)?;

        if plant.connected
            && existing_controller
                .as_ref()
                .is_some_and(|controller| controller.plugin_id != plugin.id)
        {
            return Err(AppError::InvalidArgument(
                "Não é permitido trocar o plugin do controlador com a planta ligada".into(),
            ));
        }

        let current_variables = map_current_variables(&plant.variables);
        let mut controller_request = CreatePlantControllerRequest {
            id: Some(request.controller_id.clone()),
            plugin_id: plugin.id.clone(),
            name: request.name.clone(),
            controller_type: request.controller_type.clone(),
            active: request.active,
            input_variable_ids: request.input_variable_ids.clone(),
            output_variable_ids: request.output_variable_ids.clone(),
            params: request
                .params
                .into_iter()
                .map(|param| {
                    (
                        param.key,
                        ControllerParam {
                            param_type: param.param_type,
                            value: param.value,
                            label: param.label,
                        },
                    )
                })
                .collect(),
        };
        Self::fill_missing_controller_params(&mut controller_request.params, &plugin);

        validate_controller(&controller_request, &current_variables, plugins)?;

        let mut conflict_requests = plant
            .controllers
            .iter()
            .enumerate()
            .map(|(index, controller)| {
                if controller_index == Some(index) {
                    controller_request.clone()
                } else {
                    CreatePlantControllerRequest {
                        id: Some(controller.id.clone()),
                        plugin_id: controller.plugin_id.clone(),
                        name: controller.name.clone(),
                        controller_type: controller.controller_type.clone(),
                        active: controller.active,
                        input_variable_ids: controller.input_variable_ids.clone(),
                        output_variable_ids: controller.output_variable_ids.clone(),
                        params: controller.params.clone(),
                    }
                }
            })
            .collect::<Vec<_>>();

        if controller_index.is_none() {
            conflict_requests.push(controller_request.clone());
        }

        validate_active_controller_conflicts(&conflict_requests)?;

        let updated = store.update(&request.plant_id, |plant| {
            if let Some(controller) = plant
                .controllers
                .iter_mut()
                .find(|controller| controller.id == request.controller_id)
            {
                controller.plugin_id = plugin.id.clone();
                controller.plugin_name = plugin.name.clone();
                controller.name = request.name.trim().to_string();
                controller.active = request.active;
                controller.input_variable_ids = request.input_variable_ids.clone();
                controller.output_variable_ids = request.output_variable_ids.clone();
                controller.controller_type = request.controller_type.trim().to_string();
                controller.params = controller_request.params.clone();
                return;
            }

            plant.controllers.push(PlantController {
                id: request.controller_id.clone(),
                plugin_id: plugin.id.clone(),
                plugin_name: plugin.name.clone(),
                name: request.name.trim().to_string(),
                controller_type: request.controller_type.trim().to_string(),
                active: false,
                input_variable_ids: request.input_variable_ids.clone(),
                output_variable_ids: request.output_variable_ids.clone(),
                params: controller_request.params.clone(),
            });
        })?;

        WorkspaceService::update_plant_registry(&updated, &updated.name)?;
        Ok(updated)
    }

    pub fn remove_controller(
        store: &PlantStore,
        request: RemovePlantControllerRequest,
    ) -> AppResult<Plant> {
        let plant = store.get(&request.plant_id)?;
        if !plant
            .controllers
            .iter()
            .any(|controller| controller.id == request.controller_id)
        {
            return Err(AppError::NotFound(format!(
                "Controlador '{}' não encontrado na planta '{}'",
                request.controller_id, plant.name
            )));
        }

        let updated = store.update(&request.plant_id, |plant| {
            plant
                .controllers
                .retain(|controller| controller.id != request.controller_id);
        })?;

        WorkspaceService::update_plant_registry(&updated, &updated.name)?;
        Ok(updated)
    }

    pub fn save_setpoint(
        store: &PlantStore,
        request: SavePlantSetpointRequest,
    ) -> AppResult<Plant> {
        let plant = store.get(&request.plant_id)?;
        let variable = plant
            .variables
            .iter()
            .find(|variable| variable.id == request.variable_id)
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "Variável '{}' não encontrada na planta '{}'",
                    request.variable_id, plant.name
                ))
            })?;

        if variable.var_type != VariableType::Sensor {
            return Err(AppError::InvalidArgument(
                "Apenas sensores podem ter setpoint editado".into(),
            ));
        }

        if request.setpoint < variable.pv_min || request.setpoint > variable.pv_max {
            return Err(AppError::InvalidArgument(
                "Setpoint deve estar entre pv_min e pv_max".into(),
            ));
        }

        let updated = store.update(&request.plant_id, |plant| {
            if let Some(variable) = plant
                .variables
                .iter_mut()
                .find(|variable| variable.id == request.variable_id)
            {
                variable.setpoint = request.setpoint;
            }
        })?;

        WorkspaceService::update_plant_registry(&updated, &updated.name)?;
        Ok(updated)
    }
}

fn map_current_variables(
    variables: &[crate::core::models::plant::PlantVariable],
) -> Vec<CreatePlantVariableRequest> {
    variables
        .iter()
        .map(|variable| CreatePlantVariableRequest {
            name: variable.name.clone(),
            var_type: variable.var_type,
            unit: variable.unit.clone(),
            setpoint: variable.setpoint,
            pv_min: variable.pv_min,
            pv_max: variable.pv_max,
            linked_sensor_ids: variable.linked_sensor_ids.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests;
