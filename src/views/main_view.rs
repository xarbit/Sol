use cosmic::iced::Length;
use cosmic::widget::{column, divider};
use cosmic::Element;

use crate::cache::CalendarCache;
use crate::components;
use crate::locale::LocalePreferences;
use crate::message::Message;
use crate::models::{WeekState, DayState};
use crate::views::{self, CalendarView};

/// Render the main content area (toolbar + calendar view)
pub fn render_main_content<'a>(
    cache: &'a CalendarCache,
    week_state: &'a WeekState,
    day_state: &'a DayState,
    locale: &'a LocalePreferences,
    current_view: CalendarView,
    selected_day: Option<u32>,
) -> Element<'a, Message> {
    // Render toolbar - use week/day text for week/day views
    let period_text = match current_view {
        CalendarView::Week => &week_state.week_range_text,
        CalendarView::Day => &day_state.month_year_text,
        _ => cache.current_period_text(),
    };
    let toolbar = components::render_toolbar(period_text, current_view);

    // Render current calendar view
    let calendar_view = match current_view {
        CalendarView::Month => views::render_month_view(cache.current_state(), selected_day),
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
