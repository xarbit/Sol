use cosmic::widget::menu;

use crate::message::Message;
use crate::views::CalendarView;

/// Menu actions for the application menu bar
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    NewEvent,
    NewCalendar,
    ImportICal,
    ExportICal,
    Settings,
    Today,
    ViewYear,
    ViewMonth,
    ViewWeek,
    ViewDay,
    ToggleWeekNumbers,
    About,
    // Navigation shortcuts (Ctrl+Shift+Arrow keys)
    NavigatePrevious,
    NavigateNext,
    ScrollTimelineUp,
    ScrollTimelineDown,
    // Event actions
    DeleteSelectedEvent,
    // View cycling (V + Arrow keys)
    CycleViewNext,
    CycleViewPrevious,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::NewEvent => Message::NewEvent,
            MenuAction::NewCalendar => Message::OpenNewCalendarDialog,
            MenuAction::ImportICal => Message::ImportICal,
            MenuAction::ExportICal => Message::ExportICal,
            MenuAction::Settings => Message::Settings,
            MenuAction::Today => Message::Today,
            MenuAction::ViewYear => Message::ChangeView(CalendarView::Year),
            MenuAction::ViewMonth => Message::ChangeView(CalendarView::Month),
            MenuAction::ViewWeek => Message::ChangeView(CalendarView::Week),
            MenuAction::ViewDay => Message::ChangeView(CalendarView::Day),
            MenuAction::ToggleWeekNumbers => Message::ToggleWeekNumbers,
            MenuAction::About => Message::About,
            MenuAction::NavigatePrevious => Message::PreviousPeriod,
            MenuAction::NavigateNext => Message::NextPeriod,
            MenuAction::ScrollTimelineUp => Message::ScrollTimelineUp,
            MenuAction::ScrollTimelineDown => Message::ScrollTimelineDown,
            MenuAction::DeleteSelectedEvent => Message::RequestDeleteSelectedEvent,
            MenuAction::CycleViewNext => Message::CycleViewNext,
            MenuAction::CycleViewPrevious => Message::CycleViewPrevious,
        }
    }
}
