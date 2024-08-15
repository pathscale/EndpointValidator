use crate::ws::WsClient;
use crate::parser::{EndpointMetadata, ParameterMetadata};
use anyhow::{Context, Result};
use std::collections::HashMap;

#[derive(PartialEq)]
pub enum SettingsField {
    Url,
    Username,
    Password,
    ConnectButton,
    DisconnectButton,
}

#[derive(PartialEq)]
pub enum EndpointField {
    Param(usize),
    ConnectButton,
    DisconnectButton,
    JsonToggleButton,
}

#[derive(PartialEq)]
pub enum AppBlock {
    Settings,
    EndpointList,
    EndpointsReq,
    EndpointsRes,
}

#[derive(PartialEq)]
pub enum JsonViewMode {
    Pretty,
    Raw,
}

pub struct AppState {
    pub client: Option<WsClient>,
    pub current_block: AppBlock,
    pub focused_settings_field: Option<SettingsField>,
    pub focused_endpoint_field: Option<EndpointField>,
    pub connected: bool,
    pub endpoint_connected: bool,
    pub url: String,
    pub username: String,
    pub password: String,
    pub method_id: Option<u32>,
    pub service_name: Option<String>,
    pub params: Vec<ParameterMetadata>,
    pub param_values: Vec<String>,
    pub json_view_mode: JsonViewMode,
    pub json_data: Option<String>,
    pub endpoints: Vec<String>,
    pub selected_endpoint: usize,
    pub endpoint_data: HashMap<String, EndpointMetadata>,
    pub response_scroll: (u16, u16),
    pub is_stream: bool,
}

impl AppState {
    pub fn new(endpoint_names: Vec<String>, endpoint_data: HashMap<String, EndpointMetadata>) -> Self {
        Self {
            client: None,
            current_block: AppBlock::Settings,
            focused_settings_field: Some(SettingsField::Url),
            focused_endpoint_field: Some(EndpointField::Param(0)),
            connected: false,
            endpoint_connected: false,
            url: "ws://localhost:8443".to_string(),
            username: String::new(),
            password: String::new(),
            method_id: None,
            service_name: None,
            params: Vec::new(),
            param_values: Vec::new(),
            json_view_mode: JsonViewMode::Pretty,
            json_data: None,
            endpoints: endpoint_names,
            selected_endpoint: 0,
            endpoint_data: endpoint_data,
            response_scroll: (0, 0),
            is_stream: false,
        }
    }

    // Scroll logic for the response section
    pub fn scroll_response_down(&mut self) {
        self.response_scroll.0 += 1;
    }

    pub fn scroll_response_up(&mut self) {
        if self.response_scroll.0 > 0 {
            self.response_scroll.0 -= 1;
        }
    }

    pub fn scroll_response_right(&mut self) {
        if self.current_block == AppBlock::EndpointsRes {
            self.response_scroll.1 += 1;
        }
    }

    pub fn scroll_response_left(&mut self) {
        if self.current_block == AppBlock::EndpointsRes {
            if self.response_scroll.1 > 0 {
                self.response_scroll.1 -= 1;
            }
        }
    }

    // Handle user input
    pub fn update_input(&mut self, c: char) {
        match self.current_block {
            AppBlock::Settings => self.update_settings_input(c),
            AppBlock::EndpointsReq => self.update_endpoint_input(c),
            _ => {}
        }
    }

    fn update_settings_input(&mut self, c: char) {
        match self.focused_settings_field {
            Some(SettingsField::Url) => self.url.push(c),
            Some(SettingsField::Username) => self.username.push(c),
            Some(SettingsField::Password) => self.password.push(c),
            _ => {}
        }
    }

    fn update_endpoint_input(&mut self, c: char) {
        if let Some(EndpointField::Param(index)) = self.focused_endpoint_field {
            if let Some(param) = self.param_values.get_mut(index) {
                param.push(c);
            }
        }
    }

    pub fn delete_last_char(&mut self) {
        match self.current_block {
            AppBlock::Settings => self.delete_last_char_from_settings(),
            AppBlock::EndpointsReq => self.delete_last_char_from_endpoint(),
            _ => {}
        }
    }

    fn delete_last_char_from_settings(&mut self) {
        match self.focused_settings_field {
            Some(SettingsField::Url) => { self.url.pop(); },
            Some(SettingsField::Username) => { self.username.pop(); },
            Some(SettingsField::Password) => { self.password.pop(); },
            _ => {}
        }
    }

    fn delete_last_char_from_endpoint(&mut self) {
        if let Some(EndpointField::Param(index)) = self.focused_endpoint_field {
            if let Some(param) = self.param_values.get_mut(index) {
                param.pop();
            }
        }
    }

