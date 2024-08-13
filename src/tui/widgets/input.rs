use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    text::Span,
};

pub fn create_input_widget<'a>(label: &'a str, value: &'a str, is_focused: bool) -> Paragraph<'a> {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if is_focused {
            Color::Yellow // Highlight border when focused
        } else {
            Color::White
        }))
        .title(Span::styled(
            label,
            Style::default().fg(if is_focused {
                Color::White // Highlight title when focused
            } else {
                Color::White
            }),
        ));

    Paragraph::new(value)
        .style(Style::default().fg(Color::White))
        .block(block)
}