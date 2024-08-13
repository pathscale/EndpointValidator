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
pub enum AppScreen {
    Settings,
    Endpoints,
}

#[derive(PartialEq)]
pub enum JsonViewMode {
    Pretty,
    Raw,
}

pub struct AppState {
    pub current_screen: AppScreen,
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
    pub in_left_chunk: bool,
}

impl AppState {
    pub fn new() -> Self {
        let mut endpoint_data = HashMap::new();
        endpoint_data.insert("Endpoint 1".to_string(), ("Method1".to_string(), "Seq1".to_string(), vec!["Param1-1".to_string(), "Param1-2".to_string()]));
        endpoint_data.insert("Endpoint 2".to_string(), ("Method2".to_string(), "Seq2".to_string(), vec!["Param2-1".to_string(), "Param2-2".to_string(), "Param2-3".to_string()]));

        Self {
            current_screen: AppScreen::Settings,
            focused_settings_field: Some(SettingsField::Url),
            focused_endpoint_field: None,
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
            in_left_chunk: true,
        }
    }

    pub fn update_input(&mut self, c: char) {
        if self.current_screen == AppScreen::Settings {
            match self.focused_settings_field {
                Some(SettingsField::Url) => self.url.push(c),
                Some(SettingsField::Username) => self.username.push(c),
                Some(SettingsField::Password) => self.password.push(c),
                _ => {}
            }
        } else if self.current_screen == AppScreen::Endpoints {
            if let Some(EndpointField::Param(index)) = self.focused_endpoint_field {
                if let Some(param) = self.params.get_mut(index) {
                    param.push(c);
                }
            }
        }
    }

    pub fn delete_last_char(&mut self) {
        if self.current_screen == AppScreen::Settings {
            match self.focused_settings_field {
                Some(SettingsField::Url) => { self.url.pop(); },
                Some(SettingsField::Username) => { self.username.pop(); },
                Some(SettingsField::Password) => { self.password.pop(); },
                _ => {}
            }
        } else if self.current_screen == AppScreen::Endpoints {
            if let Some(EndpointField::Param(index)) = self.focused_endpoint_field {
                if let Some(param) = self.params.get_mut(index) {
                    param.pop();
                }
            }
        }
    }

    pub fn next_field(&mut self) {
        if self.current_screen == AppScreen::Settings {
            self.focused_settings_field = match self.focused_settings_field {
                Some(SettingsField::Url) => Some(SettingsField::Username),
                Some(SettingsField::Username) => Some(SettingsField::Password),
                Some(SettingsField::Password) => Some(SettingsField::ConnectButton),
                Some(SettingsField::ConnectButton) => Some(SettingsField::DisconnectButton),
                Some(SettingsField::DisconnectButton) => Some(SettingsField::Url),
                None => Some(SettingsField::Url),
            };
        } else if self.current_screen == AppScreen::Endpoints {
            self.focused_endpoint_field = match self.focused_endpoint_field {
                Some(EndpointField::Param(index)) => {
                    if index + 1 < self.params.len() {
                        Some(EndpointField::Param(index + 1))
                    } else {
                        Some(EndpointField::ConnectButton)
                    }
                }
                Some(EndpointField::ConnectButton) => Some(EndpointField::DisconnectButton),
                Some(EndpointField::DisconnectButton) => Some(EndpointField::JsonToggleButton),
                Some(EndpointField::JsonToggleButton) => Some(EndpointField::Param(0)),
                None => Some(EndpointField::Param(0)),
            };
        }
    }

    pub fn previous_field(&mut self) {
        if self.current_screen == AppScreen::Settings {
            self.focused_settings_field = match self.focused_settings_field {
                Some(SettingsField::Url) => Some(SettingsField::DisconnectButton),
                Some(SettingsField::Username) => Some(SettingsField::Url),
                Some(SettingsField::Password) => Some(SettingsField::Username),
                Some(SettingsField::ConnectButton) => Some(SettingsField::Password),
                Some(SettingsField::DisconnectButton) => Some(SettingsField::ConnectButton),
                None => Some(SettingsField::Url),
            };
        } else if self.current_screen == AppScreen::Endpoints {
            self.focused_endpoint_field = match self.focused_endpoint_field {
                Some(EndpointField::Param(index)) => {
                    if index > 0 {
                        Some(EndpointField::Param(index - 1))
                    } else {
                        Some(EndpointField::JsonToggleButton)
                    }
                }
                Some(EndpointField::ConnectButton) => Some(EndpointField::Param(self.params.len() - 1)),
                Some(EndpointField::DisconnectButton) => Some(EndpointField::ConnectButton),
                Some(EndpointField::JsonToggleButton) => Some(EndpointField::DisconnectButton),
                None => Some(EndpointField::Param(0)),
            };
        }
    }

    pub fn move_focus_right(&mut self) {
        if self.current_screen == AppScreen::Endpoints && self.in_left_chunk {
            self.in_left_chunk = false;
            if !self.params.is_empty() {
                self.focused_endpoint_field = Some(EndpointField::Param(0));
            } else {
                self.focused_endpoint_field = Some(EndpointField::ConnectButton);
            }
        }
    }

    pub fn move_focus_left(&mut self) {
        if self.current_screen == AppScreen::Endpoints && !self.in_left_chunk {
            self.in_left_chunk = true;
            self.focused_endpoint_field = None;
        }
    }

    pub fn handle_enter(&mut self) {
        if self.current_screen == AppScreen::Settings {
            match self.focused_settings_field {
                Some(SettingsField::ConnectButton) => self.connected = true,
                Some(SettingsField::DisconnectButton) => self.connected = false,
                _ => {}
            }
        } else if self.current_screen == AppScreen::Endpoints {
            match self.focused_endpoint_field {
                Some(EndpointField::ConnectButton) => self.connected = true,
                Some(EndpointField::DisconnectButton) => self.connected = false,
                Some(EndpointField::JsonToggleButton) => self.toggle_json_view_mode(),
                _ => {}
            }
        }
    }

    pub fn switch_screen(&mut self) {
        self.current_screen = match self.current_screen {
            AppScreen::Settings => {
                self.focused_settings_field = None;
                self.update_selected_endpoint_data();
                AppScreen::Endpoints
            }
            AppScreen::Endpoints => {
                self.focused_endpoint_field = None;
                self.focused_settings_field = Some(SettingsField::Url);
                AppScreen::Settings
            }
        };
    }

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

    pub fn toggle_json_view_mode(&mut self) {
        self.json_view_mode = match self.json_view_mode {
            JsonViewMode::Pretty => JsonViewMode::Raw,
            JsonViewMode::Raw => JsonViewMode::Pretty,
        };
    }
}

