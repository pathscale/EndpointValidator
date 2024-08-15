use crate::tui::state::{AppState};
use crate::tui::ui::draw_ui;
use crate::parser::{EndpointMetadata};
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use anyhow::Result;
use std::collections::HashMap;

pub async fn run(endpoint_names: Vec<String>, endpoint_data: HashMap<String, EndpointMetadata>, param_defaults: Vec<(String, Vec<(String, String)>)>) -> Result<()> {  // Ensure the return type is anyhow::Result
    // Set up terminal in raw mode
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize app state
    let mut app_state = AppState::new(endpoint_names, endpoint_data, param_defaults);

    // Main event loop
    loop {
        terminal.draw(|f| draw_ui(f, &mut app_state))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char(c) => app_state.update_input(c),
                KeyCode::Backspace => app_state.delete_last_char(),
                KeyCode::Tab => app_state.switch_block(),
                KeyCode::Left => app_state.scroll_response_left(),
                KeyCode::Right => app_state.scroll_response_right(),
                KeyCode::Down => app_state.next_field(),
                KeyCode::Up => app_state.previous_field(),
                KeyCode::Enter => app_state.handle_enter().await?,
                KeyCode::Esc => break,
                _ => {}
            }
        }
    }

    // Clean up terminal before exiting
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
