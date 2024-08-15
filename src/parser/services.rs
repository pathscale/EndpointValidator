use crate::parser::{Services, EndpointMetadata, ParameterMetadata, Type, ParamValue, EndpointData};
use std::collections::HashMap;
use anyhow::{Result, anyhow};
use serde_json::{Value, Number, json};

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
                    method_id: endpoint.code as u32,
                    params: param_names_and_types,
                    is_stream: returns_stream,
                };

                endpoint_data.insert(endpoint.name.clone(), metadata);
            }
        }

        (endpoint_names, endpoint_data)
    }
}

impl Type {
    pub fn convert_value(&self, value: &str) -> Result<Value, anyhow::Error> {
        match self {
            Type::String => Ok(Value::String(value.to_string())),
            Type::Int => {
                let parsed_value: i32 = value.parse().map_err(anyhow::Error::msg)?;
                Ok(Value::Number(Number::from(parsed_value)))
            }
            Type::BigInt => {
                let parsed_value: i64 = value.parse().map_err(anyhow::Error::msg)?;
                Ok(Value::Number(Number::from(parsed_value)))
            }
            Type::Numeric => {
                let parsed_value: f64 = value.parse().map_err(anyhow::Error::msg)?;
                Ok(Value::Number(Number::from_f64(parsed_value).ok_or_else(|| anyhow!("Invalid number"))?))
            }
            Type::Boolean => {
                let parsed_value: bool = value.parse().map_err(anyhow::Error::msg)?;
                Ok(Value::Bool(parsed_value))
            }
            Type::TimeStampMs => {
                let parsed_value: i64 = value.parse().map_err(anyhow::Error::msg)?;
                Ok(Value::Number(Number::from(parsed_value)))
            }
            Type::Date => Ok(Value::String(value.to_string())), // Assuming dates are strings
            Type::UUID => Ok(Value::String(value.to_string())), // Assuming UUIDs are strings
            Type::Inet => Ok(Value::String(value.to_string())), // Assuming Inet is a string representation
            Type::Bytea => Ok(Value::String(value.to_string())), // Assuming Bytea is a string representation
            Type::BlockchainDecimal => Ok(Value::String(value.to_string())), // Assuming it’s a string or number
            Type::BlockchainAddress => Ok(Value::String(value.to_string())), // Assuming it’s a string
            Type::BlockchainTransactionHash => Ok(Value::String(value.to_string())), // Assuming it’s a string
            Type::Optional(inner_type) => {
                if value.is_empty() {
                    Ok(Value::Null)
                } else {
                    inner_type.convert_value(value)
                }
            }
            Type::Vec(inner_type) => {
                let values: Vec<&str> = value.split(',').collect(); // Assuming comma-separated values
                let converted_values: Result<Vec<Value>, anyhow::Error> = values.iter().map(|v| inner_type.convert_value(v)).collect();
                Ok(Value::Array(converted_values?))
            }
            Type::Struct { fields, .. } => {
                let values: HashMap<&str, &str> = value.split(',')
                    .map(|pair| {
                        let mut iter = pair.splitn(2, ':');
                        (iter.next().unwrap(), iter.next().unwrap_or(""))
                    })
                    .collect();

                let mut map = serde_json::Map::new();
                for field in fields {
                    if let Some(val) = values.get(field.name.as_str()) {
                        map.insert(field.name.clone(), field.ty.convert_value(val)?);
                    }
                }
                Ok(Value::Object(map))
            }
            Type::DataTable { fields, .. } => {
                let rows: Vec<&str> = value.split(';').collect(); // Assuming rows are separated by semicolons
                let converted_rows: Result<Vec<Value>, anyhow::Error> = rows.iter().map(|row| {
                    let values: HashMap<&str, &str> = row.split(',')
                        .map(|pair| {
                            let mut iter = pair.splitn(2, ':');
                            (iter.next().unwrap(), iter.next().unwrap_or(""))
                        })
                        .collect();

                    let mut map = serde_json::Map::new();
                    for field in fields {
                        if let Some(val) = values.get(field.name.as_str()) {
                            map.insert(field.name.clone(), field.ty.convert_value(val)?);
                        }
                    }
                    Ok(Value::Object(map))
                }).collect();

                Ok(Value::Array(converted_rows?))
            }
            Type::Enum { name, variants } => {
                if variants.iter().any(|v| v.name == value) {
                    Ok(Value::String(value.to_string()))
                } else {
                    Err(anyhow!("Invalid variant for enum {}: {}", name, value))
                }
            }
            Type::EnumRef(_name) => {
                // Assuming EnumRef behaves similarly to Enum
                Ok(Value::String(value.to_string()))
            }
            Type::StructRef(_name) => {
                // Assuming StructRef behaves similarly to Struct
                Ok(Value::String(value.to_string()))
            }
            Type::Object => Ok(json!(value)), // Assuming object as a string or raw JSON
            Type::Unit => Ok(Value::Null), // Unit type maps to Null in JSON
        }
    }
}

pub fn extract_param_defaults(
    endpoints: &HashMap<String, EndpointData>,
) -> Vec<(String, Vec<(String, String)>)> {
    let mut result = Vec::new();

    for (method_id, endpoint_data) in endpoints {
        let mut param_vec = Vec::new();
        for (param_name, param_value) in &endpoint_data.params {
            let value_str = match param_value {
                ParamValue::String(s) => s.clone(),
                ParamValue::Number(n) => n.to_string(),
                ParamValue::Bool(b) => b.to_string(),
                ParamValue::Array(arr) => format!("{:?}", arr),
                ParamValue::Object(obj) => format!("{:?}", obj),
            };
            param_vec.push((param_name.clone(), value_str));
        }
        result.push((method_id.clone(), param_vec));
    }

    result
}
