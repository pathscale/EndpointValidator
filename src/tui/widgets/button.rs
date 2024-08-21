use ratatui::{
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    text::Span,
};

pub fn create_button<'a>(label: &'a str, is_active: bool, is_focused: bool) -> Paragraph<'a> {
    let style = if is_active {
        Style::default()
            .fg(Color::Gray)
            .bg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Gray)
            .bg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if is_focused {
            Color::Yellow
        } else {
            Color::Gray
        }));

    Paragraph::new(Span::styled(label, style))
        .alignment(ratatui::layout::Alignment::Center)
        .block(block)
}
