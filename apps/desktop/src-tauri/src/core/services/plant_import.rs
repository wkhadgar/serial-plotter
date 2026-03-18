use crate::core::error::{AppError, AppResult};
use crate::core::models::plant::{
    CreatePlantDriverRequest, CreatePlantRequest, CreatePlantVariableRequest, PlantResponse,
    PlantStats, PlantVariable, VariableType,
};
use crate::core::models::plugin::{PluginType, SchemaFieldValue};
use crate::core::services::plant::PlantService;
use crate::core::services::plugin::PluginService;
use crate::state::{PlantStore, PluginStore};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PlantImportFileRequest {
    pub file_name: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportedVariableStatsResponse {
    pub error_avg: f64,
    pub stability: f64,
    pub ripple: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportedSeriesDescriptorResponse {
    pub key: String,
    pub label: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportedSeriesCatalogResponse {
    pub plant_id: String,
    pub series: Vec<ImportedSeriesDescriptorResponse>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportedWorkspaceDriverResponse {
    pub plugin_id: String,
    pub plugin_name: String,
    pub config: HashMap<String, SchemaFieldValue>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportedWorkspacePlantResponse {
    pub id: String,
    pub name: String,
    pub sample_time_ms: u64,
    pub connected: bool,
    pub paused: bool,
    pub variables: Vec<PlantVariable>,
    pub stats: PlantStats,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver: Option<ImportedWorkspaceDriverResponse>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenPlantFileResponse {
    pub plant: ImportedWorkspacePlantResponse,
    pub data: Vec<HashMap<String, f64>>,
    pub stats: PlantStats,
    pub variable_stats: Vec<ImportedVariableStatsResponse>,
    pub series_catalog: ImportedSeriesCatalogResponse,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportPlantFileResponse {
    pub plant: PlantResponse,
    pub data: Vec<HashMap<String, f64>>,
    pub stats: PlantStats,
    pub variable_stats: Vec<ImportedVariableStatsResponse>,
    pub series_catalog: ImportedSeriesCatalogResponse,
}

pub struct PlantImportService;

impl PlantImportService {
    pub fn open_file(request: PlantImportFileRequest) -> AppResult<OpenPlantFileResponse> {
        let parsed: Value = serde_json::from_str(&request.content)
            .map_err(|error| invalid_argument(format!("JSON inválido: {error}")))?;

        let root = expect_object(&parsed, "Arquivo")?;
        if root.get("variables").is_some() {
            return open_registry_plant_file(root, &request);
        }

        let meta = resolve_meta(root)?;
        let sensors = expect_array(
            root.get("sensors")
                .ok_or_else(|| invalid_argument("Campo \"sensors\" ausente"))?,
            "sensors",
        )?;
        let actuators = expect_array(
            root.get("actuators")
                .ok_or_else(|| invalid_argument("Campo \"actuators\" ausente"))?,
            "actuators",
        )?;
        let setpoints = expect_array(
            root.get("setpoints")
                .ok_or_else(|| invalid_argument("Campo \"setpoints\" ausente"))?,
            "setpoints",
        )?;
        let data = expect_array(
            root.get("data")
                .ok_or_else(|| invalid_argument("Campo \"data\" ausente"))?,
            "data",
        )?;

        if data.is_empty() {
            return Err(invalid_argument("Campo \"data\" está vazio"));
        }

        let mut variables = Vec::new();
        let mut sensor_index_by_export_id = HashMap::new();

        for (index, sensor) in sensors.iter().enumerate() {
            let sensor_obj = expect_object(sensor, &format!("sensors[{index}]"))?;
            let sensor_id = expect_string(sensor_obj.get("id"), &format!("sensors[{index}].id"))?;
            let name = expect_string(sensor_obj.get("name"), &format!("sensors[{index}].name"))?;
            let unit = sensor_obj
                .get("unit")
                .and_then(Value::as_str)
                .unwrap_or("%")
                .to_string();

            sensor_index_by_export_id.insert(sensor_id, index);
            variables.push(PlantVariable {
                id: format!("var_{index}"),
                name,
                var_type: VariableType::Sensor,
                unit,
                setpoint: 0.0,
                pv_min: 0.0,
                pv_max: 100.0,
                linked_sensor_ids: None,
            });
        }

        let actuators_offset = variables.len();
        let mut actuator_index_by_export_id = HashMap::new();

        for (index, actuator) in actuators.iter().enumerate() {
            let actuator_obj = expect_object(actuator, &format!("actuators[{index}]"))?;
            let actuator_id =
                expect_string(actuator_obj.get("id"), &format!("actuators[{index}].id"))?;
            let name = expect_string(
                actuator_obj.get("name"),
                &format!("actuators[{index}].name"),
            )?;
            let unit = actuator_obj
                .get("unit")
                .and_then(Value::as_str)
                .unwrap_or("%")
                .to_string();
            let linked_sensor_ids = actuator_obj
                .get("linkedSensorIds")
                .and_then(Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(Value::as_str)
                        .map(|sensor_id| {
                            sensor_index_by_export_id
                                .get(sensor_id)
                                .map(|sensor_index| format!("var_{sensor_index}"))
                                .unwrap_or_else(|| sensor_id.to_string())
                        })
                        .collect::<Vec<_>>()
                });

            let variable_index = actuators_offset + index;
            actuator_index_by_export_id.insert(actuator_id, variable_index);
            variables.push(PlantVariable {
                id: format!("var_{variable_index}"),
                name,
                var_type: VariableType::Atuador,
                unit,
                setpoint: 0.0,
                pv_min: 0.0,
                pv_max: 100.0,
                linked_sensor_ids,
            });
        }

        let mut setpoint_sensor_map = HashMap::new();
        for (index, setpoint) in setpoints.iter().enumerate() {
            let setpoint_obj = expect_object(setpoint, &format!("setpoints[{index}]"))?;
            let setpoint_id =
                expect_string(setpoint_obj.get("id"), &format!("setpoints[{index}].id"))?;
            let sensor_id = expect_string(
                setpoint_obj.get("sensorId"),
                &format!("setpoints[{index}].sensorId"),
            )?;
            setpoint_sensor_map.insert(setpoint_id, sensor_id);
        }

        let mut points = Vec::with_capacity(data.len());
        for (sample_index, sample) in data.iter().enumerate() {
            let sample_obj = expect_object(sample, &format!("data[{sample_index}]"))?;
            let mut point = HashMap::new();
            point.insert(
                "time".into(),
                expect_number(
                    sample_obj.get("time"),
                    &format!("data[{sample_index}].time"),
                )?,
            );

            let sensors_record = expect_object(
                sample_obj.get("sensors").ok_or_else(|| {
                    invalid_argument(format!("data[{sample_index}].sensors ausente"))
                })?,
                &format!("data[{sample_index}].sensors"),
            )?;
            for (sensor_id, value) in sensors_record {
                if let Some(variable_index) = sensor_index_by_export_id.get(sensor_id) {
                    point.insert(
                        format!("var_{variable_index}_pv"),
                        value.as_f64().unwrap_or(0.0),
                    );
                }
            }

            let setpoints_record = expect_object(
                sample_obj.get("setpoints").ok_or_else(|| {
                    invalid_argument(format!("data[{sample_index}].setpoints ausente"))
                })?,
                &format!("data[{sample_index}].setpoints"),
            )?;
            for (setpoint_id, value) in setpoints_record {
                let Some(sensor_id) = setpoint_sensor_map.get(setpoint_id) else {
                    continue;
                };
                let Some(variable_index) = sensor_index_by_export_id.get(sensor_id) else {
                    continue;
                };
                point.insert(
                    format!("var_{variable_index}_sp"),
                    value.as_f64().unwrap_or(0.0),
                );
            }

            let actuators_record = expect_object(
                sample_obj.get("actuators").ok_or_else(|| {
                    invalid_argument(format!("data[{sample_index}].actuators ausente"))
                })?,
                &format!("data[{sample_index}].actuators"),
            )?;
            for (actuator_id, value) in actuators_record {
                if let Some(variable_index) = actuator_index_by_export_id.get(actuator_id) {
                    point.insert(
                        format!("var_{variable_index}_pv"),
                        value.as_f64().unwrap_or(0.0),
                    );
                }
            }

            points.push(point);
        }

        let stats = compute_imported_plant_stats(&points);
        let name = meta
            .get("name")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(request.file_name.as_str())
            .to_string();
        let sample_time_ms = meta
            .get("sampleTimeMs")
            .and_then(Value::as_u64)
            .unwrap_or_else(|| {
                if stats.dt > 0.0 {
                    (stats.dt * 1000.0).round() as u64
                } else {
                    100
                }
            });
        let plant_id = format!("imported_{}", uuid::Uuid::new_v4().simple());
        let variable_stats = variables
            .iter()
            .enumerate()
            .map(|(index, variable)| compute_imported_variable_stats(&points, index, variable))
            .collect::<Vec<_>>();
        let series_catalog = build_imported_series_catalog(&plant_id, &variables);

        Ok(OpenPlantFileResponse {
            plant: ImportedWorkspacePlantResponse {
                id: plant_id,
                name,
                sample_time_ms,
                connected: false,
                paused: false,
                variables,
                stats: stats.clone(),
                driver: None,
            },
            data: points,
            stats,
            variable_stats,
            series_catalog,
        })
    }

    pub fn import_file(
        plants: &PlantStore,
        plugins: &PluginStore,
        request: PlantImportFileRequest,
    ) -> AppResult<ImportPlantFileResponse> {
        let OpenPlantFileResponse {
            plant: imported_plant,
            data,
            stats,
            variable_stats,
            series_catalog,
        } = Self::open_file(request)?;

        PluginService::load_all(plugins)?;

        let driver = resolve_imported_driver_request(plugins, imported_plant.driver.as_ref())?;
        let variables = imported_plant
            .variables
            .iter()
            .map(map_imported_variable_to_create_request)
            .collect::<Vec<_>>();

        let created = PlantService::create(
            plants,
            plugins,
            CreatePlantRequest {
                name: imported_plant.name,
                sample_time_ms: imported_plant.sample_time_ms,
                variables,
                driver,
                controllers: vec![],
            },
        )?;

        Ok(ImportPlantFileResponse {
            plant: created.into(),
            data,
            stats,
            variable_stats,
            series_catalog,
        })
    }
}

fn invalid_argument(message: impl Into<String>) -> AppError {
    AppError::InvalidArgument(message.into())
}

fn expect_object<'a>(
    value: &'a Value,
    context: &str,
) -> AppResult<&'a serde_json::Map<String, Value>> {
    value
        .as_object()
        .ok_or_else(|| invalid_argument(format!("{context} deve ser um objeto")))
}

fn expect_array<'a>(value: &'a Value, context: &str) -> AppResult<&'a Vec<Value>> {
    value
        .as_array()
        .ok_or_else(|| invalid_argument(format!("{context} deve ser um array")))
}

fn resolve_meta<'a>(
    root: &'a serde_json::Map<String, Value>,
) -> AppResult<&'a serde_json::Map<String, Value>> {
    match root.get("meta") {
        Some(value) => expect_object(value, "meta"),
        None => Ok(root),
    }
}

