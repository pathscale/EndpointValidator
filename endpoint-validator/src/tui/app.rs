use crate::parser::EndpointMetadata;
use crate::tui::state::AppState;
use crate::tui::ui::draw_ui;
use crate::ws::Handle;
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use eyre::Result;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::collections::HashMap;
use std::io;
use std::time::Duration;

/// Run the TUI until Esc.
///
/// One task on the caller's reactor. Between keys it listens on the open
/// connection, so stream updates show as they arrive. A request waits for the
/// frame answering it, up to [`crate::ws::RECV_TIMEOUT`].
pub async fn run(
    endpoint_names: Vec<String>,
    endpoint_data: HashMap<String, EndpointMetadata>,
    param_defaults: HashMap<String, HashMap<String, String>>,
    handle: Handle,
) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let mut state = AppState::new(endpoint_names, endpoint_data, param_defaults, handle);
    let result = event_loop(&mut terminal, &mut state).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    result
}

async fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &mut AppState,
) -> Result<()> {
    loop {
        terminal.draw(|f| draw_ui(f, state))?;
        if !event::poll(Duration::from_millis(50))? {
            if let Err(err) = state.poll_frame(Duration::from_millis(50)).await {
                state.json_data = Some(format!("Error: {err:#}"));
                state.client = None;
                state.connected = false;
            }
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        match key.code {
            KeyCode::Char(c) => state.update_input(c),
            KeyCode::Backspace => state.delete_last_char(),
            KeyCode::Tab => state.switch_block(),
            KeyCode::Left => state.scroll_response_left(),
            KeyCode::Right => state.scroll_response_right(),
            KeyCode::Down => state.next_field(),
            KeyCode::Up => state.previous_field(),
            KeyCode::Enter => {
                if let Err(err) = state.handle_enter().await {
                    state.json_data = Some(format!("Error: {err:#}"));
                }
            }
            KeyCode::Esc => return Ok(()),
            _ => {}
        }
    }
}
