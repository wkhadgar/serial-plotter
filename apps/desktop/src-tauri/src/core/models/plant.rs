use crate::core::models::plugin::{PluginRuntime, SchemaFieldValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn default_sample_time_ms() -> u64 {
    100
}

fn default_controller_active() -> bool {
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VariableType {
    Sensor,
    Atuador,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantVariable {
    pub id: String,
    pub name: String,

    #[serde(rename = "type")]
    pub var_type: VariableType,

    pub unit: String,
    pub setpoint: f64,
    pub pv_min: f64,
    pub pv_max: f64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_sensor_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreatePlantRequest {
    pub name: String,
    #[serde(default = "default_sample_time_ms")]
    pub sample_time_ms: u64,
    pub variables: Vec<CreatePlantVariableRequest>,
    pub driver: CreatePlantDriverRequest,
    #[serde(default)]
    pub controllers: Vec<CreatePlantControllerRequest>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdatePlantRequest {
    pub id: String,
    pub name: String,
    #[serde(default = "default_sample_time_ms")]
    pub sample_time_ms: u64,
    pub variables: Vec<CreatePlantVariableRequest>,
    pub driver: CreatePlantDriverRequest,
    #[serde(default)]
    pub controllers: Vec<CreatePlantControllerRequest>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreatePlantVariableRequest {
    pub name: String,

    #[serde(rename = "type")]
    pub var_type: VariableType,

    pub unit: String,
    pub setpoint: f64,
    pub pv_min: f64,
    pub pv_max: f64,

    #[serde(default)]
    pub linked_sensor_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlantStats {
    pub dt: f64,
    pub uptime: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreatePlantDriverRequest {
    pub plugin_id: String,
    #[serde(default)]
    pub config: HashMap<String, SchemaFieldValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ControllerParamType {
    Number,
    Boolean,
    String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerParam {
    #[serde(rename = "type")]
    pub param_type: ControllerParamType,
    pub value: SchemaFieldValue,
    pub label: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreatePlantControllerRequest {
    #[serde(default)]
    pub id: Option<String>,
    pub plugin_id: String,
    pub name: String,
    pub controller_type: String,
    #[serde(default = "default_controller_active")]
    pub active: bool,
    #[serde(default)]
    pub input_variable_ids: Vec<String>,
    #[serde(default)]
    pub output_variable_ids: Vec<String>,
    #[serde(default)]
    pub params: HashMap<String, ControllerParam>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantDriver {
    pub plugin_id: String,
    pub plugin_name: String,
    pub runtime: PluginRuntime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_code: Option<String>,
    #[serde(default)]
    pub config: HashMap<String, SchemaFieldValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantController {
    pub id: String,
    pub plugin_id: String,
    pub name: String,
    pub controller_type: String,
    pub active: bool,
    #[serde(default)]
    pub input_variable_ids: Vec<String>,
    #[serde(default)]
    pub output_variable_ids: Vec<String>,
    #[serde(default)]
    pub params: HashMap<String, ControllerParam>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plant {
    pub id: String,
    pub name: String,
    #[serde(default = "default_sample_time_ms")]
    pub sample_time_ms: u64,
    pub variables: Vec<PlantVariable>,
    pub driver: PlantDriver,
    #[serde(default)]
    pub controllers: Vec<PlantController>,

    #[serde(skip, default)]
    pub connected: bool,

    #[serde(skip, default)]
    pub paused: bool,

    #[serde(skip, default = "PlantStats::default")]
    pub stats: PlantStats,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlantResponse {
    pub id: String,
    pub name: String,
    pub sample_time_ms: u64,
    pub connected: bool,
    pub paused: bool,
    pub variables: Vec<PlantVariable>,
    pub stats: PlantStats,
    pub driver: PlantDriver,
    #[serde(default)]
    pub controllers: Vec<PlantController>,
}

impl From<&Plant> for PlantResponse {
    fn from(plant: &Plant) -> Self {
        Self {
            id: plant.id.clone(),
            name: plant.name.clone(),
            sample_time_ms: plant.sample_time_ms,
            connected: plant.connected,
            paused: plant.paused,
            variables: plant.variables.clone(),
            stats: plant.stats.clone(),
            driver: plant.driver.clone(),
            controllers: plant.controllers.clone(),
        }
    }
}
