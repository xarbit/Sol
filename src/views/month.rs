use std::collections::HashMap;

use chrono::{Datelike, NaiveDate};
use cosmic::iced::widget::stack;
use cosmic::iced::{alignment, Length, Size};
use cosmic::widget::{column, container, row, responsive};
use cosmic::{widget, Element};

use crate::components::{render_day_cell_with_events, render_spanning_quick_event_input, DayCellConfig, DisplayEvent};
use crate::dialogs::ActiveDialog;
use crate::models::CalendarDay;
use crate::fl;
use crate::locale::LocalePreferences;
use crate::localized_names;
use crate::message::Message;
use crate::models::CalendarState;
use crate::selection::SelectionState;
use crate::ui_constants::{
    FONT_SIZE_MEDIUM, FONT_SIZE_SMALL, PADDING_SMALL, PADDING_MONTH_GRID,
    SPACING_TINY, WEEK_NUMBER_WIDTH
};

/// Height of the spanning quick event input overlay
const SPANNING_INPUT_HEIGHT: f32 = 36.0;

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
    /// Selection state for drag selection
    pub selection: &'a SelectionState,
    /// Active dialog state (for showing selection highlight during quick event input)
    pub active_dialog: &'a ActiveDialog,
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

/// Compute slot assignments for multi-day all-day events in a week.
/// Returns a map of event UID -> slot index.
/// Events that span multiple days get consistent slots across all days they appear.
fn compute_week_event_slots(
    week: &[CalendarDay],
    events_by_date: &HashMap<NaiveDate, Vec<DisplayEvent>>,
) -> HashMap<String, usize> {
    let mut slots: HashMap<String, usize> = HashMap::new();
    let mut next_slot: usize = 0;

    // Get dates for this week
    let week_dates: Vec<NaiveDate> = week
        .iter()
        .filter_map(|d| NaiveDate::from_ymd_opt(d.year, d.month, d.day))
        .collect();

    if week_dates.is_empty() {
        return slots;
    }

    let week_start = week_dates[0];
    let week_end = week_dates[week_dates.len() - 1];

    // Collect all multi-day all-day events that appear in this week
    // We need to find them and sort by their start date
    let mut multi_day_events: Vec<(NaiveDate, NaiveDate, String)> = Vec::new();
    let mut seen_uids: std::collections::HashSet<String> = std::collections::HashSet::new();

    for date in &week_dates {
        if let Some(day_events) = events_by_date.get(date) {
            for event in day_events {
                if event.is_multi_day() && !seen_uids.contains(&event.uid) {
                    if let (Some(start), Some(end)) = (event.span_start, event.span_end) {
                        // Only include if it overlaps with this week
                        if start <= week_end && end >= week_start {
                            seen_uids.insert(event.uid.clone());
                            multi_day_events.push((start, end, event.uid.clone()));
                        }
                    }
                }
            }
        }
    }

    // Sort by start date, then by end date (longer events first for stable ordering)
    multi_day_events.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then_with(|| b.1.cmp(&a.1)) // Longer events first (earlier end = shorter)
    });

    // Assign slots in order
    for (_, _, uid) in multi_day_events {
        slots.insert(uid, next_slot);
        next_slot += 1;
    }

    slots
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
        // Compute slot assignments for multi-day events in this week
        let event_slots = events
            .as_ref()
            .map(|e| compute_week_event_slots(week, e.events_by_date))
            .unwrap_or_default();

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
            // Include all events - multi-day events show in each cell they span
            let day_events: Vec<DisplayEvent> = if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                events
                    .as_ref()
                    .and_then(|e| e.events_by_date.get(&date))
                    .cloned()
                    .unwrap_or_default()
            } else {
                vec![]
            };

            // Quick event input is always rendered as a spanning overlay (even for single-day)
            // This provides consistent UX for all quick event creation
            let quick_event_data: Option<(String, String)> = None;

            // Check if this day is in the current drag selection range
            // Also check if there's an active multi-day quick event that includes this date
            let (is_in_selection, selection_active) = if let Some(cell_date) = cell_date {
                events.as_ref().map(|e| {
                    // Show highlight if: actively dragging OR in quick event date range
                    let in_drag_selection = e.selection.contains(cell_date);
                    let in_quick_event_range = e.active_dialog.is_date_in_quick_event_range(cell_date);
                    let is_active = e.selection.is_active || e.active_dialog.is_multi_day_quick_event();
                    (in_drag_selection || in_quick_event_range, is_active)
                }).unwrap_or((false, false))
            } else {
                (false, false)
            };

            let cell = render_day_cell_with_events(DayCellConfig {
                year,
                month,
                day,
                is_today,
                is_selected,
                is_weekend,
                is_adjacent_month: !is_current_month,
                events: day_events,
                event_slots: event_slots.clone(),
                quick_event: quick_event_data,
                is_in_selection,
                selection_active,
            });

            week_row = week_row.push(
                container(cell)
                    .width(Length::Fill)
                    .height(Length::Fill)
            );
        }
        grid = grid.push(week_row);
    }

    // Check if we need to render a spanning quick event overlay
    // Used for all quick events (single-day and multi-day) for consistent UX
    let has_quick_event_overlay = events
        .as_ref()
        .map(|e| e.active_dialog.is_quick_event())
        .unwrap_or(false);

    // Build the final view with quick event overlay if needed
    let base = container(grid)
        .width(Length::Fill)
        .height(Length::Fill);

    // If we have a quick event input, add that overlay on top
    if has_quick_event_overlay {
        let quick_overlay = events.as_ref().and_then(|e| {
            e.active_dialog.quick_event_range().map(|(start, end, text)| {
                let color = e.quick_event
                    .as_ref()
                    .map(|(_, _, c)| c.to_string())
                    .unwrap_or_else(|| "#3B82F6".to_string());

                render_spanning_overlay(
                    &calendar_state.weeks_full,
                    start,
                    end,
                    text.to_string(),
                    color,
                    show_week_numbers,
                )
            })
        });

        if let Some(overlay_element) = quick_overlay {
            return stack![base, overlay_element].into();
        }
    }

    base.into()
}

