use crate::parser::models::{EndpointData, EndpointMetadata};
use crate::ws::client::WsClient;
use serde_json::Value;
use std::collections::HashMap;

pub struct CliRunner {
    endpoint_name: String,
    endpoint_metadata: EndpointMetadata,
    endpoint_data: EndpointData,
    param_defaults: HashMap<String, Value>,
    ws_url: Option<String>,
    params: Option<String>,
    auth_username: Option<String>,
    auth_password: Option<String>,
    method: Option<u32>,
}

impl CliRunner {
    pub fn new(
        endpoint_name: String,
        endpoint_metadata: EndpointMetadata,
        endpoint_data: EndpointData,
        param_defaults: HashMap<String, Value>,
        ws_url: Option<String>,
        params: Option<String>,
        auth_username: Option<String>,
        auth_password: Option<String>,
        method: Option<u32>,
    ) -> Self {
        Self {
            endpoint_name,
            endpoint_metadata,
            endpoint_data,
            param_defaults,
            ws_url,
            params,
            auth_username,
            auth_password,
            method,
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let ws_url = self.ws_url.as_deref().unwrap_or("ws://localhost:8080");
        let auth_username = self.auth_username.as_deref().unwrap_or("rust-client");
        let auth_password = self.auth_password.as_deref().unwrap_or("pass");
        let mut ws_client =
            WsClient::new_with_debug(ws_url, auth_username, auth_password, true).await?;

        let params: HashMap<String, Value> = if let Some(params_str) = &self.params {
            serde_json::from_str(params_str)?
        } else {
            self.param_defaults.clone()
        };

        let method_id = self.method.unwrap_or(self.endpoint_metadata.method_id);

        ws_client.send_req(method_id, &params).await?;

        let response = ws_client.recv_raw().await?;
        println!("Response: {}", serde_json::to_string_pretty(&response)?);

        ws_client.close().await?;

        Ok(())
    }
}
