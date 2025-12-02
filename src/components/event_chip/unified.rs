//! Unified events rendering
//!
//! Renders events as a unified column with placeholders and timed events.
//! Supports Tetris-style slot filling for optimal space usage.

use chrono::NaiveDate;
use cosmic::iced::Length;
use cosmic::widget::{column, container};
use cosmic::{widget, Element};

use crate::message::Message;
use crate::ui_constants::{DATE_EVENT_HEIGHT, DATE_EVENT_SPACING};

use super::clickable::render_clickable_event_chip;
use super::types::DisplayEvent;

/// Result containing a unified events column with placeholders and timed events
pub struct UnifiedEventsResult {
    /// Single column containing placeholders (for date events) followed by timed events
    pub events: Option<Element<'static, Message>>,
    /// Number of events not shown (for "+N more" indicator)
    pub overflow_count: usize,
}

/// Render an empty placeholder to maintain slot alignment
/// This creates an invisible spacer with the same height as an overlay event row
fn render_empty_slot_placeholder() -> Element<'static, Message> {
    container(widget::text(""))
        .width(Length::Fill)
        .height(Length::Fixed(DATE_EVENT_HEIGHT))
        .into()
}

/// Render events as a unified column: placeholders for date events followed by timed events.
/// This ensures timed events always appear BELOW the overlay date events.
///
/// The key insight: the overlay positions date events at a fixed offset from the cell top.
/// We render invisible placeholders of the exact same height to push timed events down.
///
/// # Arguments
/// * `events` - Events to render
/// * `max_visible` - Maximum number of events to show
/// * `current_date` - The date of the cell
/// * `week_max_slot` - Maximum slot index for the week (determines placeholder count)
#[allow(dead_code)]
pub fn render_unified_events(
    events: Vec<DisplayEvent>,
    max_visible: usize,
    current_date: NaiveDate,
    week_max_slot: Option<usize>,
) -> UnifiedEventsResult {
    // Use empty set for day_occupied_slots - this legacy function doesn't do Tetris-style rendering
    let empty_slots = std::collections::HashSet::new();
    render_unified_events_with_selection(events, max_visible, current_date, week_max_slot, &empty_slots, None, false, None)
}

/// Render events as a unified column with selection support.
/// Timed events are rendered with click handlers and visual feedback for selection.
/// Uses Tetris-style slot filling: timed events fill empty slots where date events aren't present.
///
/// # Arguments
/// * `events` - Events to render
/// * `max_visible` - Maximum number of events to show
/// * `current_date` - The date of the cell
/// * `week_max_slot` - Maximum slot index for the week (determines total slot count)
/// * `day_occupied_slots` - Slots occupied by date events on THIS specific day
/// * `selected_event_uid` - UID of the currently selected event (if any)
/// * `dragging_event_uid` - UID of the event currently being dragged (if any)
pub fn render_unified_events_with_selection(
    events: Vec<DisplayEvent>,
    max_visible: usize,
    current_date: NaiveDate,
    week_max_slot: Option<usize>,
    day_occupied_slots: &std::collections::HashSet<usize>,
    selected_event_uid: Option<&str>,
    is_drag_active: bool,
    dragging_event_uid: Option<&str>,
) -> UnifiedEventsResult {
    // Separate all-day and timed events
    let (all_day_events, mut timed_events): (Vec<_>, Vec<_>) =
        events.into_iter().partition(|e| e.all_day);

    // Sort timed events by start time
    timed_events.sort_by(|a, b| a.start_time.cmp(&b.start_time));

    // Calculate total slots from week_max_slot
    // The overlay renders slots 0..=max_slot
    let total_slots = week_max_slot.map(|m| m + 1).unwrap_or(0);

    // Count total events for overflow
    let actual_date_events = all_day_events.len();
    let total_events = actual_date_events + timed_events.len();

    // Build a single column with consistent DATE_EVENT_SPACING to match overlay
    let mut col = column().spacing(DATE_EVENT_SPACING as u16);
    let mut shown = 0;

    // Track which timed events we've used
    let mut timed_event_iter = timed_events.into_iter().peekable();

    // Tetris-style rendering: for each slot position, either:
    // - Show a placeholder if the slot is occupied by a date event (overlay renders it)
    // - Show a timed event if slot is empty (fill the gap)
    for slot in 0..total_slots {
        if shown >= max_visible {
            break;
        }

        if day_occupied_slots.contains(&slot) {
            // Slot is occupied by a date event - render placeholder
            col = col.push(render_empty_slot_placeholder());
        } else {
            // Slot is empty - fill with a timed event if available
            if let Some(event) = timed_event_iter.next() {
                let event_unique_id = event.unique_id();
                let is_selected = selected_event_uid.map_or(false, |uid| uid == event_unique_id);
                let is_being_dragged = dragging_event_uid.map_or(false, |uid| uid == event_unique_id);
                col = col.push(render_clickable_event_chip(event, current_date, is_selected, is_drag_active, is_being_dragged));
            } else {
                // No more timed events - render placeholder to maintain slot alignment
                col = col.push(render_empty_slot_placeholder());
            }
        }
        shown += 1;
    }

    // Render any remaining timed events that didn't fit in empty slots
    for event in timed_event_iter {
        if shown >= max_visible {
            break;
        }
        let event_unique_id = event.unique_id();
        let is_selected = selected_event_uid.map_or(false, |uid| uid == event_unique_id);
        let is_being_dragged = dragging_event_uid.map_or(false, |uid| uid == event_unique_id);
        col = col.push(render_clickable_event_chip(event, current_date, is_selected, is_drag_active, is_being_dragged));
        shown += 1;
    }

    let overflow_count = if total_events > max_visible {
        total_events - max_visible
    } else {
        0
    };

    let events = if total_slots > 0 || shown > 0 {
        Some(col.into())
    } else {
        None
    };

    UnifiedEventsResult {
        events,
        overflow_count,
    }
}