fn expect_string(value: Option<&Value>, context: &str) -> AppResult<String> {
    value
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| invalid_argument(format!("{context} deve ser uma string")))
}

fn expect_number(value: Option<&Value>, context: &str) -> AppResult<f64> {
    value
        .and_then(Value::as_f64)
        .ok_or_else(|| invalid_argument(format!("{context} deve ser um número")))
}

fn get_value_by_keys<'a>(
    object: &'a serde_json::Map<String, Value>,
    keys: &[&str],
) -> Option<&'a Value> {
    keys.iter().find_map(|key| object.get(*key))
}

fn parse_variable_type(value: &str) -> AppResult<VariableType> {
    match value.trim().to_lowercase().as_str() {
        "sensor" => Ok(VariableType::Sensor),
        "atuador" | "actuator" => Ok(VariableType::Atuador),
        _ => Err(invalid_argument(
            "variables.type deve ser \"sensor\" ou \"atuador\"",
        )),
    }
}

fn parse_registry_variable(value: &Value, index: usize) -> AppResult<PlantVariable> {
    let variable_obj = expect_object(value, &format!("variables[{index}]"))?;
    let id = variable_obj
        .get("id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| format!("var_{index}"));
    let name = expect_string(
        variable_obj.get("name"),
        &format!("variables[{index}].name"),
    )?;
    let type_label = variable_obj
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| invalid_argument(format!("variables[{index}].type deve ser string")))?;
    let var_type = parse_variable_type(type_label)?;
    let unit = variable_obj
        .get("unit")
        .and_then(Value::as_str)
        .unwrap_or("%")
        .to_string();
    let setpoint = variable_obj
        .get("setpoint")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let pv_min = get_value_by_keys(variable_obj, &["pv_min", "pvMin"])
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let pv_max = get_value_by_keys(variable_obj, &["pv_max", "pvMax"])
        .and_then(Value::as_f64)
        .unwrap_or(100.0);
    let linked_sensor_ids =
        get_value_by_keys(variable_obj, &["linked_sensor_ids", "linkedSensorIds"])
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            });

    Ok(PlantVariable {
        id,
        name,
        var_type,
        unit,
        setpoint,
        pv_min,
        pv_max,
        linked_sensor_ids,
    })
}

