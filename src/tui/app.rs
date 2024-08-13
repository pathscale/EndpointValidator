use crate::tui::state::{AppState, AppScreen};
use crate::tui::ui::draw_ui;
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen, EnterAlternateScreen};
use crossterm::execute;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;

pub async fn run() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app_state = AppState::new();

    loop {
        terminal.draw(|f| draw_ui(f, &mut app_state))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char(c) => app_state.update_input(c),
                KeyCode::Backspace => app_state.delete_last_char(),
                KeyCode::Down => {
                    if app_state.current_screen == AppScreen::Endpoints {
                        if app_state.in_left_chunk {
                            app_state.select_next_endpoint();
                        } else {
                            app_state.next_field();
                        }
                    } else {
                        app_state.next_field();
                    }
                }
                KeyCode::Up => {
                    if app_state.current_screen == AppScreen::Endpoints {
                        if app_state.in_left_chunk {
                            app_state.select_previous_endpoint();
                        } else {
                            app_state.previous_field();
                        }
                    } else {
                        app_state.previous_field();
                    }
                }
                KeyCode::Right => app_state.move_focus_right(),
                KeyCode::Left => app_state.move_focus_left(),
                KeyCode::Enter => app_state.handle_enter().await,
                KeyCode::Tab => app_state.switch_screen(),
                KeyCode::Esc => break,
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,  // Exit alternate screen buffer
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
