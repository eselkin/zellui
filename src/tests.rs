use super::*;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

#[test]
fn test_parse_time() {
    assert_eq!(parse_time("1s"), 1);
    assert_eq!(parse_time("1m"), 60);
    assert_eq!(parse_time("1h"), 3600);
    assert_eq!(parse_time("1day"), 86400);
    assert_eq!(parse_time("2days"), 172800);
    assert_eq!(parse_time("1day 2h 3m 4s"), 86400 + 7200 + 180 + 4);
    assert_eq!(parse_time(""), 0);
}

#[test]
fn test_session_navigation() {
    let mut app = App {
        action: ACTIONS[0], // Attach
        sessions: vec![
            Session {
                name: "s1".to_string(),
                time: 0,
                exited: true,
            },
            Session {
                name: "s2".to_string(),
                time: 0,
                exited: true,
            },
        ],
        layouts: Vec::new(),
        configs: Vec::new(),
        selected_session: 0,
        selected_layout: 0,
        selected_config: 0,
        focused_list: ListType::Session, // Changed to Session
        input_mode: InputMode::Normal,
        goto_buffer: String::new(),
        status_message: String::new(),
        exit: false,
    };

    app.next_item();
    assert_eq!(app.selected_session, 1);
    app.next_item();
    assert_eq!(app.selected_session, 0); // Wrap around

    app.previous_item();
    assert_eq!(app.selected_session, 1); // Wrap around
}

#[test]
fn test_file_navigation() {
    let mut app = App {
        action: ACTIONS[3], // New Session
        sessions: Vec::new(),
        layouts: vec![
            KdlFile {
                name: "l1.kdl".to_string(),
                path: PathBuf::from("l1.kdl"),
            },
            KdlFile {
                name: "l2.kdl".to_string(),
                path: PathBuf::from("l2.kdl"),
            },
        ],
        configs: vec![KdlFile {
            name: "c1.kdl".to_string(),
            path: PathBuf::from("c1.kdl"),
        }],
        selected_session: 0,
        selected_layout: 0,
        selected_config: 0,
        focused_list: ListType::Layout,
        input_mode: InputMode::Normal,
        goto_buffer: String::new(),
        status_message: String::new(),
        exit: false,
    };

    app.next_item();
    assert_eq!(app.selected_layout, 1);

    // Switch focus
    app.focused_list = ListType::Config;
    app.next_item();
    assert_eq!(app.selected_config, 0); // Only one item
}

#[test]
fn test_goto_index_session() {
    let mut app = App {
        action: ACTIONS[0],
        sessions: (0..20)
            .map(|i| Session {
                name: format!("s{}", i),
                time: 0,
                exited: true,
            })
            .collect(),
        layouts: Vec::new(),
        configs: Vec::new(),
        selected_session: 0,
        selected_layout: 0,
        selected_config: 0,
        focused_list: ListType::Session, // Changed to Session
        input_mode: InputMode::Normal,
        goto_buffer: String::new(),
        status_message: String::new(),
        exit: false,
    };

    app.handle_key_event(KeyEvent::new(KeyCode::Char('g'), event::KeyModifiers::NONE));
    app.handle_key_event(KeyEvent::new(KeyCode::Char('1'), event::KeyModifiers::NONE));
    app.handle_key_event(KeyEvent::new(KeyCode::Char('5'), event::KeyModifiers::NONE));
    app.handle_key_event(KeyEvent::new(KeyCode::Enter, event::KeyModifiers::NONE));
    assert_eq!(app.selected_session, 14);
}

#[test]
fn test_render_no_panic() {
    let mut app = App::default();
    app.sessions = vec![Session {
        name: "test".to_string(),
        time: 0,
        exited: true,
    }];
    let mut buf = Buffer::empty(Rect::new(0, 0, 173, 50));
    app.render(buf.area, &mut buf);

    // Test small width
    let mut buf_small = Buffer::empty(Rect::new(0, 0, 100, 50));
    app.render(buf_small.area, &mut buf_small);
}

#[test]
fn test_clamp_selection_empty() {
    let mut app = App::default();
    app.sessions = vec![Session {
        name: "s1".to_string(),
        time: 0,
        exited: true,
    }];
    app.selected_session = 0;

    // Simulate deletion
    app.sessions.clear();
    app.clamp_selection();
    assert_eq!(app.selected_session, 0);
}
