use cosmic::widget::menu;

use crate::message::Message;
use crate::views::CalendarView;

/// Menu actions for the application menu bar
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    NewEvent,
    Settings,
    ViewMonth,
    ViewWeek,
    ViewDay,
    About,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::NewEvent => Message::NewEvent,
            MenuAction::Settings => Message::Settings,
            MenuAction::ViewMonth => Message::ChangeView(CalendarView::Month),
            MenuAction::ViewWeek => Message::ChangeView(CalendarView::Week),
            MenuAction::ViewDay => Message::ChangeView(CalendarView::Day),
            MenuAction::About => Message::About,
        }
    }
}
