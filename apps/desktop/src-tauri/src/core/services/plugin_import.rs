use crate::core::error::{AppError, AppResult};
use crate::core::models::plugin::{
    PluginDependency, PluginRegistry, PluginRuntime, PluginSchemaField, PluginType,
    SchemaFieldType, SchemaFieldValue,
};
use serde_json::Value;
use uuid::Uuid;

const DEFAULT_SOURCE_FILE: &str = "main.py";

pub struct PluginImportService;

impl PluginImportService {
    pub fn parse_file(content: &str) -> AppResult<PluginRegistry> {
        let parsed: Value = serde_json::from_str(content)
            .map_err(|error| invalid_argument(format!("JSON inválido: {error}")))?;
        let root = parsed
            .as_object()
            .ok_or_else(|| invalid_argument("Arquivo inválido: não é um objeto JSON"))?;

        let name = get_non_empty_string(root, &["name"], "name")?;
        let plugin_type =
            parse_plugin_type(get_string(root, &["kind", "type"]).ok_or_else(|| {
                invalid_argument("Campo \"kind\" deve ser uma string não vazia")
            })?)?;
        let runtime = parse_plugin_runtime(
            get_string(root, &["runtime"])
                .ok_or_else(|| invalid_argument("Campo \"runtime\" deve ser uma string"))?,
        )?;
        let entry_class = get_optional_non_empty_string(root, &["entryClass", "entry_class"])
            .unwrap_or_else(|| default_entry_class_for(&name, plugin_type));

        let source_file = get_optional_non_empty_string(root, &["sourceFile", "source_file"])
            .unwrap_or_else(|| DEFAULT_SOURCE_FILE.to_string());
        let source_code = get_optional_non_empty_string(root, &["sourceCode", "source_code"]);

        let schema = get_array(root, &["schema"])
            .ok_or_else(|| invalid_argument("Campo \"schema\" deve ser um array"))?
            .iter()
            .map(parse_schema_field)
            .collect::<Result<Vec<_>, _>>()?;

        let dependencies = get_array(root, &["dependencies"])
            .map(|items| {
                items
                    .iter()
                    .map(parse_dependency)
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?
            .unwrap_or_default();

        Ok(PluginRegistry {
            id: root
                .get("id")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map_or_else(|| format!("plugin_{}", Uuid::new_v4()), str::to_string),
            name,
            plugin_type,
            runtime,
            entry_class,
            schema,
            source_file: Some(source_file),
            source_code,
            dependencies,
            description: get_optional_non_empty_string(root, &["description"]),
            version: get_optional_non_empty_string(root, &["version"]),
            author: get_optional_non_empty_string(root, &["author"]),
        })
    }
}

fn parse_schema_field(field: &Value) -> AppResult<PluginSchemaField> {
    let field_obj = field
        .as_object()
        .ok_or_else(|| invalid_argument("Campo de schema inválido"))?;

    Ok(PluginSchemaField {
        name: field_obj
            .get("name")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| invalid_argument("schema.name deve ser string"))?
            .to_string(),
        field_type: parse_schema_field_type(
            field_obj
                .get("type")
                .and_then(Value::as_str)
                .ok_or_else(|| invalid_argument("schema.type inválido"))?,
        )?,
        default_value: field_obj
            .get("defaultValue")
            .or_else(|| field_obj.get("default_value"))
            .map(parse_schema_field_value)
            .transpose()?,
        description: field_obj
            .get("description")
            .and_then(Value::as_str)
            .map(str::to_string),
    })
}

fn parse_dependency(dependency: &Value) -> AppResult<PluginDependency> {
    let dependency_obj = dependency
        .as_object()
        .ok_or_else(|| invalid_argument("Dependência inválida"))?;

    Ok(PluginDependency {
        name: dependency_obj
            .get("name")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| invalid_argument("dependencies.name deve ser string"))?
            .to_string(),
        version: dependency_obj
            .get("version")
            .and_then(Value::as_str)
            .map(str::trim)
            .ok_or_else(|| invalid_argument("dependencies.version deve ser string"))?
            .to_string(),
    })
}

fn get_string<'a>(root: &'a serde_json::Map<String, Value>, keys: &[&str]) -> Option<&'a str> {
    keys.iter()
        .find_map(|key| root.get(*key).and_then(Value::as_str))
}

fn get_non_empty_string(
    root: &serde_json::Map<String, Value>,
    keys: &[&str],
    field_label: &str,
) -> AppResult<String> {
    get_string(root, keys)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| invalid_argument(format!("Campo \"{field_label}\" é obrigatório")))
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