fn parse_registry_driver(
    root: &serde_json::Map<String, Value>,
) -> AppResult<Option<ImportedWorkspaceDriverResponse>> {
    let Some(driver_value) = root.get("driver") else {
        return Ok(None);
    };

    let driver_obj = expect_object(driver_value, "driver")?;
    let plugin_id = get_value_by_keys(driver_obj, &["plugin_id", "pluginId"])
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| invalid_argument("driver.plugin_id deve ser uma string não vazia"))?;

    let plugin_name = get_value_by_keys(driver_obj, &["plugin_name", "pluginName"])
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(plugin_id.as_str())
        .to_string();

    let config = match driver_obj.get("config") {
        None => HashMap::new(),
        Some(value) if value.is_null() => HashMap::new(),
        Some(value) => {
            serde_json::from_value::<HashMap<String, SchemaFieldValue>>(value.clone())
                .map_err(|error| invalid_argument(format!("driver.config inválido: {error}")))?
        }
    };

    Ok(Some(ImportedWorkspaceDriverResponse {
        plugin_id,
        plugin_name,
        config,
    }))
}

fn open_registry_plant_file(
    root: &serde_json::Map<String, Value>,
    request: &PlantImportFileRequest,
) -> AppResult<OpenPlantFileResponse> {
    let variables_payload = expect_array(
        root.get("variables")
            .ok_or_else(|| invalid_argument("Campo \"variables\" ausente"))?,
        "variables",
    )?;

    if variables_payload.is_empty() {
        return Err(invalid_argument("Campo \"variables\" está vazio"));
    }

    let variables = variables_payload
        .iter()
        .enumerate()
        .map(|(index, variable)| parse_registry_variable(variable, index))
        .collect::<Result<Vec<_>, _>>()?;

    let name = root
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(request.file_name.as_str())
        .to_string();
    let sample_time_ms = get_value_by_keys(root, &["sample_time_ms", "sampleTimeMs"])
        .and_then(Value::as_u64)
        .unwrap_or(100);
    let driver = parse_registry_driver(root)?;
    let plant_id = format!("imported_{}", uuid::Uuid::new_v4().simple());
    let stats = PlantStats {
        dt: sample_time_ms as f64 / 1000.0,
        uptime: 0,
    };
    let variable_stats = variables
        .iter()
        .enumerate()
        .map(|(index, variable)| compute_imported_variable_stats(&[], index, variable))
        .collect::<Vec<_>>();
    let series_catalog = build_imported_series_catalog(&plant_id, &variables);

    Ok(OpenPlantFileResponse {
        plant: ImportedWorkspacePlantResponse {
            id: plant_id,
            name,
            sample_time_ms,
            connected: false,
            paused: false,
            variables,
            stats: stats.clone(),
            driver,
        },
        data: vec![],
        stats,
        variable_stats,
        series_catalog,
    })
}

