use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    text::Span,
};

pub fn create_error_message<'a>(message: &'a str) -> Paragraph<'a> {
    Paragraph::new(Span::styled(message, Style::default().fg(Color::Red)))
        .block(Block::default().borders(Borders::ALL).title("Error"))
}