fn invalid_argument(message: impl Into<String>) -> AppError {
    AppError::InvalidArgument(message.into())
}

fn parse_plugin_runtime(value: &str) -> AppResult<PluginRuntime> {
    match value.trim().to_ascii_lowercase().as_str() {
        "python" => Ok(PluginRuntime::Python),
        "rust-native" | "rust_native" => Ok(PluginRuntime::RustNative),
        _ => Err(invalid_argument(
            "Campo \"runtime\" deve ser \"python\" ou \"rust-native\"",
        )),
    }
}

fn parse_plugin_type(value: &str) -> AppResult<PluginType> {
    match value.trim().to_ascii_lowercase().as_str() {
        "driver" => Ok(PluginType::Driver),
        "controller" => Ok(PluginType::Controller),
        _ => Err(invalid_argument(
            "Campo \"kind\" deve ser \"driver\" ou \"controller\"",
        )),
    }
}

fn parse_schema_field_type(value: &str) -> AppResult<SchemaFieldType> {
    match value.trim().to_ascii_lowercase().as_str() {
        "bool" => Ok(SchemaFieldType::Bool),
        "int" => Ok(SchemaFieldType::Int),
        "float" => Ok(SchemaFieldType::Float),
        "string" => Ok(SchemaFieldType::String),
        "list" => Ok(SchemaFieldType::List),
        _ => Err(invalid_argument("schema.type inválido")),
    }
}

fn parse_schema_field_value(value: &Value) -> AppResult<SchemaFieldValue> {
    match value {
        Value::Bool(flag) => Ok(SchemaFieldValue::Bool(*flag)),
        Value::Number(number) => {
            if let Some(integer) = number.as_i64() {
                Ok(SchemaFieldValue::Int(integer))
            } else if let Some(float) = number.as_f64() {
                Ok(SchemaFieldValue::Float(float))
            } else {
                Err(invalid_argument("Número inválido em schema.defaultValue"))
            }
        }
        Value::String(text) => Ok(SchemaFieldValue::String(text.clone())),
        Value::Array(items) => Ok(SchemaFieldValue::List(
            items
                .iter()
                .map(parse_schema_field_value)
                .collect::<Result<_, _>>()?,
        )),
        _ => Err(invalid_argument(
            "schema.defaultValue possui tipo não suportado",
        )),
    }
}

fn default_entry_class_for(plugin_name: &str, plugin_type: PluginType) -> String {
    let fallback = match plugin_type {
        PluginType::Driver => "MyDriver",
        PluginType::Controller => "MyController",
    };

    let class_name = plugin_name
        .chars()
        .filter(|character| {
            character.is_ascii_alphanumeric()
                || character.is_ascii_whitespace()
                || *character == '_'
        })
        .collect::<String>()
        .split(|character: char| character.is_ascii_whitespace() || character == '_')
        .filter(|token| !token.is_empty())
        .map(|token| {
            let mut chars = token.chars();
            match chars.next() {
                Some(first) => {
                    let mut normalized = String::new();
                    normalized.push(first.to_ascii_uppercase());
                    for character in chars {
                        normalized.push(character.to_ascii_lowercase());
                    }
                    normalized
                }
                None => String::new(),
            }
        })
        .collect::<String>();

    if class_name.is_empty() {
        fallback.to_string()
    } else {
        class_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_file_supports_empty_dependency_version() {
        let json = r#"
        {
          "name": "Driver Teste",
          "kind": "driver",
          "runtime": "python",
          "schema": [
            {
              "name": "channels",
              "type": "list",
              "defaultValue": ["A0", 1, true]
            }
          ],
          "dependencies": [
            { "name": "numpy", "version": "" }
          ]
        }
        "#;

        let parsed = PluginImportService::parse_file(json).expect("parse should succeed");
        assert_eq!(parsed.name, "Driver Teste");
        assert_eq!(parsed.dependencies.len(), 1);
        assert_eq!(parsed.dependencies[0].name, "numpy");
        assert_eq!(parsed.dependencies[0].version, "");
    }

    #[test]
    fn parse_file_generates_id_when_missing() {
        let json = r#"
        {
          "name": "Controller Teste",
          "kind": "controller",
          "runtime": "python",
          "schema": []
        }
        "#;

        let parsed = PluginImportService::parse_file(json).expect("parse should succeed");
        assert!(parsed.id.starts_with("plugin_"));
    }
}
