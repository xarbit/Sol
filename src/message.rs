use crate::views::CalendarView;

#[derive(Debug, Clone)]
pub enum Message {
    ChangeView(CalendarView),
    PreviousPeriod,
    NextPeriod,
    Today,
    SelectDay(u32),
    ToggleSidebar,
    ToggleSearch,
    ToggleCalendar(String),
    OpenColorPicker(String),
    ChangeCalendarColor(String, String),
    CloseColorPicker,
    MiniCalendarPrevMonth,
    MiniCalendarNextMonth,
    NewEvent,
    Settings,
    About,
    LaunchUrl(String),
    ToggleContextDrawer,
}
