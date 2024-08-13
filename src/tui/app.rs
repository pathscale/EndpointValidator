use crate::tui::state::{AppState, AppBlock, EndpointField, SettingsField};
use crate::tui::ui::draw_ui;
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use anyhow::Result;  // Import anyhow::Result for better error handling

pub async fn run() -> Result<()> {  // Ensure the return type is anyhow::Result
    // Set up terminal in raw mode
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize app state
    let mut app_state = AppState::new();

    // Main event loop
    loop {
        terminal.draw(|f| draw_ui(f, &mut app_state))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char(c) => app_state.update_input(c), // Handle character input
                KeyCode::Backspace => app_state.delete_last_char(), // Handle backspace
                KeyCode::Tab => app_state.switch_block(), // Switch between blocks (e.g., settings and endpoints)
                KeyCode::Down => app_state.next_field(), // Move to the next input field
                KeyCode::Up => app_state.previous_field(), // Move to the previous input field
                KeyCode::Enter => app_state.handle_enter().await?, // Handle the Enter key for actions like connect/disconnect
                KeyCode::Esc => break, // Exit the application
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
