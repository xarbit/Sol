use chrono::{Datelike, NaiveDate, NaiveTime, Timelike};
use cosmic::iced::{alignment, Background, Border, Length};
use cosmic::iced::widget::stack;
use cosmic::iced_widget::text_input;
use cosmic::widget::{column, container, mouse_area, row, scrollable};
use cosmic::{widget, Element};
use std::collections::HashMap;

use crate::components::{DisplayEvent, parse_hex_color, quick_event_input_id};

/// Returns the scrollable ID for the week view time grid
pub fn week_time_grid_id() -> cosmic::iced_core::id::Id {
    cosmic::iced_core::id::Id::new("week_time_grid")
}
use crate::dialogs::ActiveDialog;
use crate::locale::LocalePreferences;
use crate::localized_names;
use crate::message::Message;
use crate::models::WeekState;
use crate::selection::SelectionState;
use crate::styles::today_filled_style;
use crate::ui_constants::{
    PADDING_SMALL, FONT_SIZE_SMALL, FONT_SIZE_MEDIUM, COLOR_DAY_CELL_BORDER,
    HOUR_ROW_HEIGHT, TIME_LABEL_WIDTH, COLOR_WEEKEND_BACKGROUND, BORDER_WIDTH_THIN,
    SPACING_TINY, COLOR_CURRENT_TIME, BORDER_RADIUS,
};

/// Represents an event with its calculated column position for overlap handling
#[derive(Clone)]
struct PositionedEvent {
    event: DisplayEvent,
    column: usize,
    total_columns: usize,
}

/// Height of the day header row
const DAY_HEADER_HEIGHT: f32 = 60.0;

/// Minimum height for the all-day events section
const ALL_DAY_MIN_HEIGHT: f32 = 28.0;

/// Height per all-day event row
const ALL_DAY_EVENT_HEIGHT: f32 = 22.0;

/// Spacing between all-day events
const ALL_DAY_SPACING: f32 = 2.0;

/// Events grouped by day for display in the week view
pub struct WeekViewEvents<'a> {
    /// Events for each day, keyed by date
    pub events_by_date: &'a HashMap<NaiveDate, Vec<DisplayEvent>>,
    /// Currently selected event UID (for visual feedback)
    pub selected_event_uid: Option<&'a str>,
    /// Selection state for time slot highlighting
    pub selection: &'a SelectionState,
    /// Active dialog state (for quick event input)
    pub active_dialog: &'a ActiveDialog,
    /// Selected calendar color (for quick event styling)
    pub calendar_color: &'a str,
}

pub fn render_week_view<'a>(
    week_state: &'a WeekState,
    locale: &'a LocalePreferences,
    events: Option<WeekViewEvents<'a>>,
) -> Element<'a, Message> {
    // Extract selected event UID for selection highlighting
    let selected_event_uid = events.as_ref().and_then(|e| e.selected_event_uid);

    // Extract selection state for time slot highlighting
    let selection = events.as_ref().map(|e| e.selection);

    // Extract active dialog and calendar color for quick event input
    let active_dialog = events.as_ref().map(|e| e.active_dialog);
    let calendar_color = events.as_ref().map(|e| e.calendar_color);

    // Separate events into all-day and timed
    let (all_day_events, timed_events) = if let Some(ref ev) = events {
        separate_events(ev.events_by_date, &week_state.days)
    } else {
        (HashMap::new(), HashMap::new())
    };

    // Calculate how many rows we need for all-day events
    let max_all_day_slots = calculate_max_all_day_slots(&all_day_events);
    let all_day_section_height = ALL_DAY_MIN_HEIGHT + (max_all_day_slots as f32 * (ALL_DAY_EVENT_HEIGHT + ALL_DAY_SPACING));

    // Day headers with all-day events section
    let header_section = render_header_section(week_state, locale, &all_day_events, all_day_section_height, selected_event_uid);

    // Time grid with timed events
    let time_grid = render_time_grid_with_events(locale, week_state, &timed_events, selected_event_uid, selection, active_dialog, calendar_color);

    let content = column()
        .spacing(0)
        .push(header_section)
        .push(
            scrollable(time_grid)
                .id(week_time_grid_id())
                .height(Length::Fill)
        );

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Separate events into all-day and timed categories
fn separate_events(
    events_by_date: &HashMap<NaiveDate, Vec<DisplayEvent>>,
    week_days: &[NaiveDate],
) -> (HashMap<NaiveDate, Vec<DisplayEvent>>, HashMap<NaiveDate, Vec<DisplayEvent>>) {
    let mut all_day: HashMap<NaiveDate, Vec<DisplayEvent>> = HashMap::new();
    let mut timed: HashMap<NaiveDate, Vec<DisplayEvent>> = HashMap::new();

    for day in week_days {
        if let Some(day_events) = events_by_date.get(day) {
            for event in day_events {
                if event.all_day {
                    all_day.entry(*day).or_default().push(event.clone());
                } else {
                    timed.entry(*day).or_default().push(event.clone());
                }
            }
        }
    }

    (all_day, timed)
}

