use crate::core::models::plant::{ControllerParam, ControllerParamType};
use crate::core::models::plugin::{
    PluginRegistry, PluginSchemaField, SchemaFieldType, SchemaFieldValue,
};
use std::collections::HashMap;

pub(super) fn fill_missing_controller_params(
    params: &mut HashMap<String, ControllerParam>,
    plugin: &PluginRegistry,
) -> bool {
    let mut changed = false;

    for field in &plugin.schema {
        if params.contains_key(&field.name) {
            continue;
        }

        params.insert(field.name.clone(), controller_param_from_schema(field));
        changed = true;
    }

    changed
}

fn controller_param_from_schema(field: &PluginSchemaField) -> ControllerParam {
    let label = field
        .description
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(field.name.as_str())
        .to_string();

    match field.field_type {
        SchemaFieldType::Bool => ControllerParam {
            param_type: ControllerParamType::Boolean,
            value: match field.default_value.clone() {
                Some(SchemaFieldValue::Bool(value)) => SchemaFieldValue::Bool(value),
                _ => SchemaFieldValue::Bool(false),
            },
            label,
        },
        SchemaFieldType::Int => ControllerParam {
            param_type: ControllerParamType::Number,
            value: match field.default_value.clone() {
                Some(SchemaFieldValue::Int(value)) => SchemaFieldValue::Int(value),
                Some(SchemaFieldValue::Float(value)) => SchemaFieldValue::Float(value),
                _ => SchemaFieldValue::Int(0),
            },
            label,
        },
        SchemaFieldType::Float => ControllerParam {
            param_type: ControllerParamType::Number,
            value: match field.default_value.clone() {
                Some(SchemaFieldValue::Float(value)) => SchemaFieldValue::Float(value),
                Some(SchemaFieldValue::Int(value)) => SchemaFieldValue::Int(value),
                _ => SchemaFieldValue::Float(0.0),
            },
            label,
        },
        SchemaFieldType::String => ControllerParam {
            param_type: ControllerParamType::String,
            value: match field.default_value.clone() {
                Some(SchemaFieldValue::String(value)) => SchemaFieldValue::String(value),
                Some(other) => SchemaFieldValue::String(stringify_schema_value(&other)),
                None => SchemaFieldValue::String(String::new()),
            },
            label,
        },
        SchemaFieldType::List => ControllerParam {
            param_type: ControllerParamType::String,
            value: match field.default_value.clone() {
                Some(other) => SchemaFieldValue::String(stringify_schema_value(&other)),
                None => SchemaFieldValue::String(String::new()),
            },
            label,
        },
    }
}

fn stringify_schema_value(value: &SchemaFieldValue) -> String {
    match value {
        SchemaFieldValue::Bool(value) => value.to_string(),
        SchemaFieldValue::Int(value) => value.to_string(),
        SchemaFieldValue::Float(value) => value.to_string(),
        SchemaFieldValue::String(value) => value.clone(),
        SchemaFieldValue::List(values) => values
            .iter()
            .map(stringify_schema_value)
            .collect::<Vec<_>>()
            .join(", "),
    }
}
