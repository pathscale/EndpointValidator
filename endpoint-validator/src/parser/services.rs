use crate::parser::{
    EndpointData, EndpointMetadata, ParamValue, ParameterMetadata, Services, Type,
};
use anyhow::{Context, Result, anyhow, bail};
use serde_json::{Number, Value};
use std::collections::HashMap;

/// Turning a `config.toml` string into a wire value.
///
/// A trait because `Type` lives in `endpoint-libs` now, so an inherent impl is
/// not ours to write.
pub trait ConvertValue {
    fn convert_value(&self, value: &str) -> Result<Value>;
}

impl Services {
    pub fn extract_endpoints(&self) -> (Vec<String>, HashMap<String, EndpointMetadata>) {
        let mut endpoint_names = Vec::new();
        let mut endpoint_data = HashMap::new();

        for service in &self.services {
            for endpoint in &service.endpoints {
                endpoint_names.push(endpoint.name.clone());

                let param_names_and_types = endpoint
                    .parameters
                    .iter()
                    .map(|param| ParameterMetadata {
                        name: param.name.clone(),
                        ty: param.ty.clone(),
                    })
                    .collect();

                let returns_stream = endpoint.stream_response.is_some();

                let metadata = EndpointMetadata {
                    service_name: service.name.clone(),
                    method_id: endpoint.code,
                    params: param_names_and_types,
                    is_stream: returns_stream,
                };

                endpoint_data.insert(endpoint.name.clone(), metadata);
            }
        }

        (endpoint_names, endpoint_data)
    }
}

impl ConvertValue for Type {
    /// Converts a `config.toml` string into the JSON value the wire expects.
    ///
    /// Mirrors `endpoint_libs::model::Type::to_json_schema`: whatever that says
    /// a field looks like on the wire is what this must produce, or the server
    /// rejects the call.
    fn convert_value(&self, value: &str) -> Result<Value> {
        Ok(match self {
            Type::String
            | Type::UUID
            | Type::Bytea
            | Type::IpAddr
            | Type::NanoId { .. }
            | Type::BlockchainDecimal
            | Type::BlockchainAddress
            | Type::BlockchainTransactionHash => Value::String(value.to_string()),

            Type::UInt32 => Value::Number(Number::from(value.parse::<u32>()?)),
            Type::Int32 => Value::Number(Number::from(value.parse::<i32>()?)),
            Type::Int64 | Type::TimeStampMs => Value::Number(Number::from(value.parse::<i64>()?)),
            Type::Float64 => Value::Number(
                Number::from_f64(value.parse::<f64>()?)
                    .ok_or_else(|| anyhow!("`{value}` is not a finite number"))?,
            ),
            Type::Boolean => Value::Bool(value.parse::<bool>()?),
            Type::Unit => Value::Null,
            Type::Object => serde_json::from_str(value)
                .with_context(|| format!("`{value}` is not valid JSON for an Object field"))?,

            Type::Optional(inner) => {
                if value.is_empty() {
                    Value::Null
                } else {
                    inner.convert_value(value)?
                }
            }
            Type::Vec(inner) => Value::Array(
                split_top_level(value, ',')
                    .iter()
                    .map(|v| inner.convert_value(v))
                    .collect::<Result<Vec<_>>>()?,
            ),

            Type::Struct { name, fields } => {
                let supplied: HashMap<&str, &str> = split_top_level(value, ',')
                    .into_iter()
                    .filter_map(|pair| pair.split_once(':'))
                    .map(|(k, v)| (k.trim(), v.trim()))
                    .collect();

                let mut map = serde_json::Map::new();
                for field in fields {
                    match supplied.get(field.name.as_str()) {
                        Some(v) => {
                            map.insert(field.name.clone(), field.ty.convert_value(v)?);
                        }
                        // A missing Optional is simply absent; a missing required
                        // field is a config error worth naming.
                        None if matches!(field.ty, Type::Optional(_)) => {}
                        None => bail!("struct `{name}`: missing required field `{}`", field.name),
                    }
                }
                Value::Object(map)
            }

            // Enums go over the wire as their integer value, not their name --
            // see `enum_to_schema` upstream, which emits `type: integer` with a
            // `const` per variant. Accept either spelling in config.
            Type::Enum { name, variants } => {
                if let Some(variant) = variants.iter().find(|v| v.name == value) {
                    Value::Number(Number::from(variant.value))
                } else if let Ok(n) = value.parse::<i64>() {
                    if variants.iter().any(|v| v.value == n) {
                        Value::Number(Number::from(n))
                    } else {
                        bail!("enum `{name}`: no variant has value {n}");
                    }
                } else {
                    bail!(
                        "enum `{name}`: `{value}` is not a variant. Expected one of: {}",
                        variants
                            .iter()
                            .map(|v| v.name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                }
            }

            // References cannot be resolved without the registry, so accept raw
            // JSON and let the server validate. Better than the previous
            // behaviour, which silently sent the string.
            Type::StructRef(name) | Type::EnumRef { name, .. } => serde_json::from_str(value)
                .with_context(|| format!("`{value}` is not valid JSON for `{name}`"))?,
            Type::StructTable { struct_ref } => serde_json::from_str(value)
                .with_context(|| format!("`{value}` is not valid JSON for table `{struct_ref}`"))?,

            // `Type` is #[non_exhaustive] upstream: a new variant must fail
            // loudly here rather than be silently mis-encoded.
            other => {
                bail!("unsupported parameter type {other:?} — endpoint-validator needs updating")
            }
        })
    }
}

/// Splits on `sep`, ignoring separators nested inside `{}`, `[]` or quotes.
///
/// The previous implementation used a plain `split(',')`, which corrupted any
/// nested struct or array value.
fn split_top_level(value: &str, sep: char) -> Vec<&str> {
    let (mut parts, mut depth, mut start, mut quoted) = (Vec::new(), 0i32, 0usize, false);
    for (i, c) in value.char_indices() {
        match c {
            '"' => quoted = !quoted,
            '{' | '[' if !quoted => depth += 1,
            '}' | ']' if !quoted => depth -= 1,
            c if c == sep && depth == 0 && !quoted => {
                parts.push(value[start..i].trim());
                start = i + c.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(value[start..].trim());
    parts.into_iter().filter(|p| !p.is_empty()).collect()
}

pub fn extract_param_defaults(
    endpoints: &HashMap<String, EndpointData>,
) -> HashMap<String, HashMap<String, String>> {
    let mut result = HashMap::new();

    for (method_id, endpoint_data) in endpoints {
        let mut param_map = HashMap::new();
        for (param_name, param_value) in &endpoint_data.params {
            let value_str = match param_value {
                ParamValue::String(s) => s.clone(),
                ParamValue::Number(n) => n.to_string(),
                ParamValue::Bool(b) => b.to_string(),
                ParamValue::Array(arr) => format!("{:?}", arr),
                ParamValue::Object(obj) => format!("{:?}", obj),
            };
            param_map.insert(param_name.to_string(), value_str);
        }
        result.insert(method_id.to_string(), param_map);
    }

    result
}
