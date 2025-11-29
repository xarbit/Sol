use chrono::{Datelike, NaiveDate};
use cosmic::iced::{alignment, Length, Size};
use cosmic::widget::{column, container, row, responsive};
use cosmic::{widget, Element};

use crate::components::{render_day_cell_with_events, DayCellConfig, DisplayEvent};
use crate::models::CalendarDay;
use crate::fl;
use crate::locale::LocalePreferences;
use crate::localized_names;
use crate::message::Message;
use crate::models::CalendarState;
use crate::ui_constants::{
    FONT_SIZE_MEDIUM, FONT_SIZE_SMALL, PADDING_SMALL, PADDING_MONTH_GRID,
    SPACING_TINY, WEEK_NUMBER_WIDTH
};

/// Minimum width per day cell to use full weekday names
/// Below this threshold, short names are used
const MIN_CELL_WIDTH_FOR_FULL_NAMES: f32 = 100.0;

/// Fixed height for the weekday header row
const WEEKDAY_HEADER_HEIGHT: f32 = 32.0;

/// Events grouped by day for display in the month view
pub struct MonthViewEvents<'a> {
    /// Events for each day, keyed by full date (supports adjacent month days)
    pub events_by_date: &'a std::collections::HashMap<NaiveDate, Vec<DisplayEvent>>,
    /// Quick event editing state: (date, text, calendar_color)
    pub quick_event: Option<(NaiveDate, &'a str, &'a str)>,
}

/// Render the weekday header row with responsive names
fn render_weekday_header(show_week_numbers: bool, use_short_names: bool) -> Element<'static, Message> {
    let mut header_row = row().spacing(SPACING_TINY);

    // Week number header (only if enabled)
    if show_week_numbers {
        header_row = header_row.push(
            container(widget::text(fl!("week-abbr")).size(FONT_SIZE_SMALL))
                .width(Length::Fixed(WEEK_NUMBER_WIDTH))
                .padding(PADDING_SMALL)
                .align_y(alignment::Vertical::Center)
        );
    }

    // Weekday headers - use short or full names based on available width
    let weekday_names = if use_short_names {
        localized_names::get_weekday_names_short()
    } else {
        localized_names::get_weekday_names_full()
    };

    for weekday in weekday_names {
        header_row = header_row.push(
            container(widget::text(weekday).size(FONT_SIZE_MEDIUM))
                .width(Length::Fill)
                .padding(PADDING_SMALL)
                .center_x(Length::Fill),
        );
    }

    header_row.into()
}

pub fn render_month_view<'a>(
    calendar_state: &CalendarState,
    selected_date: Option<NaiveDate>,
    locale: &LocalePreferences,
    show_week_numbers: bool,
    events: Option<MonthViewEvents<'a>>,
) -> Element<'a, Message> {
    let mut grid = column().spacing(SPACING_TINY).padding(PADDING_MONTH_GRID);

    // Responsive weekday header - uses short names when cells are narrow
    let week_number_offset = if show_week_numbers { WEEK_NUMBER_WIDTH } else { 0.0 };
    let header = responsive(move |size: Size| {
        // Calculate approximate cell width (7 days + spacing)
        let available_for_days = size.width - week_number_offset - (SPACING_TINY as f32 * 6.0);
        let cell_width = available_for_days / 7.0;
        let use_short_names = cell_width < MIN_CELL_WIDTH_FOR_FULL_NAMES;
        render_weekday_header(show_week_numbers, use_short_names)
    });

    // Fixed height container for the header to prevent it from expanding
    grid = grid.push(container(header).height(Length::Fixed(WEEKDAY_HEADER_HEIGHT)));

    // Get week numbers for the month
    let week_numbers = calendar_state.week_numbers();

    // Use pre-calculated weeks from CalendarState cache (with adjacent month days)
    for (week_index, week) in calendar_state.weeks_full.iter().enumerate() {
        let mut week_row = row().spacing(SPACING_TINY).height(Length::Fill);

        // Week number cell (only if enabled)
        if show_week_numbers {
            let week_number = week_numbers.get(week_index).copied().unwrap_or(0);
            week_row = week_row.push(
                container(
                    widget::text(format!("{}", week_number))
                        .size(FONT_SIZE_SMALL)
                )
                .width(Length::Fixed(WEEK_NUMBER_WIDTH))
                .height(Length::Fill)
                .padding(PADDING_SMALL)
                .align_y(alignment::Vertical::Center)
            );
        }

        // Day cells
        for calendar_day in week {
            let CalendarDay { year, month, day, is_current_month } = *calendar_day;

            // Check if today (need to compare full date)
            let today = chrono::Local::now();
            let is_today = year == today.year()
                && month == today.month()
                && day == today.day();

            // Check if this day is selected (works for both current and adjacent month days)
            let cell_date = NaiveDate::from_ymd_opt(year, month, day);
            let is_selected = selected_date.is_some() && cell_date == selected_date;

            // Get weekday for weekend detection
            let weekday = chrono::NaiveDate::from_ymd_opt(year, month, day)
                .map(|d| d.weekday())
                .unwrap_or(chrono::Weekday::Mon);
            let is_weekend = locale.is_weekend(weekday);

            // Get events for this day using full date as key (works for adjacent months too)
            let day_events: Vec<DisplayEvent> = if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                events
                    .as_ref()
                    .and_then(|e| e.events_by_date.get(&date))
                    .cloned()
                    .unwrap_or_default()
            } else {
                vec![]
            };

            // Check if quick event input should be shown for this day
            let quick_event_data: Option<(String, String)> = events.as_ref().and_then(|e| {
                e.quick_event.as_ref().and_then(|(date, text, color)| {
                    if date.day() == day
                        && date.month() == month
                        && date.year() == year
                    {
                        Some((text.to_string(), color.to_string()))
                    } else {
                        None
                    }
                })
            });

            let cell = render_day_cell_with_events(DayCellConfig {
                year,
                month,
                day,
                is_today,
                is_selected,
                is_weekend,
                is_adjacent_month: !is_current_month,
                events: day_events,
                quick_event: quick_event_data,
            });

            week_row = week_row.push(
                container(cell)
                    .width(Length::Fill)
                    .height(Length::Fill)
            );
        }
        grid = grid.push(week_row);
    }

    container(grid)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
