use crate::parser::{Services, EndpointMetadata, ParameterMetadata, Type};
use std::collections::HashMap;

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
                        ty: extract_type(&param.ty),
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

fn extract_type(ty: &Type) -> String {
    match ty {
        Type::TimeStampMs => "TimeStampMs".to_string(),
        Type::Date => "Date".to_string(),
        Type::Int => "Int".to_string(),
        Type::BigInt => "BigInt".to_string(),
        Type::Numeric => "Numeric".to_string(),
        Type::Boolean => "Boolean".to_string(),
        Type::String => "String".to_string(),
        Type::Bytea => "Bytea".to_string(),
        Type::UUID => "UUID".to_string(),
        Type::Inet => "Inet".to_string(),
        Type::Struct { name, fields } => {
            let fields_str = fields
                .iter()
                .map(|field| format!("{}: {}", field.name, extract_type(&field.ty)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("Struct<{}> {{ {} }}", name, fields_str)
        }
        Type::StructRef(name) => format!("StructRef<{}>", name),
        Type::Object => "Object".to_string(),
        Type::DataTable { name, fields } => {
            let fields_str = fields
                .iter()
                .map(|field| format!("{}: {}", field.name, extract_type(&field.ty)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("DataTable<{}> {{ {} }}", name, fields_str)
        }
        Type::Vec(inner_type) => format!("Vec<{}>", extract_type(inner_type)),
        Type::Unit => "Unit".to_string(),
        Type::Optional(inner_type) => format!("Optional<{}>", extract_type(inner_type)),
        Type::Enum { name, variants } => {
            let variants_str = variants
                .iter()
                .map(|variant| variant.name.clone())
                .collect::<Vec<_>>()
                .join(", ");
            format!("Enum<{}> {{ {} }}", name, variants_str)
        }
        Type::EnumRef(name) => format!("EnumRef<{}>", name),
        Type::BlockchainDecimal => "BlockchainDecimal".to_string(),
        Type::BlockchainAddress => "BlockchainAddress".to_string(),
        Type::BlockchainTransactionHash => "BlockchainTransactionHash".to_string(),
    }
}