/// Render the spanning quick event overlay positioned over the selected date range
fn render_spanning_overlay<'a>(
    weeks: &[Vec<CalendarDay>],
    start_date: NaiveDate,
    end_date: NaiveDate,
    text: String,
    calendar_color: String,
    show_week_numbers: bool,
) -> Element<'a, Message> {
    // Find which week(s) the selection spans
    let mut overlay_rows: Vec<(usize, usize, usize)> = Vec::new(); // (week_index, start_col, end_col)

    for (week_idx, week) in weeks.iter().enumerate() {
        let mut week_start_col: Option<usize> = None;
        let mut week_end_col: Option<usize> = None;

        for (day_idx, calendar_day) in week.iter().enumerate() {
            if let Some(cell_date) = NaiveDate::from_ymd_opt(
                calendar_day.year,
                calendar_day.month,
                calendar_day.day,
            ) {
                if cell_date >= start_date && cell_date <= end_date {
                    if week_start_col.is_none() {
                        week_start_col = Some(day_idx);
                    }
                    week_end_col = Some(day_idx);
                }
            }
        }

        if let (Some(start_col), Some(end_col)) = (week_start_col, week_end_col) {
            overlay_rows.push((week_idx, start_col, end_col));
        }
    }

    // Build the overlay structure matching the grid layout
    let mut overlay_column = column()
        .spacing(SPACING_TINY)
        .padding(PADDING_MONTH_GRID);

    // Add header spacer (same height as weekday header)
    overlay_column = overlay_column.push(
        container(widget::text(""))
            .height(Length::Fixed(WEEKDAY_HEADER_HEIGHT))
    );

    let num_weeks = weeks.len();

    for week_idx in 0..num_weeks {
        // Check if this week has part of the selection
        let week_overlay = overlay_rows.iter().find(|(idx, _, _)| *idx == week_idx);

        if let Some((_, start_col, end_col)) = week_overlay {
            // This week has the selection - render the spanning input
            let mut week_row = row().spacing(SPACING_TINY).height(Length::Fill);

            // Week number spacer (if enabled)
            if show_week_numbers {
                week_row = week_row.push(
                    container(widget::text(""))
                        .width(Length::Fixed(WEEK_NUMBER_WIDTH))
                );
            }

            // Add empty spacers for columns before the selection
            for _ in 0..*start_col {
                week_row = week_row.push(
                    container(widget::text(""))
                        .width(Length::Fill)
                );
            }

            // Calculate span (number of columns the input covers)
            let span_columns = end_col - start_col + 1;

            // Create a container that spans the selected columns
            // We use FillPortion to make it take the right amount of space
            let input = render_spanning_quick_event_input(
                text.clone(),
                calendar_color.clone(),
                span_columns,
            );

            // Wrap in a container with the correct proportion
            let spanning_container = container(input)
                .width(Length::FillPortion(span_columns as u16))
                .height(Length::Fixed(SPANNING_INPUT_HEIGHT))
                .center_y(Length::Fixed(SPANNING_INPUT_HEIGHT));

            week_row = week_row.push(spanning_container);

            // Add empty spacers for columns after the selection
            for _ in (*end_col + 1)..7 {
                week_row = week_row.push(
                    container(widget::text(""))
                        .width(Length::Fill)
                );
            }

            overlay_column = overlay_column.push(week_row);
        } else {
            // Empty row - just a spacer with the same height
            overlay_column = overlay_column.push(
                container(widget::text(""))
                    .height(Length::Fill)
            );
        }
    }

    container(overlay_column)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

