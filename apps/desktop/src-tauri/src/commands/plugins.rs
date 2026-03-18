use tauri::State;

use crate::core::error::{AppError, ErrorDto};
use crate::core::models::plugin::{
    CreatePluginRequest, PluginDependency, PluginRegistry, PluginRuntime, PluginSchemaField,
    PluginType, SchemaFieldType, SchemaFieldValue, UpdatePluginRequest,
};
use crate::core::services::plugin::PluginService;
use crate::state::AppState;
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

const DEFAULT_SOURCE_FILE: &str = "main.py";

#[derive(Debug, Deserialize)]
pub struct ImportPluginFileRequest {
    pub content: String,
}

fn get_string<'a>(
    root: &'a serde_json::Map<String, Value>,
    keys: &[&str],
) -> Option<&'a str> {
    keys.iter()
        .find_map(|key| root.get(*key).and_then(Value::as_str))
}

fn get_non_empty_string(
    root: &serde_json::Map<String, Value>,
    keys: &[&str],
    field_label: &str,
) -> Result<String, ErrorDto> {
    get_string(root, keys)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| plugin_invalid_argument(format!("Campo \"{field_label}\" é obrigatório")))
}

fn get_optional_non_empty_string(
    root: &serde_json::Map<String, Value>,
    keys: &[&str],
) -> Option<String> {
    get_string(root, keys)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn get_array<'a>(
    root: &'a serde_json::Map<String, Value>,
    keys: &[&str],
) -> Option<&'a Vec<Value>> {
    keys.iter()
        .find_map(|key| root.get(*key).and_then(Value::as_array))
}

fn plugin_invalid_argument(message: impl Into<String>) -> ErrorDto {
    ErrorDto::from(AppError::InvalidArgument(message.into()))
}

fn parse_plugin_runtime(value: &str) -> Result<PluginRuntime, ErrorDto> {
    match value {
        "python" => Ok(PluginRuntime::Python),
        "rust-native" => Ok(PluginRuntime::RustNative),
        _ => Err(plugin_invalid_argument(
            "Campo \"runtime\" deve ser \"python\" ou \"rust-native\"",
        )),
    }
}

fn parse_plugin_type(value: &str) -> Result<PluginType, ErrorDto> {
    match value.trim().to_lowercase().as_str() {
        "driver" => Ok(PluginType::Driver),
        "controller" => Ok(PluginType::Controller),
        _ => Err(plugin_invalid_argument(
            "Campo \"kind\" deve ser \"driver\" ou \"controller\"",
        )),
    }
}

fn parse_schema_field_type(value: &str) -> Result<SchemaFieldType, ErrorDto> {
    match value {
        "bool" => Ok(SchemaFieldType::Bool),
        "int" => Ok(SchemaFieldType::Int),
        "float" => Ok(SchemaFieldType::Float),
        "string" => Ok(SchemaFieldType::String),
        "list" => Ok(SchemaFieldType::List),
        _ => Err(plugin_invalid_argument("schema.type inválido")),
    }
}

fn parse_schema_field_value(value: &Value) -> Result<SchemaFieldValue, ErrorDto> {
    match value {
        Value::Bool(flag) => Ok(SchemaFieldValue::Bool(*flag)),
        Value::Number(number) => {
            if let Some(integer) = number.as_i64() {
                Ok(SchemaFieldValue::Int(integer))
            } else if let Some(float) = number.as_f64() {
                Ok(SchemaFieldValue::Float(float))
            } else {
                Err(plugin_invalid_argument("Número inválido em schema.defaultValue"))
            }
        }
        Value::String(text) => Ok(SchemaFieldValue::String(text.clone())),
        Value::Array(items) => Ok(SchemaFieldValue::List(
            items.iter().map(parse_schema_field_value).collect::<Result<_, _>>()?,
        )),
        _ => Err(plugin_invalid_argument(
            "schema.defaultValue possui tipo não suportado",
        )),
    }
}

#[tauri::command]
pub fn create_plugin(
    state: State<'_, AppState>,
    request: CreatePluginRequest,
) -> Result<PluginRegistry, ErrorDto> {
    let plugin = PluginService::create(state.plugins(), request).map_err(ErrorDto::from)?;

    Ok(plugin)
}

#[tauri::command]
pub fn get_plugin(state: State<'_, AppState>, id: String) -> Result<PluginRegistry, ErrorDto> {
    PluginService::get(state.plugins(), &id).map_err(ErrorDto::from)
}

#[tauri::command]
pub fn update_plugin(
    state: State<'_, AppState>,
    request: UpdatePluginRequest,
) -> Result<PluginRegistry, ErrorDto> {
    PluginService::update(state.plugins(), request).map_err(ErrorDto::from)
}