fn compute_imported_plant_stats(data: &[HashMap<String, f64>]) -> PlantStats {
    if data.len() <= 1 {
        return PlantStats {
            dt: 0.0,
            uptime: data
                .first()
                .and_then(|point| point.get("time"))
                .copied()
                .unwrap_or(0.0)
                .max(0.0)
                .round() as u64,
        };
    }

    let mut deltas = Vec::with_capacity(data.len().saturating_sub(1));
    for index in 1..data.len() {
        let prev = data[index - 1].get("time").copied().unwrap_or(0.0);
        let current = data[index].get("time").copied().unwrap_or(0.0);
        deltas.push((current - prev).max(0.0));
    }

    let avg_delta = deltas.iter().sum::<f64>() / deltas.len() as f64;
    let uptime = data
        .last()
        .and_then(|point| point.get("time"))
        .copied()
        .unwrap_or(0.0)
        .max(0.0);

    PlantStats {
        dt: (avg_delta * 10_000.0).round() / 10_000.0,
        uptime: uptime.round() as u64,
    }
}

fn compute_imported_variable_stats(
    data: &[HashMap<String, f64>],
    variable_index: usize,
    variable: &PlantVariable,
) -> ImportedVariableStatsResponse {
    let pv_key = format!("var_{variable_index}_pv");
    let sp_key = format!("var_{variable_index}_sp");
    let values: Vec<f64> = data
        .iter()
        .map(|point| point.get(&pv_key).copied().unwrap_or(0.0))
        .collect();

    if values.is_empty() {
        return ImportedVariableStatsResponse {
            error_avg: 0.0,
            stability: 100.0,
            ripple: 0.0,
        };
    }

    let min = values
        .iter()
        .fold(f64::INFINITY, |acc, value| acc.min(*value));
    let max = values
        .iter()
        .fold(f64::NEG_INFINITY, |acc, value| acc.max(*value));
    let ripple = ((max - min) * 1000.0).round() / 1000.0;

    if variable.var_type == VariableType::Atuador {
        return ImportedVariableStatsResponse {
            error_avg: 0.0,
            stability: (100.0 - ripple).max(0.0),
            ripple,
        };
    }

    let error_avg = data
        .iter()
        .map(|point| {
            let pv = point.get(&pv_key).copied().unwrap_or(0.0);
            let sp = point.get(&sp_key).copied().unwrap_or(0.0);
            (pv - sp).abs()
        })
        .sum::<f64>()
        / values.len() as f64;

    ImportedVariableStatsResponse {
        error_avg: (error_avg * 1000.0).round() / 1000.0,
        stability: ((100.0 - ripple) * 100.0).round() / 100.0,
        ripple,
    }
}