/// Calculate the maximum number of all-day event slots needed
fn calculate_max_all_day_slots(all_day_events: &HashMap<NaiveDate, Vec<DisplayEvent>>) -> usize {
    all_day_events.values().map(|v| v.len()).max().unwrap_or(0)
}

/// Render the header section with day names, dates, and all-day events
fn render_header_section<'a>(
    week_state: &'a WeekState,
    locale: &'a LocalePreferences,
    all_day_events: &HashMap<NaiveDate, Vec<DisplayEvent>>,
    all_day_section_height: f32,
    selected_event_uid: Option<&str>,
) -> Element<'a, Message> {
    let mut header_col = column().spacing(0);

    // Day headers row
    let mut day_headers = row().spacing(0);

    // Time column placeholder for day headers
    day_headers = day_headers.push(
        container(widget::text(""))
            .width(Length::Fixed(TIME_LABEL_WIDTH))
            .height(Length::Fixed(DAY_HEADER_HEIGHT))
    );

    // Day headers
    for date in &week_state.days {
        let is_today = week_state.is_today(date);
        let is_weekend = locale.is_weekend(date.weekday());
        let day_name = localized_names::get_weekday_short(date.weekday());
        let day_number = format!("{}", date.day());

        let day_header = render_day_header(&day_name, &day_number, is_today);

        day_headers = day_headers.push(
            container(day_header)
                .width(Length::Fill)
                .height(Length::Fixed(DAY_HEADER_HEIGHT))
                .padding(PADDING_SMALL)
                .center_x(Length::Fill)
                .style(move |_theme: &cosmic::Theme| container::Style {
                    background: if is_weekend {
                        Some(Background::Color(COLOR_WEEKEND_BACKGROUND))
                    } else {
                        None
                    },
                    border: Border {
                        width: BORDER_WIDTH_THIN,
                        color: COLOR_DAY_CELL_BORDER,
                        ..Default::default()
                    },
                    ..Default::default()
                })
        );
    }

    header_col = header_col.push(day_headers);

    // All-day events section
    let all_day_section = render_all_day_section(week_state, locale, all_day_events, all_day_section_height, selected_event_uid);
    header_col = header_col.push(all_day_section);

    header_col.into()
}

/// Render a single day header with day name and number
fn render_day_header<'a>(day_name: &str, day_number: &str, is_today: bool) -> Element<'a, Message> {
    let day_number_element: Element<'a, Message> = if is_today {
        container(
            widget::text(day_number.to_string()).size(FONT_SIZE_MEDIUM)
        )
        .padding(PADDING_SMALL)
        .style(|theme: &cosmic::Theme| today_filled_style(theme))
        .into()
    } else {
        widget::text(day_number.to_string()).size(FONT_SIZE_MEDIUM).into()
    };

    column()
        .spacing(SPACING_TINY)
        .align_x(alignment::Horizontal::Center)
        .push(widget::text(day_name.to_string()).size(FONT_SIZE_SMALL))
        .push(day_number_element)
        .into()
}

