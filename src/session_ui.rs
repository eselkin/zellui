use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    widgets::{Block, List, ListItem, Paragraph, Widget},
};

use crate::{App, InputMode};

pub fn render_session_list(app: &App, area: Rect, buf: &mut Buffer) {
    let visible_sessions = app.get_visible_sessions();

    if visible_sessions.is_empty() {
        let no_sessions_text = ratatui::text::Text::from(vec![ratatui::text::Line::from(vec![
            "There are no sessions for this action".into(),
        ])]);

        Paragraph::new(no_sessions_text)
            .centered()
            .block(Block::bordered().title(" Zellij Sessions ".bold()))
            .render(area, buf);
    } else {
        let items: Vec<ListItem> = visible_sessions
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let status = if s.exited { " (EXITED)" } else { "" };
                let content = format!("{}. {} - {}s ago{}", i + 1, s.name, s.time, status);
                if i == app.selected_session {
                    ListItem::new(content).white().bold().on_blue()
                } else {
                    ListItem::new(content)
                }
            })
            .collect();

        let session_title = match app.input_mode {
            InputMode::GoToIndex => format!(" Go to index: {}_ ", app.goto_buffer)
                .yellow()
                .bold(),
            _ => " Zellij Sessions ".bold(),
        };

        let session_list = List::new(items)
            .block(Block::bordered().title(session_title))
            .highlight_symbol(">> ");

        Widget::render(session_list, area, buf);
    }
}
