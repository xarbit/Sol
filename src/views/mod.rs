mod day;
mod main_view;
mod month;
mod sidebar;
mod week;
mod year;

pub use day::render_day_view;
pub use main_view::render_main_content;
pub use month::{render_month_view, MonthViewEvents};
pub use sidebar::render_sidebar;
pub use week::{render_week_view, week_time_grid_id, WeekViewEvents};
pub use year::render_year_view;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalendarView {
    Year,
    Month,
    Week,
    Day,
}

impl CalendarView {
    /// Get the next view in the cycle: Year → Month → Week → Day → Year
    pub fn next(self) -> Self {
        match self {
            CalendarView::Year => CalendarView::Month,
            CalendarView::Month => CalendarView::Week,
            CalendarView::Week => CalendarView::Day,
            CalendarView::Day => CalendarView::Year,
        }
    }

    /// Get the previous view in the cycle: Day → Week → Month → Year → Day
    pub fn previous(self) -> Self {
        match self {
            CalendarView::Year => CalendarView::Day,
            CalendarView::Month => CalendarView::Year,
            CalendarView::Week => CalendarView::Month,
            CalendarView::Day => CalendarView::Week,
        }
    }
}