/// Render the all-day events section
fn render_all_day_section<'a>(
    week_state: &'a WeekState,
    locale: &'a LocalePreferences,
    all_day_events: &HashMap<NaiveDate, Vec<DisplayEvent>>,
    height: f32,
    selected_event_uid: Option<&str>,
) -> Element<'a, Message> {
    let mut all_day_row = row().spacing(0);

    // Time column with "all-day" label
    all_day_row = all_day_row.push(
        container(
            widget::text("").size(FONT_SIZE_SMALL)
        )
        .width(Length::Fixed(TIME_LABEL_WIDTH))
        .height(Length::Fixed(height))
        .padding(PADDING_SMALL)
        .align_y(alignment::Vertical::Top)
        .style(|_theme: &cosmic::Theme| container::Style {
            border: Border {
                width: BORDER_WIDTH_THIN,
                color: COLOR_DAY_CELL_BORDER,
                ..Default::default()
            },
            ..Default::default()
        })
    );

    // All-day events for each day
    for date in &week_state.days {
        let is_weekend = locale.is_weekend(date.weekday());
        let day_events = all_day_events.get(date).cloned().unwrap_or_default();
        let date_copy = *date;

        let events_column = render_all_day_events_for_day(*date, &day_events, selected_event_uid);

        let cell = container(events_column)
            .width(Length::Fill)
            .height(Length::Fixed(height))
            .padding([2, 2])
            .style(move |_theme: &cosmic::Theme| container::Style {
                background: if is_weekend {
                    Some(Background::Color(COLOR_WEEKEND_BACKGROUND))
                } else {
                    None
                },
                border: Border {
                    width: BORDER_WIDTH_THIN,
                    color: COLOR_DAY_CELL_BORDER,
                    ..Default::default()
                },
                ..Default::default()
            });

        // Wrap in mouse_area for drag support (on_enter allows receiving drag updates)
        let clickable_cell = mouse_area(cell)
            .on_enter(Message::DragEventUpdate(date_copy));

        all_day_row = all_day_row.push(clickable_cell);
    }

    all_day_row.into()
}

/// Render all-day events for a single day as a vertical stack with click and drag support
fn render_all_day_events_for_day(date: NaiveDate, events: &[DisplayEvent], selected_event_uid: Option<&str>) -> Element<'static, Message> {
    let mut col = column().spacing(ALL_DAY_SPACING as u16);

    for event in events {
        let color = parse_hex_color(&event.color)
            .unwrap_or(cosmic::iced::Color::from_rgb(0.5, 0.5, 0.5));
        let uid = event.uid.clone();
        let is_selected = selected_event_uid == Some(&event.uid);

        // Selection highlight
        let (bg_opacity, border_width) = if is_selected {
            (0.9, 2.0)
        } else {
            (0.85, 0.0)
        };

        let chip = container(
            widget::text(event.summary.clone())
                .size(10)
        )
        .padding([2, 4])
        .width(Length::Fill)
        .height(Length::Fixed(ALL_DAY_EVENT_HEIGHT))
        .style(move |theme: &cosmic::Theme| container::Style {
            background: Some(Background::Color(cosmic::iced::Color {
                a: bg_opacity,
                ..color
            })),
            text_color: Some(cosmic::iced::Color::WHITE),
            border: Border {
                radius: BORDER_RADIUS.into(),
                width: border_width,
                color: if is_selected {
                    theme.cosmic().accent_color().into()
                } else {
                    cosmic::iced::Color::TRANSPARENT
                },
            },
            ..Default::default()
        });

        // Get color hex for drag preview
        let color_hex = event.color.clone();

        // Wrap with mouse area for click and drag handling
        let clickable_chip = mouse_area(chip)
            .on_press(Message::DragEventStart(uid.clone(), date, event.summary.clone(), color_hex))
            .on_release(Message::DragEventEnd)
            .on_double_click(Message::OpenEditEventDialog(uid))
            .on_enter(Message::DragEventUpdate(date));

        col = col.push(clickable_chip);
    }

    col.into()
}

