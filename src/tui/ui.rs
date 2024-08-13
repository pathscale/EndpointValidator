use crate::tui::state::{AppState, AppScreen, EndpointField, SettingsField, JsonViewMode};
use crate::tui::widgets::{
    create_button,
    create_input_widget,
    create_tabs,
    create_json_viewer,
    create_list_widget,
}; 
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    Frame,
    widgets::{Block, Borders, Paragraph},
    style::{Color, Style},
};

pub fn draw_ui<B: Backend>(f: &mut Frame<B>, app_state: &mut AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Min(1), Constraint::Length(2)].as_ref())
        .split(f.size());

    draw_title(f, chunks[0]);
    draw_tabs(f, app_state, chunks[1]);

    match app_state.current_screen {
        AppScreen::Settings => draw_settings_screen(f, app_state, chunks[2]),
        AppScreen::Endpoints => draw_endpoints_screen(f, app_state, chunks[2]),
    }

    draw_help_text(f, chunks[3]);
}

fn draw_title<B: Backend>(f: &mut Frame<B>, area: Rect) {
    let title = Paragraph::new("Endpoint_Validator")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
        )
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center);
    f.render_widget(title, area);
}

fn draw_tabs<B: Backend>(f: &mut Frame<B>, app_state: &AppState, area: Rect) {
    let tabs = create_tabs(&["Settings", "Endpoints"], &app_state.current_screen);
    f.render_widget(tabs, area);
}

fn draw_help_text<B: Backend>(f: &mut Frame<B>, area: Rect) {
    let help_text = Paragraph::new("Press Esc to quit | Use arrow keys to navigate | Enter to press button")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    f.render_widget(help_text, area);
}

fn draw_settings_screen<B: Backend>(f: &mut Frame<B>, app_state: &mut AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);

    let settings_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Gray))
        .title(" Settings ")
        .title_style(Style::default().fg(Color::Gray));
    f.render_widget(settings_block, chunks[0]);

    let settings_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .margin(1)
        .split(chunks[0]);

    let url_input = create_input_widget(" URL ", &app_state.url, app_state.focused_settings_field == Some(SettingsField::Url));
    let username_input = create_input_widget(" Username ", &app_state.username, app_state.focused_settings_field == Some(SettingsField::Username));
    let password_input = create_input_widget(" Password ", &app_state.password, app_state.focused_settings_field == Some(SettingsField::Password));

    let button_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(settings_chunks[3]);

    let connect_button = create_button("Connect", app_state.connected == false, app_state.focused_settings_field == Some(SettingsField::ConnectButton));
    let disconnect_button = create_button("Disconnect", app_state.connected == true, app_state.focused_settings_field == Some(SettingsField::DisconnectButton));

    f.render_widget(url_input, settings_chunks[0]);
    f.render_widget(username_input, settings_chunks[1]);
    f.render_widget(password_input, settings_chunks[2]);
    f.render_widget(connect_button, button_chunks[0]);
    f.render_widget(disconnect_button, button_chunks[1]);

    let server_raw_response = app_state.server_raw_response.clone().unwrap_or_else(|| "".to_string());

    let status = if app_state.connected {
        Paragraph::new(server_raw_response)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Gray))
                    .title(" Resoponse ")
                    .title_style(Style::default().fg(Color::Gray))
            )
    } else {
        Paragraph::new(server_raw_response)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Gray))
                    .title(" Resoponse ")
                    .title_style(Style::default().fg(Color::Gray))
            )
    };
    f.render_widget(status, chunks[1]);
}

fn draw_endpoints_screen<B: Backend>(f: &mut Frame<B>, app_state: &mut AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(30), Constraint::Percentage(40)].as_ref())
        .split(area);

    // Left chunk for the list of endpoints
    let list_widget = create_list_widget(
        &app_state.endpoints,
        app_state.selected_endpoint,
        app_state.focused_endpoint_field.is_none(),
    );
    f.render_widget(list_widget, chunks[0]);

    // Middle chunk for the request details
    let request_block = Block::default()
        .borders(Borders::ALL)
        .title(" Request ")
        .border_style(Style::default().fg(if app_state.focused_endpoint_field.is_some() {
            Color::Yellow
        } else {
            Color::Gray
        }));
    f.render_widget(request_block, chunks[1]);

    let num_params = app_state.params.len(); // Get the count of params
    let num_items = num_params + 4; // Total number of items

    let mut constraints = Vec::new();

    // Add a fixed length constraint for each item
    for _ in 0..num_items {
        constraints.push(Constraint::Length(3));
    }

    // Add a minimum length constraint for the remaining space
    constraints.push(Constraint::Min(1));

    let request_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints.as_slice())
        .margin(1)
        .split(chunks[1]);

    // Method ID display (not an input field)
    let method_id_paragraph = Paragraph::new(app_state.method_id.clone())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Gray))
                .title(" Method ID ")
                .title_style(Style::default().fg(Color::Gray)),
        )
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Left);
    f.render_widget(method_id_paragraph, request_chunks[0]);

    // Seq display (not an input field)
    let seq_paragraph = Paragraph::new(app_state.seq.clone())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Gray))
                .title(" Seq ")
                .title_style(Style::default().fg(Color::Gray)),
        )
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Left);
    f.render_widget(seq_paragraph, request_chunks[1]);

    // Displaying input fields for parameters
    for (i, param) in app_state.params.iter().enumerate() {
        let param_label = format!(" Param{} ", i + 1);  // Store the result of format!
        let param_input = create_input_widget(
            &param_label,
            param,
            app_state.focused_endpoint_field == Some(EndpointField::Param(i)),
        );
        f.render_widget(param_input, request_chunks[i + 2]);
    }

    // Layout for Connect/Disconnect and JSON toggle buttons
    let button_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(request_chunks[request_chunks.len() - 3]);

    let connect_button = create_button(
        "Connect",
        app_state.connected == false,
        app_state.focused_endpoint_field == Some(EndpointField::ConnectButton),
    );
    let disconnect_button = create_button(
        "Disconnect",
        app_state.connected == true,
        app_state.focused_endpoint_field == Some(EndpointField::DisconnectButton),
    );
    let json_toggle_button = create_button(
        if app_state.json_view_mode == JsonViewMode::Pretty {
            " Pretty JSON "
        } else {
            " Raw JSON "
        },
        true,
        app_state.focused_endpoint_field == Some(EndpointField::JsonToggleButton),
    );

    f.render_widget(connect_button, button_chunks[0]);
    f.render_widget(disconnect_button, button_chunks[1]);
    f.render_widget(json_toggle_button, request_chunks[request_chunks.len() - 2]);

    // Right chunk for JSON or response data
    let json_viewer = create_json_viewer(&app_state.json_data);
    f.render_widget(json_viewer, chunks[2]);
}