    // Navigation within blocks
    pub fn next_field(&mut self) {
        match self.current_block {
            AppBlock::Settings => self.focused_settings_field = self.next_settings_field(),
            AppBlock::EndpointList => self.select_next_endpoint(),
            AppBlock::EndpointsReq => self.focused_endpoint_field = self.next_endpoint_field(),
            AppBlock::EndpointsRes => self.scroll_response_down(),
        }
    }

    pub fn previous_field(&mut self) {
        match self.current_block {
            AppBlock::Settings => self.focused_settings_field = self.previous_settings_field(),
            AppBlock::EndpointList => self.select_previous_endpoint(),
            AppBlock::EndpointsReq => self.focused_endpoint_field = self.previous_endpoint_field(),
            AppBlock::EndpointsRes => self.scroll_response_up(),
        }
    }

    fn next_settings_field(&self) -> Option<SettingsField> {
        match self.focused_settings_field {
            Some(SettingsField::Url) => Some(SettingsField::Username),
            Some(SettingsField::Username) => Some(SettingsField::Password),
            Some(SettingsField::Password) => Some(SettingsField::ConnectButton),
            Some(SettingsField::ConnectButton) => Some(SettingsField::DisconnectButton),
            Some(SettingsField::DisconnectButton) => Some(SettingsField::Url),
            None => Some(SettingsField::Url),
        }
    }

    fn previous_settings_field(&self) -> Option<SettingsField> {
        match self.focused_settings_field {
            Some(SettingsField::Url) => Some(SettingsField::DisconnectButton),
            Some(SettingsField::Username) => Some(SettingsField::Url),
            Some(SettingsField::Password) => Some(SettingsField::Username),
            Some(SettingsField::ConnectButton) => Some(SettingsField::Password),
            Some(SettingsField::DisconnectButton) => Some(SettingsField::ConnectButton),
            None => Some(SettingsField::Url),
        }
    }

    fn next_endpoint_field(&self) -> Option<EndpointField> {
        match self.focused_endpoint_field {
            Some(EndpointField::Param(index)) if index + 1 < self.params.len() => {
                Some(EndpointField::Param(index + 1))
            }
            Some(EndpointField::Param(_)) => Some(EndpointField::ConnectButton),
            Some(EndpointField::ConnectButton) => Some(EndpointField::DisconnectButton),
            Some(EndpointField::DisconnectButton) => Some(EndpointField::JsonToggleButton),
            Some(EndpointField::JsonToggleButton) | None => {
                if self.params.is_empty() {
                    Some(EndpointField::ConnectButton)
                } else {
                    Some(EndpointField::Param(0))
                }
            }
        }
    }
    
    fn previous_endpoint_field(&self) -> Option<EndpointField> {
        match self.focused_endpoint_field {
            Some(EndpointField::Param(index)) if index > 0 => {
                Some(EndpointField::Param(index - 1))
            }
            Some(EndpointField::Param(_)) => Some(EndpointField::JsonToggleButton),
            Some(EndpointField::JsonToggleButton) => Some(EndpointField::DisconnectButton),
            Some(EndpointField::DisconnectButton) => Some(EndpointField::ConnectButton),
            Some(EndpointField::ConnectButton) | None => {
                if self.params.is_empty() {
                    Some(EndpointField::JsonToggleButton)
                } else {
                    Some(EndpointField::Param(self.params.len() - 1))
                }
            }
        }
    }    
    
    // Block switching
    pub fn switch_block(&mut self) {
        if self.connected {
            self.current_block = match self.current_block {
                AppBlock::Settings => {
                    self.update_selected_endpoint_data();
                    AppBlock::EndpointList 
                },
                AppBlock::EndpointList => {
                    self.update_selected_endpoint_data();
                    AppBlock::EndpointsReq
                }
                AppBlock::EndpointsReq => AppBlock::EndpointsRes,
                AppBlock::EndpointsRes => AppBlock::Settings,
            };
        }
    }

    // Endpoint selection
    pub fn select_next_endpoint(&mut self) {
        if self.connected {
            if self.selected_endpoint < self.endpoints.len() - 1 {
                self.selected_endpoint += 1;
                self.update_selected_endpoint_data();
            }
        }
    }

    pub fn select_previous_endpoint(&mut self) {
        if self.connected {
            if self.selected_endpoint > 0 {
                self.selected_endpoint -= 1;
                self.update_selected_endpoint_data();
            }
        }
    }