/// Render the time grid with timed events spanning their full duration
fn render_time_grid_with_events<'a>(
    locale: &'a LocalePreferences,
    week_state: &'a WeekState,
    timed_events: &HashMap<NaiveDate, Vec<DisplayEvent>>,
    selected_event_uid: Option<&'a str>,
    selection: Option<&'a SelectionState>,
    active_dialog: Option<&'a ActiveDialog>,
    calendar_color: Option<&'a str>,
) -> Element<'a, Message> {
    // Get current time for the "now" indicator
    let now = chrono::Local::now();
    let today = now.date_naive();
    let current_hour = now.hour();
    let current_minute = now.minute();

    // Check if today is in the current week
    let today_column_index = week_state.days.iter().position(|d| *d == today);

    // Check if there's an active timed quick event to display
    let quick_event_data = active_dialog.and_then(|dialog| {
        if let ActiveDialog::QuickEvent { start_date, start_time: Some(start_time), end_time: Some(end_time), text, .. } = dialog {
            Some((*start_date, *start_time, *end_time, text.as_str()))
        } else {
            None
        }
    });

    // Build the grid as a row: time labels column + day columns
    let mut main_row = row().spacing(0);

    // Time labels column
    let time_labels = render_time_labels_column(locale, today_column_index.is_some(), current_hour);
    main_row = main_row.push(time_labels);

    // Day columns with events
    for (day_idx, date) in week_state.days.iter().enumerate() {
        let is_weekend = locale.is_weekend(date.weekday());
        let is_today_column = today_column_index == Some(day_idx);
        let day_events = timed_events.get(date).cloned().unwrap_or_default();

        // Check if this day has the quick event input
        let day_quick_event = quick_event_data.and_then(|(qe_date, start, end, text)| {
            if qe_date == *date {
                Some((start, end, text, calendar_color.unwrap_or("#3B82F6")))
            } else {
                None
            }
        });

        let day_column = render_day_column_with_events(
            *date,
            &day_events,
            is_weekend,
            is_today_column,
            today_column_index.is_some(), // today_in_week - true if today is visible in this week
            current_hour,
            current_minute,
            selected_event_uid,
            selection,
            day_quick_event,
        );

        main_row = main_row.push(day_column);
    }

    main_row.into()
}

/// Render the time labels column (left side)
fn render_time_labels_column<'a>(
    locale: &'a LocalePreferences,
    today_in_view: bool,
    current_hour: u32,
) -> Element<'a, Message> {
    let mut col = column().spacing(0);

    for hour in 0..24 {
        let is_current_hour = today_in_view && hour == current_hour;
        let time_label = locale.format_hour(hour);

        col = col.push(
            container(
                widget::text(time_label)
                    .size(FONT_SIZE_SMALL)
            )
            .width(Length::Fixed(TIME_LABEL_WIDTH))
            .height(Length::Fixed(HOUR_ROW_HEIGHT))
            .padding(PADDING_SMALL)
            .align_y(alignment::Vertical::Top)
            .style(move |_theme: &cosmic::Theme| container::Style {
                text_color: if is_current_hour {
                    Some(COLOR_CURRENT_TIME)
                } else {
                    None
                },
                border: Border {
                    width: BORDER_WIDTH_THIN,
                    color: COLOR_DAY_CELL_BORDER,
                    ..Default::default()
                },
                ..Default::default()
            })
        );
    }

    col.into()
}

/// Render a single day column with events spanning their full duration using stack overlay
fn render_day_column_with_events(
    date: NaiveDate,
    events: &[DisplayEvent],
    is_weekend: bool,
    is_today: bool,
    today_in_week: bool,
    current_hour: u32,
    current_minute: u32,
    selected_event_uid: Option<&str>,
    selection: Option<&SelectionState>,
    quick_event: Option<(NaiveTime, NaiveTime, &str, &str)>, // (start_time, end_time, text, color)
) -> Element<'static, Message> {
    // Build the base hour grid (background layer) - now with click support
    let hour_grid = render_hour_grid_background(date, is_weekend, is_today, today_in_week, current_hour, current_minute, selection);

    // Build quick event input layer if active
    let quick_event_layer = quick_event.map(|(start_time, end_time, text, color)| {
        render_quick_event_input_layer(start_time, end_time, text.to_string(), color.to_string())
    });

    // If no events and no quick event, just return the grid
    if events.is_empty() && quick_event_layer.is_none() {
        return container(hour_grid)
            .width(Length::Fill)
            .into();
    }

    // Calculate column assignments for overlapping events
    let positioned_events = calculate_event_columns(events);
    let max_columns = positioned_events.iter().map(|p| p.total_columns).max().unwrap_or(1).max(1);

    // Build the events overlay layer
    let events_layer = render_events_overlay_layer(date, &positioned_events, max_columns, selected_event_uid);

    // Use stack to overlay events on top of the grid, and quick event on top of that
    let stacked: Element<'static, Message> = if let Some(qe_layer) = quick_event_layer {
        stack![
            hour_grid,
            events_layer,
            qe_layer
        ].into()
    } else {
        stack![
            hour_grid,
            events_layer
        ].into()
    };

    container(stacked)
        .width(Length::Fill)
        .into()
}

