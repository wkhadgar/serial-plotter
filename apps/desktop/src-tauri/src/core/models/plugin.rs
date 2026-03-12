use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginType {
    Driver,
    Controller,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginRuntime {
    Python,
    RustNative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemaFieldType {
    Bool,
    Int,
    Float,
    String,
    List,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SchemaFieldValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<SchemaFieldValue>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSchemaField {
    pub name: String,

    #[serde(rename = "type")]
    pub field_type: SchemaFieldType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<SchemaFieldValue>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRegistry {
    pub id: String,
    pub name: String,

    #[serde(rename = "type")]
    pub plugin_type: PluginType,

    pub runtime: PluginRuntime,

    #[serde(default)]
    pub schema: Vec<PluginSchemaField>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_file: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_code: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<PluginDependency>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreatePluginRequest {
    pub name: String,

    #[serde(rename = "type")]
    pub plugin_type: PluginType,

    pub runtime: PluginRuntime,

    #[serde(default)]
    pub schema: Vec<PluginSchemaField>,

    #[serde(default)]
    pub source_file: Option<String>,

    #[serde(default)]
    pub source_code: Option<String>,

    #[serde(default)]
    pub dependencies: Vec<PluginDependency>,

    #[serde(default)]
    pub description: Option<String>,

    #[serde(default)]
    pub version: Option<String>,

    #[serde(default)]
    pub author: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginInstanceStatus {
    Idle,
    Running,
    Stopped,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInstance {
    pub id: String,
    pub plugin_id: String,
    pub plugin_name: String,
    pub plugin_type: PluginType,
    pub status: PluginInstanceStatus,
    pub config: std::collections::HashMap<String, SchemaFieldValue>,
}
