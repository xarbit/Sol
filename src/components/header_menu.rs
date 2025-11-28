use cosmic::widget::{button, menu};
use cosmic::{widget, Apply, Element};
use std::collections::HashMap;

use crate::menu_action::MenuAction;
use crate::message::Message;
use crate::ui_constants::{ICON_SEARCH, ICON_SIDEBAR};

/// Render the left side of the header (sidebar toggle + menu items)
pub fn render_header_start() -> Vec<Element<'static, Message>> {
    let key_binds: HashMap<menu::KeyBind, MenuAction> = HashMap::new();

    vec![
        button::icon(widget::icon::from_name(ICON_SIDEBAR))
            .on_press(Message::ToggleSidebar)
            .into(),
        menu::bar(vec![
            menu::Tree::with_children(
                menu::root("File").apply(Element::from),
                menu::items(
                    &key_binds,
                    vec![
                        menu::Item::Button("New Event", None, MenuAction::NewEvent),
                    ],
                ),
            ),
            menu::Tree::with_children(
                menu::root("Edit").apply(Element::from),
                menu::items(
                    &key_binds,
                    vec![
                        menu::Item::Button("Settings", None, MenuAction::Settings),
                    ],
                ),
            ),
            menu::Tree::with_children(
                menu::root("View").apply(Element::from),
                menu::items(
                    &key_binds,
                    vec![
                        menu::Item::Button("Month", None, MenuAction::ViewMonth),
                        menu::Item::Button("Week", None, MenuAction::ViewWeek),
                        menu::Item::Button("Day", None, MenuAction::ViewDay),
                        menu::Item::Divider,
                        menu::Item::Button("About Sol Calendar", None, MenuAction::About),
                    ],
                ),
            ),
        ])
        .into(),
    ]
}

/// Render the right side of the header (search button)
pub fn render_header_end() -> Vec<Element<'static, Message>> {
    vec![
        button::icon(widget::icon::from_name(ICON_SEARCH))
            .on_press(Message::ToggleSearch)
            .into()
    ]
}