/// Render the hour grid background (lines only, no events) with clickable time slots
fn render_hour_grid_background(
    date: NaiveDate,
    is_weekend: bool,
    is_today: bool,
    today_in_week: bool,
    current_hour: u32,
    current_minute: u32,
    selection: Option<&SelectionState>,
) -> Element<'static, Message> {
    let mut hour_cells = column().spacing(0);

    for hour in 0..24u32 {
        // Show time line in all columns if today is in this week and it's the current hour
        let show_time_line = today_in_week && hour == current_hour;
        // Show the dot only on today's column
        let show_time_dot = is_today && hour == current_hour;
        // Check if this hour cell is within the current selection
        let is_selected = selection.map(|s| s.is_active && s.contains_time(date, hour)).unwrap_or(false);

        let cell = if show_time_line {
            let minute_offset = (current_minute as f32 / 60.0) * HOUR_ROW_HEIGHT;
            render_clickable_hour_cell_with_indicator(date, hour, is_weekend, minute_offset, is_selected, show_time_dot)
        } else {
            render_clickable_hour_cell(date, hour, is_weekend, is_selected)
        };

        hour_cells = hour_cells.push(cell);
    }

    hour_cells.into()
}

/// Render the events overlay layer with events positioned based on their time spans
/// Uses a row of columns approach where each column renders its events independently
fn render_events_overlay_layer(
    date: NaiveDate,
    positioned_events: &[PositionedEvent],
    max_columns: usize,
    selected_event_uid: Option<&str>,
) -> Element<'static, Message> {
    // Each column renders its events independently with proper vertical positioning
    // This ensures overlapping events appear side-by-side
    let mut columns_row = row().spacing(1);

    for col_idx in 0..max_columns {
        // Get all events for this column, sorted by start time
        let mut col_events: Vec<&PositionedEvent> = positioned_events.iter()
            .filter(|pe| pe.column == col_idx)
            .collect();
        col_events.sort_by_key(|pe| event_time_range(&pe.event).0);

        // Build this column's content with spacers and events
        let col_content = render_column_events(date, &col_events, selected_event_uid);

        columns_row = columns_row.push(
            container(col_content)
                .width(Length::Fill)
        );
    }

    columns_row.into()
}

/// Render a single column of events with proper vertical spacing
fn render_column_events(
    date: NaiveDate,
    events: &[&PositionedEvent],
    selected_event_uid: Option<&str>,
) -> Element<'static, Message> {
    let mut col = column().spacing(0);
    let mut current_mins: u32 = 0;

    for pe in events {
        let (start_mins, end_mins) = event_time_range(&pe.event);

        // Add spacer to position this event correctly
        if start_mins > current_mins {
            let spacer_height = ((start_mins - current_mins) as f32 / 60.0) * HOUR_ROW_HEIGHT;
            col = col.push(
                container(widget::text(""))
                    .height(Length::Fixed(spacer_height))
                    .width(Length::Fill)
            );
        }

        // Render the event
        let ev_height = ((end_mins - start_mins) as f32 / 60.0) * HOUR_ROW_HEIGHT;
        let event_block = render_positioned_event_block(
            date,
            &pe.event,
            ev_height.max(20.0), // Minimum height for visibility
            selected_event_uid,
        );
        col = col.push(event_block);

        current_mins = end_mins;
    }

    // Fill remaining space to maintain column height
    let total_mins = 24 * 60;
    if current_mins < total_mins {
        let remaining_height = ((total_mins - current_mins) as f32 / 60.0) * HOUR_ROW_HEIGHT;
        col = col.push(
            container(widget::text(""))
                .height(Length::Fixed(remaining_height))
                .width(Length::Fill)
        );
    }

    col.into()
}