fn build_imported_series_catalog(
    plant_id: &str,
    variables: &[PlantVariable],
) -> ImportedSeriesCatalogResponse {
    let mut series = Vec::new();

    for (index, variable) in variables.iter().enumerate() {
        let pv_key = format!("var_{index}_pv");
        let sp_key = format!("var_{index}_sp");

        if variable.var_type == VariableType::Sensor {
            series.push(ImportedSeriesDescriptorResponse {
                key: pv_key,
                label: format!("{} PV", variable.name),
                role: "pv".into(),
            });
            series.push(ImportedSeriesDescriptorResponse {
                key: sp_key,
                label: format!("{} SP", variable.name),
                role: "sp".into(),
            });
            continue;
        }

        series.push(ImportedSeriesDescriptorResponse {
            key: pv_key,
            label: variable.name.clone(),
            role: "mv".into(),
        });
    }

    ImportedSeriesCatalogResponse {
        plant_id: plant_id.to_string(),
        series,
    }
}

fn map_imported_variable_to_create_request(variable: &PlantVariable) -> CreatePlantVariableRequest {
    CreatePlantVariableRequest {
        name: variable.name.clone(),
        var_type: variable.var_type,
        unit: variable.unit.clone(),
        setpoint: variable.setpoint,
        pv_min: variable.pv_min,
        pv_max: variable.pv_max,
        linked_sensor_ids: variable.linked_sensor_ids.clone(),
    }
}