    fn update_selected_endpoint_data(&mut self) {
        if let Some(endpoint) = self.endpoints.get(self.selected_endpoint) {
            if let Some(metadata) = self.endpoint_data.get(endpoint) {
                self.method_id = Some(metadata.method_id);
                self.service_name = Some(metadata.service_name.clone());
                self.params = metadata.params.clone();
                self.param_values = vec!["".to_string(); self.params.len()];
                self.is_stream = metadata.is_stream;
            }
        }
    }
    
    pub fn toggle_json_view_mode(&mut self) {
        if let Some(raw_json) = self.json_data.as_ref() {
            self.json_view_mode = match self.json_view_mode {
                JsonViewMode::Pretty => {
                    let raw_json = serde_json::to_string(
                        &serde_json::from_str::<serde_json::Value>(raw_json).unwrap_or_default()
                    ).unwrap_or_else(|_| raw_json.clone());
                    self.json_data = Some(raw_json);
                    JsonViewMode::Raw
                },
                JsonViewMode::Raw => {
                    let pretty_json = serde_json::to_string_pretty(
                        &serde_json::from_str::<serde_json::Value>(raw_json).unwrap_or_default()
                    ).unwrap_or_else(|_| raw_json.clone());
                    self.json_data = Some(pretty_json);
                    JsonViewMode::Pretty
                },
            };
        }
    }
    
    // Handle connection and disconnection
    pub async fn handle_enter(&mut self) -> Result<()> {
        if self.current_block == AppBlock::Settings {
            match self.focused_settings_field {
                Some(SettingsField::ConnectButton) => {
                    if let Err(_err) = self.handle_connect().await {
                        self.connected = false;
                        self.json_data = Some(_err.to_string());
                    }
                }
                Some(SettingsField::DisconnectButton) => {
                    if let Err(_err) = self.handle_disconnect().await {
                        self.connected = true;
                        self.json_data = Some(_err.to_string());
                    }
                }
                _ => {}
            }
        } else if self.current_block == AppBlock::EndpointsReq {
            match self.focused_endpoint_field {
                Some(EndpointField::ConnectButton) => {
                    if let Err(_err) = self.handle_endpoint_connect().await {
                        self.endpoint_connected = false;
                        self.json_data = Some(_err.to_string());
                    }
                }
                Some(EndpointField::DisconnectButton) => {
                    if let Err(_err) = self.handle_endpoint_disconnect().await {
                        self.endpoint_connected = true;
                        self.json_data = Some(_err.to_string());
                    }
                }
                Some(EndpointField::JsonToggleButton) => self.toggle_json_view_mode(),
                _ => {}
            }
        }

        Ok(())
    }

    pub async fn handle_connect(&mut self) -> Result<()> {
        let headers = format!(
            "0login, 1{}, 2{}, 3User, 424787297130491616, 5android",
            self.username, self.password
        );
        
        let client = WsClient::new(&self.url, &headers)
            .await
            .context("Failed to connect to WebSocket")?;
        self.client = Some(client);
    
        let client = self.client.as_mut().context("WebSocket client is not connected")?;
        let raw_response = client.recv_raw().await.context("Failed to receive response from WebSocket")?;
        
        let formatted_response = match self.json_view_mode {
            JsonViewMode::Pretty => {
                serde_json::to_string_pretty(&raw_response).context("Failed to format JSON as pretty")?
            }
            JsonViewMode::Raw => {
                serde_json::to_string(&raw_response).context("Failed to format JSON as raw")?
            }
        };
        
        let resp = format!("Connected to {}\n{}", self.url, formatted_response);        
        self.json_data = Some(resp);
        self.connected = true;
    
        Ok(())
    }
    

    pub async fn handle_disconnect(&mut self) -> Result<()> {
        if let Some(client) = self.client.take() {
            client.close().await?;
        }
        let resp = format!("Disconnected from to {}", self.url);
        self.json_data = Some(resp);
        self.connected = false;
        Ok(())
    }

    pub async fn handle_endpoint_connect(&mut self) -> Result<()> { 
        let client = self.client.as_mut().context("WebSocket client is not connected")?;
        client.send_req(self.method_id.unwrap(), &self.param_values).await.context("Failed to send request to WebSocket")?;
        let raw_response = client.recv_raw().await.context("Failed to receive response from WebSocket")?;

        let resp = match self.json_view_mode {
            JsonViewMode::Pretty => {
                serde_json::to_string_pretty(&raw_response).context("Failed to format JSON as pretty")?
            }
            JsonViewMode::Raw => {
                serde_json::to_string(&raw_response).context("Failed to format JSON as raw")?
            }
        };
        
        self.endpoint_connected = true;
        self.json_data = Some(resp); 
        Ok(())
    }

    pub async fn handle_endpoint_disconnect(&mut self) -> Result<()> {
        self.endpoint_connected = false;
        self.json_data = None;
        Ok(())
    }
}