/// Render a positioned event block with the specified height
fn render_positioned_event_block(
    date: NaiveDate,
    event: &DisplayEvent,
    height: f32,
    selected_event_uid: Option<&str>,
) -> Element<'static, Message> {
    let color = parse_hex_color(&event.color)
        .unwrap_or(cosmic::iced::Color::from_rgb(0.5, 0.5, 0.5));
    let uid = event.uid.clone();
    let is_selected = selected_event_uid == Some(&event.uid);

    let (bg_opacity, border_width) = if is_selected {
        (0.9, 2.0)
    } else {
        (0.85, 0.0)
    };

    // Build the label with time and summary
    let time_str = event.start_time
        .map(|t| format!("{:02}:{:02}", t.hour(), t.minute()))
        .unwrap_or_default();
    let label = format!("{} {}", time_str, event.summary);

    let chip = container(
        widget::text(label.clone())
            .size(10)
    )
    .padding([2, 4])
    .width(Length::Fill)
    .height(Length::Fixed(height))
    .style(move |theme: &cosmic::Theme| container::Style {
        background: Some(Background::Color(cosmic::iced::Color {
            a: bg_opacity,
            ..color
        })),
        text_color: Some(cosmic::iced::Color::WHITE),
        border: Border {
            radius: BORDER_RADIUS.into(),
            width: border_width,
            color: if is_selected {
                theme.cosmic().accent_color().into()
            } else {
                cosmic::iced::Color::TRANSPARENT
            },
        },
        ..Default::default()
    });

    // Get color hex for drag preview
    let color_hex = event.color.clone();

    mouse_area(chip)
        .on_press(Message::DragEventStart(uid.clone(), date, event.summary.clone(), color_hex))
        .on_release(Message::DragEventEnd)
        .on_double_click(Message::OpenEditEventDialog(uid))
        .on_enter(Message::DragEventUpdate(date))
        .into()
}

/// Get the time range of an event in minutes from midnight
fn event_time_range(event: &DisplayEvent) -> (u32, u32) {
    let start = event.start_time
        .map(|t| t.hour() * 60 + t.minute())
        .unwrap_or(0);
    let end = event.end_time
        .map(|t| t.hour() * 60 + t.minute())
        .unwrap_or(start + 60); // Default 1 hour if no end time

    // Ensure end is after start
    let end = if end <= start { start + 30 } else { end };

    (start, end)
}

/// Render a clickable hour cell (for creating new events and drag targets)
fn render_clickable_hour_cell(date: NaiveDate, hour: u32, is_weekend: bool, is_selected: bool) -> Element<'static, Message> {
    // Create the time for this hour cell
    let start_time = NaiveTime::from_hms_opt(hour, 0, 0).unwrap();
    let _end_time = NaiveTime::from_hms_opt(hour, 59, 59).unwrap_or_else(|| {
        NaiveTime::from_hms_opt(23, 59, 59).unwrap()
    });

    let cell = container(widget::text(""))
        .width(Length::Fill)
        .height(Length::Fixed(HOUR_ROW_HEIGHT))
        .style(move |theme: &cosmic::Theme| {
            let background = if is_selected {
                // Use theme accent color for selection (consistent with month view)
                let accent = theme.cosmic().accent_color();
                Some(Background::Color(cosmic::iced::Color::from_rgba(
                    accent.red, accent.green, accent.blue, 0.2
                )))
            } else if is_weekend {
                Some(Background::Color(COLOR_WEEKEND_BACKGROUND))
            } else {
                None
            };
            container::Style {
                background,
                border: Border {
                    width: BORDER_WIDTH_THIN,
                    color: COLOR_DAY_CELL_BORDER,
                    ..Default::default()
                },
                ..Default::default()
            }
        });

    // Press: start time selection for creating timed events
    // Release: end time selection
    // on_enter: update time selection (for drag selection)
    // Double-click: open new event dialog
    mouse_area(cell)
        .on_press(Message::TimeSelectionStart(date, start_time))
        .on_release(Message::TimeSelectionEnd)
        .on_double_click(Message::OpenNewEventDialog)
        .on_enter(Message::TimeSelectionUpdate(date, start_time))
        .into()
}

