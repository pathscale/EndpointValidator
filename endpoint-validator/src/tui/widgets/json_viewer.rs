use ratatui::{
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
};

pub fn create_json_viewer<'a>(json_data: &'a Option<String>, is_focused: bool) -> Paragraph<'a> {
    let json_display = json_data.as_deref().unwrap_or("");

    let title = Spans::from(vec![Span::styled(
        " Response ",
        Style::default().fg(if is_focused { Color::Yellow } else { Color::Gray }),
    )]);

    Paragraph::new(json_display)
        .style(Style::default().fg(Color::Gray))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if is_focused { Color::Yellow } else { Color::Gray }))
                .title(title),
        )
}
