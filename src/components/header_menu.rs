use cosmic::widget::{button, menu};
use cosmic::{widget, Element};
use cosmic::app::Core;
use std::collections::HashMap;

use crate::fl;
use crate::menu_action::MenuAction;
use crate::message::Message;
use crate::ui_constants::{ICON_ADD, ICON_SEARCH, ICON_SIDEBAR_OPEN, ICON_SIDEBAR_CLOSED, ICON_TODAY, MENU_ITEM_HEIGHT, MENU_ITEM_WIDTH, MENU_SPACING};

const MENU_ID: &str = "sol-calendar-menu";

/// Render the left side of the header (sidebar toggle + menu items)
pub fn render_header_start<'a>(
    core: &'a Core,
    key_binds: &'a HashMap<menu::KeyBind, MenuAction>,
    sidebar_visible: bool,
    show_week_numbers: bool,
) -> Vec<Element<'a, Message>> {
    let sidebar_icon = if sidebar_visible {
        ICON_SIDEBAR_OPEN
    } else {
        ICON_SIDEBAR_CLOSED
    };

    vec![
        button::icon(widget::icon::from_name(sidebar_icon))
            .on_press(Message::ToggleSidebar)
            .into(),
        widget::responsive_menu_bar()
            .item_height(menu::ItemHeight::Dynamic(MENU_ITEM_HEIGHT))
            .item_width(menu::ItemWidth::Uniform(MENU_ITEM_WIDTH))
            .spacing(MENU_SPACING)
            .into_element(
                core,
                key_binds,
                widget::Id::new(MENU_ID),
                Message::Surface,
                vec![
                    (fl!("menu-file"), vec![
                        menu::Item::Button(fl!("menu-new-event"), None, MenuAction::NewEvent),
                    ]),
                    (fl!("menu-edit"), vec![
                        menu::Item::Button(fl!("menu-settings"), None, MenuAction::Settings),
                    ]),
                    (fl!("menu-view"), vec![
                        menu::Item::Button(fl!("menu-today"), None, MenuAction::Today),
                        menu::Item::Divider,
                        menu::Item::Button(fl!("menu-day-view"), None, MenuAction::ViewDay),
                        menu::Item::Button(fl!("menu-week-view"), None, MenuAction::ViewWeek),
                        menu::Item::Button(fl!("menu-month-view"), None, MenuAction::ViewMonth),
                        menu::Item::Button(fl!("menu-year-view"), None, MenuAction::ViewYear),
                        menu::Item::Divider,
                        menu::Item::CheckBox(fl!("menu-show-week-numbers"), None, show_week_numbers, MenuAction::ToggleWeekNumbers),
                        menu::Item::Divider,
                        menu::Item::Button(fl!("menu-about"), None, MenuAction::About),
                    ]),
                ],
            ),
    ]
}

/// Render the right side of the header (add, today, and search buttons)
pub fn render_header_end() -> Vec<Element<'static, Message>> {
    vec![
        button::icon(widget::icon::from_name(ICON_ADD))
            .on_press(Message::NewEvent)
            .into(),
        button::icon(widget::icon::from_name(ICON_TODAY))
            .on_press(Message::Today)
            .into(),
        button::icon(widget::icon::from_name(ICON_SEARCH))
            .on_press(Message::ToggleSearch)
            .into(),
    ]
}
