use crate::ws::WsClient;
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
    pub url: String,
    pub username: String,
    pub password: String,
    pub method_id: String,
    pub seq: String,
    pub params: Vec<String>,
    pub json_view_mode: JsonViewMode,
    pub json_data: Option<String>,
    pub endpoints: Vec<String>,
    pub selected_endpoint: usize,
    pub endpoint_data: HashMap<String, (String, String, Vec<String>)>,
}

impl AppState {
    pub fn new() -> Self {
        let mut endpoint_data = HashMap::new();
        endpoint_data.insert("Endpoint 1".to_string(), ("Method1".to_string(), "Seq1".to_string(), vec!["Param1-1".to_string(), "Param1-2".to_string()]));
        endpoint_data.insert("Endpoint 2".to_string(), ("Method2".to_string(), "Seq2".to_string(), vec!["Param2-1".to_string(), "Param2-2".to_string(), "Param2-3".to_string()]));

        Self {
            client: None,
            current_block: AppBlock::Settings,
            focused_settings_field: Some(SettingsField::Url),
            focused_endpoint_field: Some(EndpointField::Param(0)),
            connected: false,
            url: String::new(),
            username: String::new(),
            password: String::new(),
            method_id: String::new(),
            seq: String::new(),
            params: Vec::new(),
            json_view_mode: JsonViewMode::Pretty,
            json_data: None,
            endpoints: vec!["Endpoint 1".to_string(), "Endpoint 2".to_string()],
            selected_endpoint: 0,
            endpoint_data,
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
            if let Some(param) = self.params.get_mut(index) {
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
            if let Some(param) = self.params.get_mut(index) {
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
        }
    }

    pub fn previous_field(&mut self) {
        match self.current_block {
            AppBlock::Settings => self.focused_settings_field = self.previous_settings_field(),
            AppBlock::EndpointList => self.select_previous_endpoint(),
            AppBlock::EndpointsReq => self.focused_endpoint_field = self.previous_endpoint_field(),
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
            Some(EndpointField::JsonToggleButton) => Some(EndpointField::Param(0)),
            None => Some(EndpointField::Param(0)),
        }
    }

    fn previous_endpoint_field(&self) -> Option<EndpointField> {
        match self.focused_endpoint_field {
            Some(EndpointField::Param(index)) if index > 0 => {
                Some(EndpointField::Param(index - 1))
            }
            Some(EndpointField::Param(_)) => Some(EndpointField::JsonToggleButton),
            Some(EndpointField::ConnectButton) => Some(EndpointField::Param(self.params.len() - 1)),
            Some(EndpointField::DisconnectButton) => Some(EndpointField::ConnectButton),
            Some(EndpointField::JsonToggleButton) => Some(EndpointField::DisconnectButton),
            None => Some(EndpointField::Param(0)),
        }
    }

    // Block switching
    pub fn switch_block(&mut self) {
        self.current_block = match self.current_block {
            AppBlock::Settings => {
                self.update_selected_endpoint_data();
                AppBlock::EndpointList 
            },
            AppBlock::EndpointList => {
                self.update_selected_endpoint_data();
                AppBlock::EndpointsReq
            }
            AppBlock::EndpointsReq => AppBlock::Settings,
        };
    }

    // Endpoint selection
    pub fn select_next_endpoint(&mut self) {
        if self.selected_endpoint < self.endpoints.len() - 1 {
            self.selected_endpoint += 1;
            self.update_selected_endpoint_data();
        }
    }

    pub fn select_previous_endpoint(&mut self) {
        if self.selected_endpoint > 0 {
            self.selected_endpoint -= 1;
            self.update_selected_endpoint_data();
        }
    }

    fn update_selected_endpoint_data(&mut self) {
        if let Some(endpoint) = self.endpoints.get(self.selected_endpoint) {
            if let Some((method_id, seq, params)) = self.endpoint_data.get(endpoint) {
                self.method_id = method_id.clone();
                self.seq = seq.clone();
                self.params = params.clone();
            }
        }
    }

    // JSON view toggle
    pub fn toggle_json_view_mode(&mut self) {
        self.json_view_mode = match self.json_view_mode {
            JsonViewMode::Pretty => JsonViewMode::Raw,
            JsonViewMode::Raw => JsonViewMode::Pretty,
        };
    }

    // Handle connection and disconnection
    pub async fn handle_enter(&mut self) -> Result<()> {
        if self.current_block == AppBlock::Settings {
            match self.focused_settings_field {
                Some(SettingsField::ConnectButton) => {
                    if let Err(err) = self.handle_connect().await {
                        self.connected = false;
                    }
                }
                Some(SettingsField::DisconnectButton) => {
                    if let Err(err) = self.handle_disconnect().await {
                        self.connected = true;
                    }
                }
                _ => {}
            }
        } else if self.current_block == AppBlock::EndpointsReq {
            match self.focused_endpoint_field {
                Some(EndpointField::ConnectButton) => {
                    if let Err(err) = self.handle_endpoint_connect().await {
                    }
                }
                Some(EndpointField::DisconnectButton) => {
                    if let Err(err) = self.handle_endpoint_disconnect().await {
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
        self.connected = true;
        Ok(())
    }

    pub async fn handle_disconnect(&mut self) -> Result<()> {
        if let Some(mut client) = self.client.take() {
            client.close().await?;
        }
        self.connected = false;
        Ok(())
    }

    pub async fn handle_endpoint_connect(&mut self) -> Result<()> {
        // if let Some(client) = self.client.as_mut() {
        //     let response = client
        //         .send_req(self.method_id.clone(), &self.params)
        //         .await
        //         .context("Failed to send request to endpoint")?;
        //     self.json_data = Some(serde_json::to_string_pretty(&response)?);
        // }
        Ok(())
    }

    pub async fn handle_endpoint_disconnect(&mut self) -> Result<()> {
        self.json_data = None;
        Ok(())
    }
}