fn resolve_imported_driver_request(
    plugins: &PluginStore,
    imported_driver: Option<&ImportedWorkspaceDriverResponse>,
) -> AppResult<CreatePlantDriverRequest> {
    let Some(driver) = imported_driver else {
        return Err(invalid_argument(
            "Arquivo da planta não contém driver configurado",
        ));
    };

    match plugins.get(&driver.plugin_id) {
        Ok(plugin) => {
            if plugin.plugin_type != PluginType::Driver {
                return Err(invalid_argument(format!(
                    "Plugin '{}' não é um driver válido",
                    driver.plugin_name
                )));
            }

            return Ok(CreatePlantDriverRequest {
                plugin_id: plugin.id,
                config: driver.config.clone(),
            });
        }
        Err(AppError::NotFound(_)) => {}
        Err(error) => return Err(error),
    }

    let resolved_by_name = plugins
        .list_by_type(PluginType::Driver)
        .into_iter()
        .find(|plugin| plugin.name.eq_ignore_ascii_case(&driver.plugin_name));

    let Some(plugin) = resolved_by_name else {
        return Err(invalid_argument(format!(
            "Driver '{}' não está carregado no sistema",
            driver.plugin_name
        )));
    };

    Ok(CreatePlantDriverRequest {
        plugin_id: plugin.id,
        config: driver.config.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::plugin::SchemaFieldValue;

    #[test]
    fn open_file_reads_registry_shape_with_driver() {
        let json = r#"
        {
          "name": "Planta Registry",
          "sample_time_ms": 500,
          "variables": [
            {
              "id": "sensor_a",
              "name": "Temperatura",
              "type": "sensor",
              "unit": "C",
              "setpoint": 45.0,
              "pv_min": 0.0,
              "pv_max": 100.0
            }
          ],
          "driver": {
            "plugin_id": "driver_mock",
            "plugin_name": "Driver Mock",
            "config": {
              "baud": 9600
            }
          }
        }
        "#;

        let response = PlantImportService::open_file(PlantImportFileRequest {
            file_name: "registry.json".to_string(),
            content: json.to_string(),
        })
        .expect("open file should succeed");

        assert_eq!(response.plant.name, "Planta Registry");
        assert_eq!(response.plant.sample_time_ms, 500);
        assert_eq!(response.plant.variables.len(), 1);
        assert_eq!(response.data.len(), 0);
        assert!(response.plant.driver.is_some());

        let driver = response.plant.driver.expect("driver should be present");
        assert_eq!(driver.plugin_id, "driver_mock");
        assert_eq!(driver.plugin_name, "Driver Mock");
        assert!(matches!(
            driver.config.get("baud"),
            Some(SchemaFieldValue::Int(9600))
        ));
    }

    #[test]
    fn open_file_reads_legacy_shape_without_driver() {
        let json = r#"
        {
          "meta": {
            "name": "Planta Legacy",
            "sampleTimeMs": 1000
          },
          "sensors": [
            { "id": "s1", "name": "Temp", "unit": "C" }
          ],
          "actuators": [
            {
              "id": "a1",
              "name": "Valvula",
              "unit": "%",
              "linkedSensorIds": ["s1"]
            }
          ],
          "setpoints": [
            { "id": "sp1", "sensorId": "s1" }
          ],
          "data": [
            {
              "time": 0.0,
              "sensors": { "s1": 20.0 },
              "setpoints": { "sp1": 25.0 },
              "actuators": { "a1": 10.0 }
            },
            {
              "time": 1.0,
              "sensors": { "s1": 21.0 },
              "setpoints": { "sp1": 26.0 },
              "actuators": { "a1": 11.0 }
            }
          ]
        }
        "#;

        let response = PlantImportService::open_file(PlantImportFileRequest {
            file_name: "legacy.json".to_string(),
            content: json.to_string(),
        })
        .expect("open file should succeed");

        assert_eq!(response.plant.name, "Planta Legacy");
        assert_eq!(response.plant.sample_time_ms, 1000);
        assert_eq!(response.plant.variables.len(), 2);
        assert_eq!(response.data.len(), 2);
        assert!(response.plant.driver.is_none());
        assert_eq!(response.stats.dt, 1.0);
    }
}
