use std::io;
use std::path::PathBuf;
use std::process::Command;

// helpers from crate
use crate::helpers::{get_zellij_config_dir, list_kdl_files, parse_time};
use color_eyre::eyre::Result;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::DefaultTerminal;

pub mod helpers;
pub mod layout_ui;
pub mod session_ui;
pub mod ui;

#[cfg(test)]
mod tests;

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut app = App::default();
    let result = ratatui::run(|terminal| app.run(terminal))?;
    if let Some(args) = result {
        Command::new("zellij").args(args).status()?;
    }
    Ok(())
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Action {
    pub code: u8,
    pub shortcut: char,
    pub title: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub name: String,
    pub time: i64,
    pub exited: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KdlFile {
    pub name: String,
    pub path: PathBuf,
}

impl PartialEq<char> for Action {
    fn eq(&self, other: &char) -> bool {
        self.shortcut == *other
    }
}

impl PartialEq<Action> for char {
    fn eq(&self, other: &Action) -> bool {
        *self == other.shortcut
    }
}

impl PartialEq for Action {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
    }
}

pub const ACTIONS: [Action; 4] = [
    Action {
        code: 0,
        shortcut: 'a',
        title: "Attach",
    },
    Action {
        code: 1,
        shortcut: 'k',
        title: "Kill",
    },
    Action {
        code: 2,
        shortcut: 'd',
        title: "Delete",
    },
    Action {
        code: 3,
        shortcut: 'n',
        title: "New Session",
    },
];

