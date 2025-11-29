use chrono::NaiveDate;
use cosmic::iced::Length;
use cosmic::widget::{column, divider};
use cosmic::Element;

use crate::cache::CalendarCache;
use crate::components;
use crate::locale::LocalePreferences;
use crate::message::Message;
use crate::models::{WeekState, DayState, YearState};
use crate::views::{self, CalendarView, MonthViewEvents};

/// Render the main content area (toolbar + calendar view)
pub fn render_main_content<'a>(
    cache: &'a CalendarCache,
    week_state: &'a WeekState,
    day_state: &'a DayState,
    year_state: &'a YearState,
    locale: &'a LocalePreferences,
    current_view: CalendarView,
    selected_date: Option<NaiveDate>,
    show_week_numbers: bool,
    month_events: Option<MonthViewEvents<'a>>,
) -> Element<'a, Message> {
    // Render toolbar - use appropriate text for each view
    // primary_text is bold (month/period), secondary_text is normal weight (year)
    let (primary_text, secondary_text): (String, String) = match current_view {
        CalendarView::Year => (year_state.year_text.clone(), String::new()),
        CalendarView::Week => (week_state.week_range_text.clone(), String::new()),
        CalendarView::Day => (day_state.month_year_text.clone(), String::new()),
        CalendarView::Month => (cache.current_month_text(), cache.current_year_text()),
    };
    let toolbar = components::render_toolbar(&primary_text, &secondary_text);

    // Render current calendar view
    let calendar_view = match current_view {
        CalendarView::Year => views::render_year_view(year_state, locale),
        CalendarView::Month => views::render_month_view(cache.current_state(), selected_date, locale, show_week_numbers, month_events),
        CalendarView::Week => views::render_week_view(week_state, locale),
        CalendarView::Day => views::render_day_view(day_state, locale),
    };

    column()
        .spacing(0)
        .push(toolbar)
        .push(divider::horizontal::default())
        .push(calendar_view)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