/// Render a clickable hour cell with the current time indicator
fn render_clickable_hour_cell_with_indicator(
    date: NaiveDate,
    hour: u32,
    is_weekend: bool,
    minute_offset: f32,
    is_selected: bool,
    show_dot: bool,
) -> Element<'static, Message> {
    // Create the time for this hour cell
    let start_time = NaiveTime::from_hms_opt(hour, 0, 0).unwrap();

    // The line spans full width in all columns
    // Dot is 8px, line is 2px - both centered vertically in an 8px row
    let dot_size = 8.0_f32;
    let line_height = 2.0_f32;

    // Adjust the top spacer to account for the indicator being 8px tall (dot height)
    // We want the CENTER of the indicator (where the line is) to be at minute_offset
    // So we subtract half the dot size from the offset
    let adjusted_offset = (minute_offset - (dot_size / 2.0)).max(0.0);
    let top_spacer = container(widget::text(""))
        .height(Length::Fixed(adjusted_offset))
        .width(Length::Fill);

    // Time indicator - line in all columns, dot overlaid on today's column only
    let time_indicator: Element<'static, Message> = if show_dot {
        // Today's column: dot on left, line filling the rest, vertically centered
        let dot = container(widget::text(""))
            .width(Length::Fixed(dot_size))
            .height(Length::Fixed(dot_size))
            .style(|_theme: &cosmic::Theme| container::Style {
                background: Some(Background::Color(COLOR_CURRENT_TIME)),
                border: Border {
                    radius: 4.0.into(), // dot_size / 2.0
                    ..Default::default()
                },
                ..Default::default()
            });

        // Line with vertical padding to center it within the 8px row height
        let time_line = container(
            container(widget::text(""))
                .width(Length::Fill)
                .height(Length::Fixed(line_height))
                .style(|_theme: &cosmic::Theme| container::Style {
                    background: Some(Background::Color(COLOR_CURRENT_TIME)),
                    ..Default::default()
                })
        )
        .width(Length::Fill)
        .height(Length::Fixed(dot_size))
        .align_y(alignment::Vertical::Center);

        row()
            .spacing(0)
            .align_y(alignment::Vertical::Center)
            .push(dot)
            .push(time_line)
            .into()
    } else {
        // Other columns: just the line, centered vertically in 8px height to match dot columns
        container(
            container(widget::text(""))
                .width(Length::Fill)
                .height(Length::Fixed(line_height))
                .style(|_theme: &cosmic::Theme| container::Style {
                    background: Some(Background::Color(COLOR_CURRENT_TIME)),
                    ..Default::default()
                })
        )
        .width(Length::Fill)
        .height(Length::Fixed(dot_size))
        .align_y(alignment::Vertical::Center)
        .into()
    };

    let content = column()
        .spacing(0)
        .push(top_spacer)
        .push(time_indicator);

    let cell = container(content)
        .width(Length::Fill)
        .height(Length::Fixed(HOUR_ROW_HEIGHT))
        .style(move |theme: &cosmic::Theme| {
            let background = if is_selected {
                // Use theme accent color for selection (consistent with month view)
                let accent = theme.cosmic().accent_color();
                Some(Background::Color(cosmic::iced::Color::from_rgba(
                    accent.red, accent.green, accent.blue, 0.2
                )))
            } else if is_weekend {
                Some(Background::Color(COLOR_WEEKEND_BACKGROUND))
            } else {
                None
            };
            container::Style {
                background,
                border: Border {
                    width: BORDER_WIDTH_THIN,
                    color: COLOR_DAY_CELL_BORDER,
                    ..Default::default()
                },
                ..Default::default()
            }
        });

    // Press: start time selection for creating timed events
    // Release: end time selection
    // on_enter: update time selection (for drag selection)
    // Double-click: open new event dialog
    mouse_area(cell)
        .on_press(Message::TimeSelectionStart(date, start_time))
        .on_release(Message::TimeSelectionEnd)
        .on_double_click(Message::OpenNewEventDialog)
        .on_enter(Message::TimeSelectionUpdate(date, start_time))
        .into()
}

/// Check if two events overlap in time
fn events_overlap(e1: &DisplayEvent, e2: &DisplayEvent) -> bool {
    let Some(start1) = e1.start_time else { return false };
    let Some(end1) = e1.end_time else { return false };
    let Some(start2) = e2.start_time else { return false };
    let Some(end2) = e2.end_time else { return false };

    // Events overlap if one starts before the other ends
    start1 < end2 && start2 < end1
}

