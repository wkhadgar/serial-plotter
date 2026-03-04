use serde::{Deserialize, Serialize};

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
    pub variables: Vec<CreatePlantVariableRequest>,

    #[serde(default)]
    pub driver_id: Option<String>,

    #[serde(default)]
    pub controller_ids: Option<Vec<String>>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantStats {
    pub dt: f64,
    pub uptime: u64,
}

impl PlantStats {
    pub fn default() -> Self {
        Self {
            dt: 0.0,
            uptime: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plant {
    pub id: String,
    pub name: String,
    pub variables: Vec<PlantVariable>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub controller_ids: Option<Vec<String>>,

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
    pub connected: bool,
    pub paused: bool,
    pub variables: Vec<PlantVariable>,
    pub stats: PlantStats,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub controller_ids: Option<Vec<String>>,
}

impl From<&Plant> for PlantResponse {
    fn from(plant: &Plant) -> Self {
        Self {
            id: plant.id.clone(),
            name: plant.name.clone(),
            connected: plant.connected,
            paused: plant.paused,
            variables: plant.variables.clone(),
            stats: plant.stats.clone(),
            driver_id: plant.driver_id.clone(),
            controller_ids: plant.controller_ids.clone(),
        }
    }
}
