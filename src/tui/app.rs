use crate::tui::state::{AppState, JsonViewMode, EndpointField};
use crate::tui::ui::draw_ui;
use crate::parser::{EndpointMetadata};
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use anyhow::{Result, Context};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{self, Duration};

pub async fn run(endpoint_names: Vec<String>, endpoint_data: HashMap<String, EndpointMetadata>, param_defaults: Vec<(String, Vec<(String, String)>)>) -> Result<()> {
    // Set up terminal in raw mode
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Arc::new(Mutex::new(Terminal::new(backend)?));

    // Initialize app state with shared state
    let app_state = Arc::new(Mutex::new(AppState::new(endpoint_names, endpoint_data, param_defaults)));

    // Spawn a task to handle TUI updates
    let terminal_clone = Arc::clone(&terminal);
    let app_state_clone = Arc::clone(&app_state);
    tokio::spawn(async move {
        let mut ticker = time::interval(Duration::from_millis(500));
        loop {
            ticker.tick().await;  // Wait for the next tick
            let mut app_state_guard = app_state_clone.lock().await;
            let mut terminal_guard = terminal_clone.lock().await;
            if let Err(e) = terminal_guard.draw(|f| draw_ui(f, &mut *app_state_guard)) {
                eprintln!("Error drawing UI: {}", e);
                break;  // Exit the loop if drawing fails
            }
        }
    });

    // Spawn a task to handle input events
    let terminal_clone = Arc::clone(&terminal);
    let handle_event_task = tokio::spawn(async move {
        loop {
            if let Err(e) = handle_event(&app_state, &terminal_clone).await {
                eprintln!("Error handling event: {}", e);
                break;
            }
        }
    });

    // Wait for the handle_event_task to finish, which will happen when Esc is pressed
    handle_event_task.await?;

    // Perform terminal cleanup in the async context
    {
        let mut terminal = terminal.lock().await;
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
    }

    Ok(())
}

async fn handle_event(app_state: &Arc<Mutex<AppState>>, terminal: &Arc<Mutex<Terminal<CrosstermBackend<std::io::Stdout>>>>) -> Result<()> {
    // Poll for events with a short timeout
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            let mut needs_redraw = false;

            match key.code {
                KeyCode::Char(c) => {
                    let mut app_state_guard = app_state.lock().await;
                    app_state_guard.update_input(c);
                    needs_redraw = true;
                }
                KeyCode::Backspace => {
                    let mut app_state_guard = app_state.lock().await;
                    app_state_guard.delete_last_char();
                    needs_redraw = true;
                }
                KeyCode::Tab => {
                    let mut app_state_guard = app_state.lock().await;
                    app_state_guard.switch_block();
                    needs_redraw = true;
                }
                KeyCode::Left => {
                    let mut app_state_guard = app_state.lock().await;
                    app_state_guard.scroll_response_left();
                    needs_redraw = true;
                }
                KeyCode::Right => {
                    let mut app_state_guard = app_state.lock().await;
                    app_state_guard.scroll_response_right();
                    needs_redraw = true;
                }
                KeyCode::Down => {
                    let mut app_state_guard = app_state.lock().await;
                    app_state_guard.next_field();
                    needs_redraw = true;
                }
                KeyCode::Up => {
                    let mut app_state_guard = app_state.lock().await;
                    app_state_guard.previous_field();
                    needs_redraw = true;
                }
                KeyCode::Enter => {
                    let app_state_clone = Arc::clone(&app_state);

                    // Lock the app_state and check the focused field
                    let focused_field = {
                        let state = app_state_clone.lock().await;
                        state.focused_endpoint_field.clone()
                    };

                    // Now move the app_state_clone and start the connection
                    if let Some(EndpointField::ConnectButton) = focused_field {
                        if let Err(e) = connect_and_listen(app_state_clone).await {
                            eprintln!("Error connecting and listening: {}", e);
                        }
                    } else {
                        let mut app_state_guard = app_state.lock().await;
                        if let Err(err) = app_state_guard.handle_enter().await {
                            app_state_guard.json_data = Some(format!("Error: {}", err));
                        }
                    }
                    needs_redraw = true;
                }
                KeyCode::Esc => {
                    // Return an error to indicate that the program should exit
                    return Err(anyhow::anyhow!("Esc pressed, exiting program"));
                }
                _ => {}
            }

            // Redraw TUI after handling key events only if necessary
            if needs_redraw {
                let mut app_state_guard = app_state.lock().await;
                let mut terminal_guard = terminal.lock().await;
                if let Err(e) = terminal_guard.draw(|f| draw_ui(f, &mut *app_state_guard)) {
                    eprintln!("Error drawing UI: {}", e);
                }
            }
        }
    }

    Ok(())
}

async fn connect_and_listen(app_state: Arc<Mutex<AppState>>) -> Result<()> {
    tokio::spawn(async move {
        let (method_id, converted_params, is_stream) = {
            let state = app_state.lock().await;

            // Extract necessary data while holding the lock
            let method_id = state.method_id.ok_or_else(|| anyhow::anyhow!("Method ID is missing"))?;
            let is_stream = state.is_stream;

            let converted_params = state.params.iter().zip(state.param_values.iter())
                .map(|(param, value)| {
                    param.ty.convert_value(value)
                        .context(format!("Failed to convert value for parameter: {}", param.name))
                })
                .collect::<Result<Vec<_>>>()?;

            (method_id, converted_params, is_stream)
        };

        {
            // Send the request to the WebSocket
            let mut state = app_state.lock().await;
            let client = state.client.as_mut().context("WebSocket client is not connected")?;
            client.send_req(method_id, converted_params).await.context("Failed to send request to WebSocket")?;
        }

        // Enter the receiving loop
        loop {
            let raw_response_result = {
                let mut state = app_state.lock().await;
                let client = state.client.as_mut().context("WebSocket client is not connected")?;
                
                client.recv_raw().await
            };

            match raw_response_result {
                Ok(raw_response) => {
                    let resp = {
                        let state = app_state.lock().await;
                        match state.json_view_mode {
                            JsonViewMode::Pretty => serde_json::to_string_pretty(&raw_response).context("Failed to format JSON as pretty"),
                            JsonViewMode::Raw => serde_json::to_string(&raw_response).context("Failed to format JSON as raw"),
                        }
                    };

                    let mut state = app_state.lock().await;
                    match resp {
                        Ok(formatted_json) => {
                            state.json_data = Some(formatted_json);
                            state.endpoint_connected = true;
                        }
                        Err(err) => {
                            state.json_data = Some(format!("Error: {}", err));
                        }
                    }
                }
                Err(e) => {
                    let mut state = app_state.lock().await;
                    state.json_data = Some(format!("Error receiving data: {:?}", e));
                    break;
                }
            }

            if !is_stream {
                // If not streaming, break after receiving one response
                break;
            }

            // Optional: sleep before the next iteration if streaming
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        }

        Ok::<(), anyhow::Error>(())
    });

    Ok(())
}
