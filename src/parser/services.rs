use crate::parser::Services;
use std::collections::HashMap;

impl Services {
    pub fn extract_endpoints(&self) -> (Vec<String>, HashMap<String, (String, u32, Vec<String>)>) {
        let mut endpoint_names = Vec::new();
        let mut endpoint_data = HashMap::new();

        for service in &self.services {
            for endpoint in &service.endpoints {
                endpoint_names.push(endpoint.name.clone());

                let param_names = endpoint
                    .parameters
                    .iter()
                    .map(|param| param.name.clone())
                    .collect();

                endpoint_data.insert(
                    endpoint.name.clone(),
                    (service.name.to_string(), endpoint.code as u32, param_names),
                );
            }
        }

        (endpoint_names, endpoint_data)
    }
}
