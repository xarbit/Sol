use cosmic::iced::keyboard::Key;
use cosmic::widget::menu;
use std::collections::HashMap;
use std::sync::OnceLock;

use crate::menu_action::MenuAction;

/// Global keyboard shortcuts registry
static KEY_BINDS: OnceLock<HashMap<menu::KeyBind, MenuAction>> = OnceLock::new();

/// Initialize the global keyboard shortcuts
pub fn init_key_binds() -> HashMap<menu::KeyBind, MenuAction> {
    let mut key_binds = HashMap::new();

    // New Event: Ctrl+N
    key_binds.insert(
        menu::KeyBind {
            modifiers: vec![menu::key_bind::Modifier::Ctrl],
            key: Key::Character("n".into()),
        },
        MenuAction::NewEvent,
    );

    // Month View: Ctrl+1
    key_binds.insert(
        menu::KeyBind {
            modifiers: vec![menu::key_bind::Modifier::Ctrl],
            key: Key::Character("1".into()),
        },
        MenuAction::ViewMonth,
    );

    // Week View: Ctrl+2
    key_binds.insert(
        menu::KeyBind {
            modifiers: vec![menu::key_bind::Modifier::Ctrl],
            key: Key::Character("2".into()),
        },
        MenuAction::ViewWeek,
    );

    // Day View: Ctrl+3
    key_binds.insert(
        menu::KeyBind {
            modifiers: vec![menu::key_bind::Modifier::Ctrl],
            key: Key::Character("3".into()),
        },
        MenuAction::ViewDay,
    );

    // Year View: Ctrl+4
    key_binds.insert(
        menu::KeyBind {
            modifiers: vec![menu::key_bind::Modifier::Ctrl],
            key: Key::Character("4".into()),
        },
        MenuAction::ViewYear,
    );

    // Store globally for subscription access
    let _ = KEY_BINDS.set(key_binds.clone());

    key_binds
}

/// Get the global keyboard shortcuts
pub fn get_key_binds() -> &'static HashMap<menu::KeyBind, MenuAction> {
    KEY_BINDS.get().expect("KEY_BINDS not initialized")
}
