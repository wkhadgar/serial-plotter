use uuid::Uuid;
use crate::core::error::{AppError, AppResult};
use crate::core::models::plant::{
    CreatePlantRequest, CreatePlantVariableRequest, Plant, PlantVariable, PlantResponse, PlantStats,
};

pub struct PlantService;

impl PlantService {
    pub fn create_plant(request: CreatePlantRequest) -> AppResult<Plant> {
        if request.name.trim().is_empty() {
            return Err(AppError::InvalidArgument("Nome da planta é obrigatório".into()));
        }

        if request.variables.is_empty() {
            return Err(AppError::InvalidArgument("Pelo menos uma variável deve ser definida".into()));
        }

        for (idx, var) in request.variables.iter().enumerate() {
            Self::validate_variable(var).map_err(|e| {
                AppError::InvalidArgument(
                    format!("Variável {} inválida: {}", idx, e)
                )
            })?;
        }

        let plant_id = format!("plant_{}", Uuid::new_v4());

        let variables = request
            .variables
            .into_iter()
            .enumerate()
            .map(|(idx, var)| {
                PlantVariable {
                    id: format!("var_{}", idx),
                    name: var.name,
                    var_type: var.var_type,
                    unit: var.unit,
                    setpoint: var.setpoint,
                    pv_min: var.pv_min,
                    pv_max: var.pv_max,
                    linked_sensor_ids: var.linked_sensor_ids,
                }
            })
            .collect();

        let plant = Plant {
            id: plant_id,
            name: request.name,
            variables,
            driver_id: request.driver_id,
            controller_ids: request.controller_ids,
            connected: false,
            paused: false,
            stats: PlantStats::default(),
        };

        Ok(plant)
    }

    fn validate_variable(var: &CreatePlantVariableRequest) -> AppResult<()> {
        if var.name.trim().is_empty() {
            return Err(AppError::InvalidArgument("Nome da variável é obrigatório".into()));
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::plant::VariableType;

    fn create_test_variable(name: &str) -> CreatePlantVariableRequest {
        CreatePlantVariableRequest {
            name: name.to_string(),
            var_type: VariableType::Sensor,
            unit: "C".to_string(),
            setpoint: 50.0,
            pv_min: 0.0,
            pv_max: 100.0,
            linked_sensor_ids: None,
        }
    }

    #[test]
    fn test_create_plant_success() {
        let request = CreatePlantRequest {
            name: "Planta 1".to_string(),
            variables: vec![create_test_variable("Temperatura")],
            driver_id: None,
            controller_ids: None,
        };

        let result = PlantService::create_plant(request);
        assert!(result.is_ok());
        let plant = result.unwrap();
        assert_eq!(plant.name, "Planta 1");
        assert_eq!(plant.variables.len(), 1);
        assert_eq!(plant.variables[0].id, "var_0");
        assert_eq!(plant.variables[0].name, "Temperatura");
        assert_eq!(plant.connected, false);
        assert_eq!(plant.paused, false);
        assert_eq!(plant.stats.uptime, 0);
        assert_eq!(plant.stats.dt, 0.0);
    }

    #[test]
    fn test_create_plant_empty_name() {
        let request = CreatePlantRequest {
            name: "".to_string(),
            variables: vec![create_test_variable("Temperatura")],
            driver_id: None,
            controller_ids: None,
        };

        let result = PlantService::create_plant(request);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_plant_no_variables() {
        let request = CreatePlantRequest {
            name: "Planta 1".to_string(),
            variables: vec![],
            driver_id: None,
            controller_ids: None,
        };

        let result = PlantService::create_plant(request);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_plant_invalid_pv_range() {
        let mut var = create_test_variable("Temp");
        var.pv_min = 100.0;
        var.pv_max = 0.0;

        let request = CreatePlantRequest {
            name: "Planta 1".to_string(),
            variables: vec![var],
            driver_id: None,
            controller_ids: None,
        };

        let result = PlantService::create_plant(request);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_plant_invalid_setpoint() {
        let mut var = create_test_variable("Temp");
        var.setpoint = 150.0;

        let request = CreatePlantRequest {
            name: "Planta 1".to_string(),
            variables: vec![var],
            driver_id: None,
            controller_ids: None,
        };

        let result = PlantService::create_plant(request);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_plant_multiple_variables() {
        let var1 = create_test_variable("Temperatura");
        let mut var2 = create_test_variable("Umidade");
        var2.unit = "%".to_string();

        let request = CreatePlantRequest {
            name: "Planta Complexa".to_string(),
            variables: vec![var1, var2],
            driver_id: Some("driver_1".to_string()),
            controller_ids: Some(vec!["ctrl_1".to_string()]),
        };

        let result = PlantService::create_plant(request);
        assert!(result.is_ok());
        let plant = result.unwrap();
        assert_eq!(plant.variables.len(), 2);
        assert_eq!(plant.variables[0].id, "var_0");
        assert_eq!(plant.variables[1].id, "var_1");
        assert_eq!(plant.driver_id, Some("driver_1".to_string()));
    }
}