/// Calculate column positions for overlapping events
/// Returns events with their assigned column and total columns in their overlap group
fn calculate_event_columns(events: &[DisplayEvent]) -> Vec<PositionedEvent> {
    if events.is_empty() {
        return Vec::new();
    }

    // Sort events by start time, then by end time (shorter events first)
    let mut sorted: Vec<_> = events.iter().cloned().collect();
    sorted.sort_by(|a, b| {
        let start_cmp = a.start_time.cmp(&b.start_time);
        if start_cmp == std::cmp::Ordering::Equal {
            a.end_time.cmp(&b.end_time)
        } else {
            start_cmp
        }
    });

    let mut positioned: Vec<PositionedEvent> = Vec::new();
    let mut column_ends: Vec<NaiveTime> = Vec::new(); // Track when each column becomes free

    for event in sorted {
        let start = event.start_time.unwrap_or(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
        let end = event.end_time.unwrap_or(NaiveTime::from_hms_opt(23, 59, 59).unwrap());

        // Find the first column where this event can fit (column is free before this event starts)
        let mut assigned_column = None;
        for (col_idx, col_end) in column_ends.iter_mut().enumerate() {
            if *col_end <= start {
                // This column is free, use it
                *col_end = end;
                assigned_column = Some(col_idx);
                break;
            }
        }

        // If no existing column is free, create a new one
        let column = assigned_column.unwrap_or_else(|| {
            column_ends.push(end);
            column_ends.len() - 1
        });

        positioned.push(PositionedEvent {
            event,
            column,
            total_columns: 0, // Will be set in second pass
        });
    }

    // Second pass: for each event, find the max columns in its overlap group
    for i in 0..positioned.len() {
        let mut max_col = positioned[i].column;

        // Check all events that overlap with this one
        for j in 0..positioned.len() {
            if i != j && events_overlap(&positioned[i].event, &positioned[j].event) {
                max_col = max_col.max(positioned[j].column);
            }
        }

        positioned[i].total_columns = max_col + 1;
    }

    positioned
}

/// Render the quick event input overlay layer for timed event creation
/// Positions the input at the correct time slot and spans the selected duration
fn render_quick_event_input_layer(
    start_time: NaiveTime,
    end_time: NaiveTime,
    text: String,
    calendar_color: String,
) -> Element<'static, Message> {
    // Calculate position and height based on time range
    let start_mins = start_time.hour() * 60 + start_time.minute();
    let end_mins = end_time.hour() * 60 + end_time.minute();

    // Ensure minimum duration and proper order
    let (start_mins, end_mins) = if start_mins <= end_mins {
        (start_mins, end_mins.max(start_mins + 30)) // Minimum 30 min
    } else {
        (end_mins, start_mins.max(end_mins + 30))
    };

    let top_offset = (start_mins as f32 / 60.0) * HOUR_ROW_HEIGHT;
    let height = ((end_mins - start_mins) as f32 / 60.0) * HOUR_ROW_HEIGHT;
    let height = height.max(HOUR_ROW_HEIGHT); // Minimum height of 1 hour cell

    let color = parse_hex_color(&calendar_color)
        .unwrap_or(cosmic::iced::Color::from_rgb(0.23, 0.51, 0.97));

    // Create text input for the event title
    let input = text_input("New event...", &text)
        .id(quick_event_input_id())
        .on_input(Message::QuickEventTextChanged)
        .on_submit(Message::CommitQuickEvent)
        .size(12)
        .padding([4, 6])
        .width(Length::Fill);

    // Style the container with calendar color
    let input_container = container(input)
        .width(Length::Fill)
        .height(Length::Fixed(height))
        .padding([2, 4])
        .style(move |_theme: &cosmic::Theme| container::Style {
            background: Some(Background::Color(cosmic::iced::Color {
                a: 0.3,
                ..color
            })),
            border: Border {
                color,
                width: 2.0,
                radius: BORDER_RADIUS.into(),
            },
            ..Default::default()
        });

    // Create spacer to position the input at the correct time
    let top_spacer = container(widget::text(""))
        .height(Length::Fixed(top_offset))
        .width(Length::Fill);

    // Build column with spacer and input
    column()
        .spacing(0)
        .push(top_spacer)
        .push(input_container)
        .width(Length::Fill)
        .into()
}
