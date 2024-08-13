use ratatui::{
    style::{Color, Style, Modifier},
    widgets::{Block, Borders, List, ListItem},
    text::Span,
};

pub fn create_list_widget<'a>(
    items: &'a [String],
    selected: usize,
    is_focused: bool,
) -> List<'a> {
    let visible_items = &items[selected as usize..];
    let list_items: Vec<ListItem> = visible_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == 0 {
                Style::default().bg(Color::Blue).fg(Color::Gray)
            } else {
                Style::default().fg(Color::Gray)
            };
            ListItem::new(Span::styled(item.clone(), style))
        })
        .collect();

    let list = List::new(list_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if is_focused { Color::Yellow } else { Color::Gray }))
            .title("Endpoints")
            .title_style(Style::default().fg(if is_focused { Color::Yellow } else { Color::Gray })))
        .highlight_style(Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD));

    list
}
