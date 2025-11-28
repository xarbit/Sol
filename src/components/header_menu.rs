use cosmic::widget::button;
use cosmic::{widget, Element};

use crate::fl;
use crate::message::Message;
use crate::ui_constants::{ICON_SEARCH, ICON_SIDEBAR};
use crate::views::CalendarView;

/// Render the left side of the header (sidebar toggle + menu items)
pub fn render_header_start() -> Vec<Element<'static, Message>> {
    vec![
        button::icon(widget::icon::from_name(ICON_SIDEBAR))
            .on_press(Message::ToggleSidebar)
            .into(),
        widget::button::text("File")
            .on_press(Message::NewEvent)
            .padding([4, 12])
            .into(),
        widget::button::text("Edit")
            .on_press(Message::Settings)
            .padding([4, 12])
            .into(),
        widget::button::text("View")
            .on_press(Message::ChangeView(CalendarView::Month))
            .padding([4, 12])
            .into(),
        widget::button::text("Help")
            .on_press(Message::About)
            .padding([4, 12])
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
