use cosmic::widget::{button, menu};
use cosmic::{widget, Apply, Element};
use cosmic::app::Core;
use std::collections::HashMap;

use crate::menu_action::MenuAction;
use crate::message::Message;
use crate::ui_constants::{ICON_SEARCH, ICON_SIDEBAR};

const MENU_ID: &str = "sol-calendar-menu";

/// Render the left side of the header (sidebar toggle + menu items)
pub fn render_header_start<'a>(
    core: &'a Core,
    key_binds: &'a HashMap<menu::KeyBind, MenuAction>,
) -> Vec<Element<'a, Message>> {
    vec![
        button::icon(widget::icon::from_name(ICON_SIDEBAR))
            .on_press(Message::ToggleSidebar)
            .into(),
        widget::responsive_menu_bar()
            .item_height(menu::ItemHeight::Dynamic(40))
            .item_width(menu::ItemWidth::Uniform(240))
            .spacing(4.0)
            .into_element(
                core,
                key_binds,
                widget::Id::new(MENU_ID),
                Message::Surface,
                vec![
                    ("File", vec![
                        menu::Item::Button("New Event...", None, MenuAction::NewEvent),
                    ]),
                    ("Edit", vec![
                        menu::Item::Button("Settings...", None, MenuAction::Settings),
                    ]),
                    ("View", vec![
                        menu::Item::Button("Month View", None, MenuAction::ViewMonth),
                        menu::Item::Button("Week View", None, MenuAction::ViewWeek),
                        menu::Item::Button("Day View", None, MenuAction::ViewDay),
                        menu::Item::Divider,
                        menu::Item::Button("About Sol Calendar", None, MenuAction::About),
                    ]),
                ],
            ),
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
