use crate::tui::state::{AppState, EndpointField, SettingsField, JsonViewMode, AppBlock};
use crate::tui::widgets::{
    create_button,
    create_input_widget,
    create_json_viewer,
    create_list_widget,
}; 
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
    widgets::{Block, Borders, Paragraph},
    style::{Color, Style},
    text::Text,
};

pub fn draw_ui<B: Backend>(f: &mut Frame<B>, app_state: &mut AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(1), Constraint::Length(2)].as_ref())
        .split(f.size());

    draw_settings_screen(f, app_state, chunks[0]);

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[1]);

    draw_endpoints_screen(f, app_state, main_chunks[0]);
    draw_response_screen(f, app_state, main_chunks[1]);

    draw_help_text(f, chunks[2]);
}

fn draw_help_text<B: Backend>(f: &mut Frame<B>, area: Rect) {
    let help_text = Paragraph::new("Press Esc to quit | Use Tab key to switch block | Use arrow keys to navigate | Enter to press button")
        .style(Style::default().fg(Color::Gray))
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(help_text, area);
}

fn draw_settings_screen<B: Backend>(f: &mut Frame<B>, app_state: &mut AppState, area: Rect) {
    let is_focused = app_state.current_block == AppBlock::Settings;
    let settings_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if is_focused { Color::Yellow } else { Color::Gray }))
        .title(" Settings ")
        .title_style(Style::default().fg(if is_focused { Color::Yellow } else { Color::Gray }));

    f.render_widget(settings_block, area);

    let settings_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ])
        .margin(1)
        .split(area);

    let url_input = create_input_widget(" URL ", &app_state.url, app_state.focused_settings_field == Some(SettingsField::Url));
    let username_input = create_input_widget(" Username ", &app_state.username, app_state.focused_settings_field == Some(SettingsField::Username));
    let password_input = create_input_widget(" Password ", &app_state.password, app_state.focused_settings_field == Some(SettingsField::Password));

    let connect_button = create_button(
        "Connect",
        !app_state.connected,
        app_state.focused_settings_field == Some(SettingsField::ConnectButton),
    );

    let disconnect_button = create_button(
        "Disconnect",
        app_state.connected,
        app_state.focused_settings_field == Some(SettingsField::DisconnectButton),
    );

    f.render_widget(url_input, settings_chunks[0]);
    f.render_widget(username_input, settings_chunks[1]);
    f.render_widget(password_input, settings_chunks[2]);
    f.render_widget(connect_button, settings_chunks[3]);
    f.render_widget(disconnect_button, settings_chunks[4]);
}

fn draw_endpoints_screen<B: Backend>(f: &mut Frame<B>, app_state: &mut AppState, area: Rect) {
    let endpoint_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);

    let is_focused = app_state.current_block == AppBlock::EndpointList;
    if app_state.connected {
        let list_widget = create_list_widget(
            &app_state.endpoints,
            app_state.selected_endpoint,
            is_focused,
        );
        f.render_widget(list_widget, endpoint_chunks[0]);
    } else {
        let empty_block = Block::default()
            .borders(Borders::ALL)
            .title(" Endpoint List ")
            .border_style(Style::default().fg(if is_focused {
                Color::Yellow
            } else {
                Color::Gray
            }))
            .title_style(Style::default().fg(if is_focused {
                Color::Yellow
            } else {
                Color::Gray
            }));

        f.render_widget(empty_block, endpoint_chunks[0]);
    }


    let is_focused = app_state.current_block == AppBlock::EndpointsReq;

    let request_block = Block::default()
        .borders(Borders::ALL)
        .title(" Request ")
        .border_style(Style::default().fg(if is_focused {
            Color::Yellow
        } else {
            Color::Gray
        }))
        .title_style(Style::default().fg(if is_focused {
            Color::Yellow
        } else {
            Color::Gray
        }));

    f.render_widget(request_block, endpoint_chunks[1]);

    let num_params = app_state.params.len();
    let num_items = num_params + 4;

    let mut constraints = Vec::new();

    for _ in 0..num_items {
        constraints.push(Constraint::Length(3));
    }

    constraints.push(Constraint::Min(1));

    let request_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints.as_slice())
        .margin(1)
        .split(endpoint_chunks[1]);

    let service_name_text = match &app_state.service_name {
        Some(service_name) => service_name.clone(),
        None => String::from(""),
    };

    let service_name_paragraph = Paragraph::new(Text::raw(service_name_text))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Gray))
                .title(" Service Name ")
                .title_style(Style::default().fg(Color::Gray)),
        )
        .style(Style::default().fg(Color::Gray))
        .alignment(ratatui::layout::Alignment::Left);
    f.render_widget(service_name_paragraph, request_chunks[0]);

    let method_id_text = match app_state.method_id {
        Some(method_id) => method_id.to_string(),
        None => String::from(""),
    };
    
    let method_id_paragraph = Paragraph::new(Text::raw(method_id_text))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Gray))
                .title(" Method ID ")
                .title_style(Style::default().fg(Color::Gray)),
        )
        .style(Style::default().fg(Color::Gray))
        .alignment(ratatui::layout::Alignment::Left);
    f.render_widget(method_id_paragraph, request_chunks[1]);

    for (i, param) in app_state.params.iter().enumerate() {
        let param_label = format!(" {} ", param);
        let param_input = create_input_widget(
            &param_label,
            "",
            app_state.focused_endpoint_field == Some(EndpointField::Param(i)),
        );
        f.render_widget(param_input, request_chunks[i + 2]);
    }

    // Layout for Connect and Disconnect buttons
    let button_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(request_chunks[request_chunks.len() - 3]);

    let connect_button = create_button(
        "Connect",
        !app_state.endpoint_connected,
        app_state.focused_endpoint_field == Some(EndpointField::ConnectButton),
    );

    let disconnect_button = create_button(
        "Disconnect",
        app_state.endpoint_connected,
        app_state.focused_endpoint_field == Some(EndpointField::DisconnectButton),
    );

    f.render_widget(connect_button, button_chunks[0]);
    f.render_widget(disconnect_button, button_chunks[1]);

    let json_toggle_button = create_button(
        if app_state.json_view_mode == JsonViewMode::Pretty {
            " Pretty JSON "
        } else {
            " Raw JSON "
        },
        true,
        app_state.focused_endpoint_field == Some(EndpointField::JsonToggleButton),
    );

    f.render_widget(json_toggle_button, request_chunks[request_chunks.len() - 2]);
}

fn draw_response_screen<B: Backend>(f: &mut Frame<B>, app_state: &mut AppState, area: Rect) {    

    let is_focused = app_state.current_block == AppBlock::EndpointsRes;
    let json_viewer = create_json_viewer(&app_state.json_data, is_focused)
        .scroll((app_state.response_scroll.0, app_state.response_scroll.1));
    f.render_widget(json_viewer, area);
}
