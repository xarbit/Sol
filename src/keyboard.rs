use cosmic::iced::keyboard::key::Named;
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

    // New Event: Ctrl+Shift+N
    key_binds.insert(
        menu::KeyBind {
            modifiers: vec![menu::key_bind::Modifier::Ctrl, menu::key_bind::Modifier::Shift],
            key: Key::Character("n".into()),
        },
        MenuAction::NewEvent,
    );

    // Today: Ctrl+Shift+T
    key_binds.insert(
        menu::KeyBind {
            modifiers: vec![menu::key_bind::Modifier::Ctrl, menu::key_bind::Modifier::Shift],
            key: Key::Character("t".into()),
        },
        MenuAction::Today,
    );

    // Day View: Ctrl+Shift+D
    key_binds.insert(
        menu::KeyBind {
            modifiers: vec![menu::key_bind::Modifier::Ctrl, menu::key_bind::Modifier::Shift],
            key: Key::Character("d".into()),
        },
        MenuAction::ViewDay,
    );

    // Week View: Ctrl+Shift+W
    key_binds.insert(
        menu::KeyBind {
            modifiers: vec![menu::key_bind::Modifier::Ctrl, menu::key_bind::Modifier::Shift],
            key: Key::Character("w".into()),
        },
        MenuAction::ViewWeek,
    );

    // Month View: Ctrl+Shift+M
    key_binds.insert(
        menu::KeyBind {
            modifiers: vec![menu::key_bind::Modifier::Ctrl, menu::key_bind::Modifier::Shift],
            key: Key::Character("m".into()),
        },
        MenuAction::ViewMonth,
    );

    // Year View: Ctrl+Shift+Y
    key_binds.insert(
        menu::KeyBind {
            modifiers: vec![menu::key_bind::Modifier::Ctrl, menu::key_bind::Modifier::Shift],
            key: Key::Character("y".into()),
        },
        MenuAction::ViewYear,
    );

    // Navigate Previous Period: Ctrl+Shift+Left (prev month/week/day depending on view)
    key_binds.insert(
        menu::KeyBind {
            modifiers: vec![menu::key_bind::Modifier::Ctrl, menu::key_bind::Modifier::Shift],
            key: Key::Named(Named::ArrowLeft),
        },
        MenuAction::NavigatePrevious,
    );

    // Navigate Next Period: Ctrl+Shift+Right (next month/week/day depending on view)
    key_binds.insert(
        menu::KeyBind {
            modifiers: vec![menu::key_bind::Modifier::Ctrl, menu::key_bind::Modifier::Shift],
            key: Key::Named(Named::ArrowRight),
        },
        MenuAction::NavigateNext,
    );

    // Scroll Timeline Up: Ctrl+Shift+Up (scroll up 1 hour in Day/Week view)
    key_binds.insert(
        menu::KeyBind {
            modifiers: vec![menu::key_bind::Modifier::Ctrl, menu::key_bind::Modifier::Shift],
            key: Key::Named(Named::ArrowUp),
        },
        MenuAction::ScrollTimelineUp,
    );

    // Scroll Timeline Down: Ctrl+Shift+Down (scroll down 1 hour in Day/Week view)
    key_binds.insert(
        menu::KeyBind {
            modifiers: vec![menu::key_bind::Modifier::Ctrl, menu::key_bind::Modifier::Shift],
            key: Key::Named(Named::ArrowDown),
        },
        MenuAction::ScrollTimelineDown,
    );

    // Delete Selected Event: Delete key (no modifiers)
    key_binds.insert(
        menu::KeyBind {
            modifiers: vec![],
            key: Key::Named(Named::Delete),
        },
        MenuAction::DeleteSelectedEvent,
    );

    // Cycle View Next: Ctrl+Shift+] (Year → Month → Week → Day)
    key_binds.insert(
        menu::KeyBind {
            modifiers: vec![menu::key_bind::Modifier::Ctrl, menu::key_bind::Modifier::Shift],
            key: Key::Character("]".into()),
        },
        MenuAction::CycleViewNext,
    );

    // Cycle View Previous: Ctrl+Shift+[ (Day → Week → Month → Year)
    key_binds.insert(
        menu::KeyBind {
            modifiers: vec![menu::key_bind::Modifier::Ctrl, menu::key_bind::Modifier::Shift],
            key: Key::Character("[".into()),
        },
        MenuAction::CycleViewPrevious,
    );

    // Store globally for subscription access
    let _ = KEY_BINDS.set(key_binds.clone());

    key_binds
}

/// Get the global keyboard shortcuts
pub fn get_key_binds() -> &'static HashMap<menu::KeyBind, MenuAction> {
    KEY_BINDS.get().expect("KEY_BINDS not initialized")
}