#[derive(Debug, Default, PartialEq, Eq)]
pub enum InputMode {
    #[default]
    Normal,
    GoToIndex,
    Status(String),
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum ListType {
    #[default]
    Layout,
    Config,
    Session,
}

#[derive(Debug)]
pub struct App {
    pub action: Action,
    pub sessions: Vec<Session>,
    pub layouts: Vec<KdlFile>,
    pub configs: Vec<KdlFile>,
    pub selected_session: usize,
    pub selected_layout: usize,
    pub selected_config: usize,
    pub focused_list: ListType,
    pub input_mode: InputMode,
    pub goto_buffer: String,
    pub status_message: String,
    pub exit: bool,
}

impl Default for App {
    fn default() -> Self {
        let mut app = Self {
            action: ACTIONS[0],
            sessions: Vec::new(),
            layouts: Vec::new(),
            configs: Vec::new(),
            selected_session: 0,
            selected_layout: 0,
            selected_config: 0,
            focused_list: ListType::Layout,
            input_mode: InputMode::Normal,
            goto_buffer: String::new(),
            status_message: String::new(),
            exit: false,
        };
        app.refresh();
        app
    }
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<Option<Vec<String>>> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            if let Some(res) = self.handle_events()? {
                return Ok(Some(res));
            }
        }
        Ok(None)
    }

    fn draw(&self, frame: &mut ratatui::Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<Option<Vec<String>>> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                Ok(self.handle_key_event(key_event))
            }
            _ => Ok(None),
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Option<Vec<String>> {
        match self.input_mode {
            InputMode::Normal => match key_event.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    self.exit();
                    None
                }
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    self.refresh();
                    None
                }
                KeyCode::Char('g') | KeyCode::Char('G') => {
                    self.input_mode = InputMode::GoToIndex;
                    self.goto_buffer.clear();
                    None
                }
                KeyCode::Tab => {
                    self.next_action();
                    None
                }
                KeyCode::BackTab => {
                    self.previous_action();
                    None
                }
                KeyCode::Right => {
                    self.focused_list = match self.focused_list {
                        ListType::Layout => ListType::Config,
                        ListType::Config => ListType::Layout,
                        _ => self.focused_list,
                    };
                    None
                }
                KeyCode::Left => {
                    self.focused_list = match self.focused_list {
                        ListType::Layout => ListType::Config,
                        ListType::Config => ListType::Layout,
                        _ => self.focused_list,
                    };
                    None
                }
                KeyCode::Up => {
                    self.previous_item();
                    None
                }
                KeyCode::Down => {
                    self.next_item();
                    None
                }
                KeyCode::PageUp => {
                    self.page_up();
                    None
                }
                KeyCode::PageDown => {
                    self.page_down();
                    None
                }
                KeyCode::Enter => self.trigger_action(),
                KeyCode::Char(c) if ACTIONS.iter().any(|&a| a == c.to_ascii_lowercase()) => {
                    self.do_action(key_event.code);
                    None
                }
                _ => None,
            },
            InputMode::GoToIndex => match key_event.code {
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    self.goto_buffer.push(c);
                    None
                }
                KeyCode::Backspace => {
                    self.goto_buffer.pop();
                    None
                }
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    None
                }
                KeyCode::Enter => {
                    if let Ok(index) = self.goto_buffer.parse::<usize>() {
                        if index > 0 {
                            if self.action.title == "New Session" {
                                let list_len = match self.focused_list {
                                    ListType::Layout => self.layouts.len(),
                                    ListType::Config => self.configs.len(),
                                    _ => 0,
                                };
                                let selected = (index - 1).min(list_len.saturating_sub(1));
                                match self.focused_list {
                                    ListType::Layout => self.selected_layout = selected,
                                    ListType::Config => self.selected_config = selected,
                                    _ => {}
                                }
                            } else {
                                let visible_count = self.get_visible_sessions().len();
                                self.selected_session =
                                    (index - 1).min(visible_count.saturating_sub(1));
                            }
                        }
                    }
                    self.input_mode = InputMode::Normal;
                    None
                }
                _ => None,
            },
            InputMode::Status(_) => {
                self.input_mode = InputMode::Normal;
                None
            }
        }
    }

    fn trigger_action(&mut self) -> Option<Vec<String>> {
        if self.action.title == "New Session" {
            let mut args = Vec::new();
            if !self.layouts.is_empty() {
                let layout_path = self.layouts[self.selected_layout]
                    .path
                    .to_string_lossy()
                    .to_string();
                args.push("-l".to_string());
                args.push(layout_path);
            }
            if !self.configs.is_empty() {
                let config_path = self.configs[self.selected_config]
                    .path
                    .to_string_lossy()
                    .to_string();
                args.push("-c".to_string());
                args.push(config_path);
            }
            self.exit = true;
            Some(args)
        } else {
            let session_name = {
                let visible_sessions = self.get_visible_sessions();
                visible_sessions
                    .get(self.selected_session)
                    .map(|s| s.name.clone())
            };

            if let Some(name) = session_name {
                match self.action.title {
                    "Kill" => {
                        let output = Command::new("zellij")
                            .args(["kill-session", &name])
                            .output();

                        match output {
                            Ok(result) => {
                                let msg = if result.status.success() {
                                    self.sessions.retain(|s| s.name != name);
                                    self.clamp_selection();
                                    String::from_utf8_lossy(&result.stdout).trim().to_string()
                                } else {
                                    String::from_utf8_lossy(&result.stderr).trim().to_string()
                                };
                                if msg.is_empty() {
                                    self.status_message = format!("Killed session: {}", name);
                                } else {
                                    self.status_message = msg;
                                }
                            }
                            Err(e) => {
                                self.status_message = format!("Error: {}", e);
                            }
                        }
                        self.input_mode = InputMode::Status(self.status_message.clone());
                        self.fetch_sessions();
                        None
                    }
                    "Attach" => {
                        self.exit = true;
                        Some(vec!["attach".to_string(), name])
                    }
                    "Delete" => {
                        let output = Command::new("zellij")
                            .args(["delete-session", &name])
                            .output();

                        match output {
                            Ok(result) => {
                                let msg = if result.status.success() {
                                    self.sessions.retain(|s| s.name != name);
                                    self.clamp_selection();
                                    String::from_utf8_lossy(&result.stdout).trim().to_string()
                                } else {
                                    String::from_utf8_lossy(&result.stderr).trim().to_string()
                                };
                                if msg.is_empty() {
                                    self.status_message = format!("Deleted session: {}", name);
                                } else {
                                    self.status_message = msg;
                                }
                            }
                            Err(e) => {
                                self.status_message = format!("Error: {}", e);
                            }
                        }
                        self.input_mode = InputMode::Status(self.status_message.clone());
                        self.fetch_sessions();
                        None
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn refresh(&mut self) {
        self.fetch_sessions();
        self.fetch_files();
    }

    fn fetch_sessions(&mut self) {
        let output = Command::new("zellij").args(["ls", "-n"]).output();

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                self.sessions = stdout
                    .lines()
                    .filter(|line| !line.is_empty())
                    .map(|line| {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        let name = parts.get(0).unwrap_or(&"").to_string();
                        let exited = line.contains("EXITED");

                        let time_str = if let Some(start) = line.find("[Created ") {
                            if let Some(end) = line.find(" ago]") {
                                &line[start + 9..end]
                            } else {
                                ""
                            }
                        } else {
                            ""
                        };

                        let time = parse_time(time_str);

                        Session { name, time, exited }
                    })
                    .collect();

                self.clamp_selection();
            } else {
                self.sessions.clear();
            }
        } else {
            self.sessions.clear();
        }
    }

    fn fetch_files(&mut self) {
        if let Some(config_dir) = get_zellij_config_dir() {
            // Layouts
            let mut layout_dir = config_dir.clone();
            layout_dir.push("layouts");
            let layout_filenames = list_kdl_files(&layout_dir);
            self.layouts = layout_filenames
                .into_iter()
                .map(|name| {
                    let mut path = layout_dir.clone();
                    path.push(&name);
                    KdlFile { name, path }
                })
                .collect();

            // Configs
            let config_filenames = list_kdl_files(&config_dir);
            self.configs = config_filenames
                .into_iter()
                .map(|name| {
                    let mut path = config_dir.clone();
                    path.push(&name);
                    KdlFile { name, path }
                })
                .collect();

            // Select default.kdl if it exists
            if let Some(idx) = self.layouts.iter().position(|f| f.name == "default.kdl") {
                self.selected_layout = idx;
            }
            if let Some(idx) = self.configs.iter().position(|f| f.name == "default.kdl") {
                self.selected_config = idx;
            }

            self.clamp_selection();
        }
    }

    pub(crate) fn get_visible_sessions(&self) -> Vec<&Session> {
        match self.action.title {
            "Attach" | "Delete" => self.sessions.iter().filter(|s| s.exited).collect(),
            "Kill" => self.sessions.iter().filter(|s| !s.exited).collect(),
            _ => self.sessions.iter().collect(),
        }
    }

    fn clamp_selection(&mut self) {
        let visible_count = self.get_visible_sessions().len();
        if visible_count == 0 {
            self.selected_session = 0;
        } else if self.selected_session >= visible_count {
            self.selected_session = visible_count.saturating_sub(1);
        }

        if self.layouts.is_empty() {
            self.selected_layout = 0;
        } else if self.selected_layout >= self.layouts.len() {
            self.selected_layout = self.layouts.len() - 1;
        }

        if self.configs.is_empty() {
            self.selected_config = 0;
        } else if self.selected_config >= self.configs.len() {
            self.selected_config = self.configs.len() - 1;
        }
    }

    fn previous_item(&mut self) {
        if self.action.title == "New Session" {
            match self.focused_list {
                ListType::Layout => {
                    if self.layouts.is_empty() {
                        return;
                    }
                    self.selected_layout =
                        (self.selected_layout + self.layouts.len() - 1) % self.layouts.len();
                }
                ListType::Config => {
                    if self.configs.is_empty() {
                        return;
                    }
                    self.selected_config =
                        (self.selected_config + self.configs.len() - 1) % self.configs.len();
                }
                ListType::Session => {}
            }
        } else {
            let visible_count = self.get_visible_sessions().len();
            if visible_count == 0 {
                return;
            }
            self.selected_session = (self.selected_session + visible_count - 1) % visible_count;
        }
    }

    fn next_item(&mut self) {
        if self.action.title == "New Session" {
            match self.focused_list {
                ListType::Layout => {
                    if self.layouts.is_empty() {
                        return;
                    }
                    self.selected_layout = (self.selected_layout + 1) % self.layouts.len();
                }
                ListType::Config => {
                    if self.configs.is_empty() {
                        return;
                    }
                    self.selected_config = (self.selected_config + 1) % self.configs.len();
                }
                ListType::Session => {}
            }
        } else {
            let visible_count = self.get_visible_sessions().len();
            if visible_count == 0 {
                return;
            }
            self.selected_session = (self.selected_session + 1) % visible_count;
        }
    }

    fn page_up(&mut self) {
        if self.action.title == "New Session" {
            // Optional: page navigation for files
        } else {
            let visible_count = self.get_visible_sessions().len();
            if visible_count == 0 {
                return;
            }
            self.selected_session = self.selected_session.saturating_sub(10);
        }
    }

    fn page_down(&mut self) {
        if self.action.title == "New Session" {
            // Optional: page navigation for files
        } else {
            let visible_count = self.get_visible_sessions().len();
            if visible_count == 0 {
                return;
            }
            self.selected_session =
                (self.selected_session + 10).min(visible_count.saturating_sub(1));
        }
    }

    fn previous_action(&mut self) {
        let current_index = ACTIONS.iter().position(|&a| a == self.action).unwrap_or(0);
        let new_index = (current_index + ACTIONS.len() - 1) % ACTIONS.len();
        self.action = ACTIONS[new_index];
        if self.action.title == "New Session" {
            self.focused_list = ListType::Layout;
        } else {
            self.focused_list = ListType::Session;
        }
        self.clamp_selection();
    }

    fn next_action(&mut self) {
        let current_index = ACTIONS.iter().position(|&a| a == self.action).unwrap_or(0);
        let new_index = (current_index + 1) % ACTIONS.len();
        self.action = ACTIONS[new_index];
        if self.action.title == "New Session" {
            self.focused_list = ListType::Layout;
        } else {
            self.focused_list = ListType::Session;
        }
        self.clamp_selection();
    }

    fn do_action(&mut self, code: KeyCode) {
        if let KeyCode::Char(c) = code {
            if let Some(action) = ACTIONS
                .iter()
                .find(|a| a.shortcut == c.to_ascii_lowercase())
            {
                self.action = *action;
                self.clamp_selection();
            }
        }
    }
}