#[tauri::command]
pub fn list_plugins(state: State<'_, AppState>) -> Vec<PluginRegistry> {
    PluginService::list(state.plugins())
}

#[tauri::command]
pub fn load_plugins(state: State<'_, AppState>) -> Result<Vec<PluginRegistry>, ErrorDto> {
    PluginService::load_all(state.plugins()).map_err(ErrorDto::from)
}

#[tauri::command]
pub fn delete_plugin(state: State<'_, AppState>, id: String) -> Result<PluginRegistry, ErrorDto> {
    PluginService::remove(state.plugins(), &id).map_err(ErrorDto::from)
}

#[tauri::command]
pub fn import_plugin_file(request: ImportPluginFileRequest) -> Result<PluginRegistry, ErrorDto> {
    let parsed: Value = serde_json::from_str(&request.content)
        .map_err(|error| plugin_invalid_argument(format!("JSON inválido: {error}")))?;
    let root = parsed
        .as_object()
        .ok_or_else(|| plugin_invalid_argument("Arquivo inválido: não é um objeto JSON"))?;

    let name = get_non_empty_string(root, &["name"], "name")?;
    let plugin_type = parse_plugin_type(
        get_string(root, &["kind", "type"])
            .ok_or_else(|| plugin_invalid_argument("Campo \"kind\" deve ser uma string não vazia"))?,
    )?;
    let runtime = parse_plugin_runtime(
        get_string(root, &["runtime"])
            .ok_or_else(|| plugin_invalid_argument("Campo \"runtime\" deve ser uma string"))?,
    )?;
    let source_file = get_optional_non_empty_string(root, &["sourceFile", "source_file"])
        .unwrap_or_else(|| DEFAULT_SOURCE_FILE.to_string());
    let source_code = get_optional_non_empty_string(root, &["sourceCode", "source_code"]);

    let schema = get_array(root, &["schema"])
        .ok_or_else(|| plugin_invalid_argument("Campo \"schema\" deve ser um array"))?
        .iter()
        .map(|field| {
            let field_obj = field
                .as_object()
                .ok_or_else(|| plugin_invalid_argument("Campo de schema inválido"))?;

            Ok(PluginSchemaField {
                name: field_obj
                    .get("name")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| plugin_invalid_argument("schema.name deve ser string"))?
                    .to_string(),
                field_type: parse_schema_field_type(
                    field_obj
                        .get("type")
                        .and_then(Value::as_str)
                        .ok_or_else(|| plugin_invalid_argument("schema.type inválido"))?,
                )?,
                default_value: field_obj.get("defaultValue")
                    .or_else(|| field_obj.get("default_value"))
                    .map(parse_schema_field_value)
                    .transpose()?,
                description: field_obj
                    .get("description")
                    .and_then(Value::as_str)
                    .map(str::to_string),
            })
        })
        .collect::<Result<Vec<_>, ErrorDto>>()?;
    let dependencies = get_array(root, &["dependencies"])
        .map(|items| {
            items
                .iter()
                .map(|dependency| {
                    let dependency_obj = dependency
                        .as_object()
                        .ok_or_else(|| plugin_invalid_argument("Dependência inválida"))?;

                    Ok(PluginDependency {
                        name: dependency_obj
                            .get("name")
                            .and_then(Value::as_str)
                            .map(str::trim)
                            .filter(|value| !value.is_empty())
                            .ok_or_else(|| plugin_invalid_argument("dependencies.name deve ser string"))?
                            .to_string(),
                        version: dependency_obj
                            .get("version")
                            .and_then(Value::as_str)
                            .map(str::trim)
                            .ok_or_else(|| {
                                plugin_invalid_argument("dependencies.version deve ser string")
                            })?
                            .to_string(),
                    })
                })
                .collect::<Result<Vec<_>, ErrorDto>>()
        })
        .transpose()?
        .unwrap_or_default();

    Ok(PluginRegistry {
        id: root
            .get("id")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| format!("plugin_{}", Uuid::new_v4())),
        name,
        plugin_type,
        runtime,
        schema,
        source_file: Some(source_file),
        source_code,
        dependencies,
        description: get_optional_non_empty_string(root, &["description"]),
        version: get_optional_non_empty_string(root, &["version"]),
        author: get_optional_non_empty_string(root, &["author"]),
    })
}

#[tauri::command]
#[allow(non_snake_case)]
pub fn list_plugins_by_type(
    state: State<'_, AppState>,
    pluginType: PluginType,
) -> Vec<PluginRegistry> {
    PluginService::list_by_type(state.plugins(), pluginType)
}
