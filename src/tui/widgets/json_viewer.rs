use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

pub fn create_json_viewer<'a>(json_data: &'a Option<String>) -> Paragraph<'a> {
    let json_display = json_data.as_deref().unwrap_or("{}");

    Paragraph::new(json_display)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Response ")
                .title_style(Style::default().fg(Color::White)),
        )
}
