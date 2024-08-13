use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem},
    text::Span,
};

pub fn create_list_widget<'a>(items: &'a [String], selected: usize, is_focused: bool) -> List<'a> {
    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == selected {
                Style::default().bg(Color::Blue).fg(Color::Gray)
            } else {
                Style::default().fg(Color::Gray)
            };
            ListItem::new(Span::styled(item.clone(), style))
        })
        .collect();

    let border_color = if is_focused {
        Color::Yellow
    } else {
        Color::Gray
    };

    List::new(list_items)
        .block(Block::default().borders(Borders::ALL).title(" Endpoints ").border_style(Style::default().fg(border_color)))
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::Gray))
}