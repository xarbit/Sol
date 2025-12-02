//! Clickable event chip wrapper
//!
//! Wraps event chips with mouse interaction for selection and dragging.

use chrono::{Local, NaiveDate};
use cosmic::widget::mouse_area;
use cosmic::Element;

use crate::components::color_picker::parse_hex_color;
use crate::message::Message;
use crate::ui_constants::COLOR_DEFAULT_GRAY;

use super::all_day::render_all_day_chip;
use super::timed::render_timed_event_chip;
use super::types::{ChipSelectionState, DisplayEvent};

/// Render a small event chip showing the event title with calendar color
/// For all-day events: colored background bar with span-aware corners
/// For timed events: colored dot + time + name
///
/// # Arguments
/// * `event` - The display event with span metadata
/// * `current_date` - The date of the cell being rendered (for span position calculation)
#[allow(dead_code)]
pub fn render_event_chip(event: DisplayEvent, current_date: NaiveDate) -> Element<'static, Message> {
    let color = parse_hex_color(&event.color).unwrap_or(COLOR_DEFAULT_GRAY);

    // Check if this event is in the past
    let is_past = is_event_past(&event, current_date);

    if event.all_day {
        // Calculate span position for multi-day events
        let span_position = event.span_position_for_date(current_date);
        render_all_day_chip(event.summary, color, span_position, None)
    } else {
        render_timed_event_chip(event.summary, event.start_time, color, None, is_past)
    }
}

/// Check if a timed event is in the past (considers time on today)
fn is_event_past(event: &DisplayEvent, current_date: NaiveDate) -> bool {
    let now = Local::now();
    let today = now.date_naive();

    if current_date < today {
        // Past day - always dim
        true
    } else if current_date == today && !event.all_day {
        // Today's timed event - check if event end time has passed
        if let Some(end_time) = event.end_time {
            let current_time = now.time();
            end_time <= current_time
        } else if let Some(start_time) = event.start_time {
            // No end time - use start time as heuristic
            let current_time = now.time();
            start_time <= current_time
        } else {
            false
        }
    } else {
        // Future day or all-day event on today
        false
    }
}

/// Render a clickable event chip with selection state and drag support
/// Wraps the event chip with mouse interaction for selection and dragging
///
/// # Arguments
/// * `event` - The display event with span metadata
/// * `current_date` - The date of the cell being rendered
/// * `is_selected` - Whether this event is currently selected
/// * `is_drag_active` - Whether any event drag is currently active
/// * `is_being_dragged` - Whether this specific event is currently being dragged (for dimming)
pub fn render_clickable_event_chip(
    event: DisplayEvent,
    current_date: NaiveDate,
    is_selected: bool,
    is_drag_active: bool,
    is_being_dragged: bool,
) -> Element<'static, Message> {
    let calendar_id = event.calendar_id.clone();
    let uid = event.uid.clone();
    let color = parse_hex_color(&event.color).unwrap_or(COLOR_DEFAULT_GRAY);
    // Clone summary and color_hex for the drag preview message (before they're moved into chip)
    let drag_summary = event.summary.clone();
    let drag_color = event.color.clone();

    // Check if this event is in the past
    let is_past = is_event_past(&event, current_date);

    let selection = Some(ChipSelectionState::new(is_selected, is_being_dragged));

    let chip = if event.all_day {
        let span_position = event.span_position_for_date(current_date);
        render_all_day_chip(event.summary, color, span_position, selection)
    } else {
        render_timed_event_chip(event.summary, event.start_time, color, selection, is_past)
    };

    // Wrap with mouse area for click/drag handling
    // - on_press: Start drag (will be resolved as select or move on release)
    // - on_release: End drag (complete move or select if no movement)
    // - on_double_click: Open edit dialog
    // Pass calendar_id, summary and color for the floating drag preview
    let mut area = mouse_area(chip)
        .on_press(Message::DragEventStart(calendar_id.clone(), uid.clone(), current_date, drag_summary, drag_color))
        .on_release(Message::DragEventEnd)
        .on_double_click(Message::OpenEditEventDialog(calendar_id, uid));

    // Only track mouse enter during active drag for performance
    if is_drag_active {
        area = area.on_enter(Message::DragEventUpdate(current_date));
    }

    area.into()
}
