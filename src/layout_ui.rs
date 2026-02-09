use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    widgets::{Block, List, ListItem, Widget},
};

use crate::{App, InputMode, ListType};

pub fn render_layout_list(app: &App, area: Rect, buf: &mut Buffer) {
    let list_chunks = Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Layouts List
    let layout_items: Vec<ListItem> = app
        .layouts
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let content = format!("{}. {}", i + 1, f.name);
            if i == app.selected_layout {
                let mut style = ratatui::style::Style::default().white().bold();
                if app.focused_list == ListType::Layout {
                    style = style.on_blue();
                } else {
                    style = style.on_black();
                }
                ListItem::new(content).style(style)
            } else {
                ListItem::new(content)
            }
        })
        .collect();

    let layout_title =
        if app.focused_list == ListType::Layout && app.input_mode == InputMode::GoToIndex {
            format!(" Go to index: {}_ ", app.goto_buffer)
                .yellow()
                .bold()
        } else {
            " Zellij Layouts ".bold()
        };

    let mut layout_block = Block::bordered().title(layout_title);
    if app.focused_list == ListType::Layout {
        layout_block = layout_block.border_style(ratatui::style::Style::default().yellow());
    }

    let layout_list = List::new(layout_items)
        .block(layout_block)
        .highlight_symbol(">> ");

    Widget::render(layout_list, list_chunks[0], buf);

    // Configs List
    let config_items: Vec<ListItem> = app
        .configs
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let content = format!("{}. {}", i + 1, f.name);
            if i == app.selected_config {
                let mut style = ratatui::style::Style::default().white().bold();
                if app.focused_list == ListType::Config {
                    style = style.on_blue();
                } else {
                    style = style.on_black();
                }
                ListItem::new(content).style(style)
            } else {
                ListItem::new(content)
            }
        })
        .collect();

    let config_title =
        if app.focused_list == ListType::Config && app.input_mode == InputMode::GoToIndex {
            format!(" Go to index: {}_ ", app.goto_buffer)
                .yellow()
                .bold()
        } else {
            " Zellij Configs ".bold()
        };

    let mut config_block = Block::bordered().title(config_title);
    if app.focused_list == ListType::Config {
        config_block = config_block.border_style(ratatui::style::Style::default().yellow());
    }

    let config_list = List::new(config_items)
        .block(config_block)
        .highlight_symbol(">> ");

    Widget::render(config_list, list_chunks[1], buf);
}
