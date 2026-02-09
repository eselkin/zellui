use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

use crate::{ACTIONS, App, InputMode, layout_ui, session_ui};

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let title = Line::from(" Zellij Action TUI ".bold());
        let mut list_keys = if area.width < 172 {
            vec![
                "Nav: ".into(),
                "<Tab>".blue().bold(),
                "<L/R>".blue().bold(),
                "<Q> ".blue().bold(),
                " Actions: ".into(),
            ]
        } else {
            vec![
                " Switch Action ".into(),
                "<Tab>".blue().bold(),
                " Switch List ".into(),
                "<Left/Right>".blue().bold(),
                " Select Item ".into(),
                "<Up/Down>".blue().bold(),
                " Go to ".into(),
                "<G>".blue().bold(),
                " Execute ".into(),
                "<Enter>".blue().bold(),
                " Refresh ".into(),
                "<R>".blue().bold(),
                " Quit ".into(),
                "<Q> ".blue().bold(),
            ]
        };

        for action in ACTIONS {
            if area.width >= 172 {
                list_keys.push(format!(" {} ", action.title).into());
            }
            list_keys.push(
                format!("<{}>", action.shortcut.to_uppercase())
                    .blue()
                    .bold(),
            );
        }

        let instructions = Line::from(list_keys);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let main_chunks = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(2),
            ])
            .split(area);

        let action_text = ratatui::text::Text::from(vec![Line::from(vec![
            "Current Action: ".into(),
            self.action.title.yellow().bold(),
        ])]);

        Paragraph::new(action_text)
            .centered()
            .block(block)
            .render(main_chunks[0], buf);

        if self.action.title == "New Session" {
            layout_ui::render_layout_list(self, main_chunks[1], buf);
        } else {
            session_ui::render_session_list(self, main_chunks[1], buf);
        }

        // Status Area
        let status_text = match self.input_mode {
            InputMode::Status(ref msg) => {
                ratatui::text::Text::from(vec![Line::from(vec![msg.clone().green()])]).bold()
            }
            _ => ratatui::text::Text::from(vec![]),
        };

        Paragraph::new(status_text)
            .centered()
            .render(main_chunks[2], buf);
    }
}
