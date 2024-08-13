use ratatui::{
    widgets::{Block, Borders, Tabs},
    style::{Style, Color, Modifier},
    text::Spans,
};
use crate::tui::state::AppScreen;

pub fn create_tabs<'a>(titles: &[&'a str], selected: &AppScreen) -> Tabs<'a> {
    let titles = titles.iter().map(|t| Spans::from(*t)).collect::<Vec<_>>();

    Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Gray))
                .title(" Menu ")
                .title_style(Style::default().fg(Color::Gray))
        )
        .style(Style::default().fg(Color::Gray))
        .select(match selected {
            AppScreen::Settings => 0,
            AppScreen::Endpoints => 1,
        })
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
}
