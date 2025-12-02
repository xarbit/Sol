//! Month view overlay rendering
//!
//! Contains slot computation and date event overlay rendering logic.

use std::collections::HashMap;

use chrono::NaiveDate;
use cosmic::iced::Length;
use cosmic::widget::{column, container, row};
use cosmic::Element;

use crate::components::spacer::{fill_spacer, horizontal_spacer, spacer, vertical_spacer};
use crate::components::DisplayEvent;
use crate::message::Message;
use crate::models::CalendarDay;
use crate::ui_constants::{
    COMPACT_EVENT_HEIGHT, DATE_EVENT_HEIGHT, DATE_EVENT_SPACING,
    DAY_CELL_HEADER_OFFSET, DAY_CELL_TOP_PADDING, PADDING_MONTH_GRID,
    SPACING_TINY, WEEK_NUMBER_WIDTH,
};

use super::events::{render_compact_date_event_chip, render_date_event_chip};

/// Fixed height for the weekday header row
pub const WEEKDAY_HEADER_HEIGHT: f32 = 32.0;

/// A segment of a date event (no specific time) to render in the overlay.
/// For single-day events, this represents the entire event.
/// For multi-day events, this represents one week's portion.
#[derive(Debug, Clone)]
pub struct DateEventSegment {
    /// Calendar ID for unique event identification
    pub calendar_id: String,
    /// Event UID for click/drag handling
    pub uid: String,
    /// Event summary/title
    pub summary: String,
    /// Event color (hex string)
    pub color: String,
    /// Week index (0-based)
    pub week_idx: usize,
    /// Slot index within the week (for vertical stacking)
    pub slot: usize,
    /// Start column (0-6)
    pub start_col: usize,
    /// End column (0-6)
    pub end_col: usize,
    /// Whether this is the first segment (shows text)
    /// For single-day events, this is always true
    pub is_first_segment: bool,
    /// The event's start date (for drag operations)
    pub event_start_date: NaiveDate,
    /// The date of the last day this segment covers (used for past event dimming)
    pub segment_end_date: NaiveDate,
}

/// Result of computing slot assignments for a week.
/// Contains both the event-to-slot mapping and per-day occupancy info.
pub struct WeekSlotInfo {
    /// Map of event unique ID (calendar_id:uid) -> slot index
    pub slots: HashMap<String, usize>,
    /// Occupied slots for each day (column) in the week: [day_0, day_1, ..., day_6]
    /// Each set contains the slot indices that are occupied by date events on that day
    pub day_occupied_slots: Vec<std::collections::HashSet<usize>>,
}

/// Compute slot assignments for all date events in a week using greedy interval scheduling.
/// Returns both the event-to-slot mapping and per-day slot occupancy.
/// Both single-day and multi-day date events get slots assigned.
/// Events are assigned to the first available slot where they don't overlap with other events.
pub fn compute_week_date_event_slots(
    week: &[CalendarDay],
    events_by_date: &HashMap<NaiveDate, Vec<DisplayEvent>>,
) -> WeekSlotInfo {
    let mut slots: HashMap<String, usize> = HashMap::new();
    let mut slot_occupancy: Vec<std::collections::HashSet<usize>> = vec![std::collections::HashSet::new(); 7];

    // Get dates for this week
    let week_dates: Vec<NaiveDate> = week
        .iter()
        .filter_map(|d| NaiveDate::from_ymd_opt(d.year, d.month, d.day))
        .collect();

    if week_dates.is_empty() {
        return WeekSlotInfo { slots, day_occupied_slots: slot_occupancy };
    }

    let week_start = week_dates[0];
    let week_end = week_dates[week_dates.len() - 1];

    // Collect all date events that appear in this week
    // Store: (start_col, end_col, unique_id) - column range within this week
    let mut date_events: Vec<(usize, usize, String)> = Vec::new();
    let mut seen_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

    for (col, date) in week_dates.iter().enumerate() {
        if let Some(day_events) = events_by_date.get(date) {
            for event in day_events {
                let unique_id = event.unique_id();
                if !event.all_day || seen_ids.contains(&unique_id) {
                    continue;
                }

                seen_ids.insert(unique_id.clone());

                // Determine column range within this week
                let (start_col, end_col) = if event.is_multi_day() {
                    match (event.span_start, event.span_end) {
                        (Some(s), Some(e)) if s <= week_end && e >= week_start => {
                            // Calculate column range clipped to this week
                            let sc = week_dates.iter()
                                .position(|&d| d >= s)
                                .unwrap_or(0);
                            let ec = week_dates.iter()
                                .rposition(|&d| d <= e)
                                .unwrap_or(6);
                            (sc, ec)
                        },
                        _ => continue,
                    }
                } else {
                    // Single-day event - only spans its own column
                    (col, col)
                };

                date_events.push((start_col, end_col, unique_id));
            }
        }
    }

    // Sort by start column, then by span length (longer events first for stable ordering)
    date_events.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then_with(|| (b.1 - b.0).cmp(&(a.1 - a.0))) // Longer events first
    });

    // Greedy interval scheduling: assign each event to the first available slot
    // Track which slots are occupied at each column: slot_occupancy[col] = set of occupied slots
    for (start_col, end_col, uid) in date_events {
        // Find the first slot that is free for all columns this event spans
        let mut slot = 0;
        loop {
            let mut slot_available = true;
            for col in start_col..=end_col.min(6) {
                if slot_occupancy[col].contains(&slot) {
                    slot_available = false;
                    break;
                }
            }
            if slot_available {
                break;
            }
            slot += 1;
        }

        // Mark this slot as occupied for all columns the event spans
        for col in start_col..=end_col.min(6) {
            slot_occupancy[col].insert(slot);
        }

        slots.insert(uid, slot);
    }

    WeekSlotInfo { slots, day_occupied_slots: slot_occupancy }
}

/// Compute slot assignments for all date events in a week (used by day_cell for placeholders).
/// Returns the full slot info including per-day occupancy for Tetris-style rendering.
/// Uses the same greedy interval scheduling as the overlay renderer for consistency.
pub fn compute_week_event_slots(
    week: &[CalendarDay],
    events_by_date: &HashMap<NaiveDate, Vec<DisplayEvent>>,
) -> WeekSlotInfo {
    // Use the same algorithm as the overlay to ensure consistent slot assignments
    compute_week_date_event_slots(week, events_by_date)
}

/// Collect all date event segments across all weeks for overlay rendering.
/// Both single-day and multi-day date events are handled.
/// Each segment represents one week's portion of an event (or the whole event for single-day).
pub fn collect_date_event_segments(
    weeks: &[Vec<CalendarDay>],
    events_by_date: &HashMap<NaiveDate, Vec<DisplayEvent>>,
) -> Vec<DateEventSegment> {
    let mut segments: Vec<DateEventSegment> = Vec::new();
    let mut global_seen: HashMap<String, bool> = HashMap::new(); // uid -> has_been_first

    for (week_idx, week) in weeks.iter().enumerate() {
        // Get week date range
        let week_dates: Vec<NaiveDate> = week
            .iter()
            .filter_map(|d| NaiveDate::from_ymd_opt(d.year, d.month, d.day))
            .collect();

        if week_dates.is_empty() {
            continue;
        }

        let week_start = week_dates[0];
        let week_end = week_dates[week_dates.len() - 1];

        // Compute slots for this week (all date events)
        let week_slot_info = compute_week_date_event_slots(week, events_by_date);

        // Find date events in this week
        let mut week_seen: std::collections::HashSet<String> = std::collections::HashSet::new();

        for (col, date) in week_dates.iter().enumerate() {
            if let Some(day_events) = events_by_date.get(date) {
                for event in day_events {
                    let unique_id = event.unique_id();
                    // Only process date events we haven't seen this week
                    if !event.all_day || week_seen.contains(&unique_id) {
                        continue;
                    }

                    week_seen.insert(unique_id);

                    // Determine start/end columns for this event in this week
                    // Also capture the event's start date for drag operations
                    let (start_col, end_col, event_start_date) = if event.is_multi_day() {
                        let (Some(span_start), Some(span_end)) = (event.span_start, event.span_end) else {
                            continue;
                        };

                        // Check if event overlaps this week
                        if span_start > week_end || span_end < week_start {
                            continue;
                        }

                        // Calculate column range for this week
                        let sc = week_dates.iter()
                            .position(|&d| d >= span_start)
                            .unwrap_or(0);
                        let ec = week_dates.iter()
                            .rposition(|&d| d <= span_end)
                            .unwrap_or(6);
                        (sc, ec, span_start)
                    } else {
                        // Single-day event: only spans its own column
                        (col, col, *date)
                    };

                    // Determine if this is the first segment for this event
                    let unique_id = event.unique_id();
                    let is_first_segment = !global_seen.contains_key(&unique_id);
                    global_seen.insert(unique_id.clone(), true);

                    // Get slot for this event
                    let slot = week_slot_info.slots.get(&unique_id).copied().unwrap_or(0);

                    // Get the actual date for the segment's end column (used for past event dimming)
                    let segment_end_date = week_dates[end_col];

                    segments.push(DateEventSegment {
                        calendar_id: event.calendar_id.clone(),
                        uid: event.uid.clone(),
                        summary: event.summary.clone(),
                        color: event.color.clone(),
                        week_idx,
                        slot,
                        start_col,
                        end_col,
                        is_first_segment,
                        event_start_date,
                        segment_end_date,
                    });
                }
            }
        }
    }

    segments
}

/// Render the date events overlay layer.
/// This renders all date events (single-day and multi-day) as spanning elements.
/// Single-day events span only their own column.
/// Multi-day events span across multiple columns.
///
/// # Arguments
/// * `weeks` - The weeks of the month
/// * `events_by_date` - Events grouped by date
/// * `show_week_numbers` - Whether week numbers column is visible
/// * `compact` - If true, render thin colored lines instead of full event chips
/// * `selected_event_uid` - Currently selected event UID for visual feedback
/// * `event_drag_active` - Whether an event drag operation is currently active
/// * `dragging_event_uid` - UID of the event currently being dragged
pub fn render_date_events_overlay<'a>(
    weeks: &[Vec<CalendarDay>],
    events_by_date: &HashMap<NaiveDate, Vec<DisplayEvent>>,
    show_week_numbers: bool,
    compact: bool,
    selected_event_uid: Option<&str>,
    event_drag_active: bool,
    dragging_event_uid: Option<&str>,
) -> Option<Element<'a, Message>> {
    let segments = collect_date_event_segments(weeks, events_by_date);

    if segments.is_empty() {
        return None;
    }

    let num_weeks = weeks.len();

    // Group segments by week
    let mut segments_by_week: HashMap<usize, Vec<DateEventSegment>> = HashMap::new();
    for segment in segments {
        segments_by_week
            .entry(segment.week_idx)
            .or_default()
            .push(segment);
    }

    // Use appropriate height based on compact mode
    let event_height = if compact { COMPACT_EVENT_HEIGHT } else { DATE_EVENT_HEIGHT };

    // Build overlay with same structure as main grid
    let mut overlay_column = column()
        .spacing(SPACING_TINY)
        .padding(PADDING_MONTH_GRID);

    // Header spacer
    overlay_column = overlay_column.push(vertical_spacer(WEEKDAY_HEADER_HEIGHT));

    for week_idx in 0..num_weeks {
        let week_segments = segments_by_week.get(&week_idx);

        if let Some(segs) = week_segments {
            // Find max slot for this week
            let max_slot = segs.iter().map(|s| s.slot).max().unwrap_or(0);

            // Build week content: header offset + slot rows
            let mut week_content = column().spacing(DATE_EVENT_SPACING);

            // Spacer for day header area
            week_content = week_content.push(vertical_spacer(DAY_CELL_HEADER_OFFSET + DAY_CELL_TOP_PADDING));

            // Render each slot as a separate row
            for slot in 0..=max_slot {
                // Find segments at this slot
                let slot_segments: Vec<&DateEventSegment> = segs.iter()
                    .filter(|s| s.slot == slot)
                    .collect();

                // Build row for this slot
                let mut slot_row = row().spacing(SPACING_TINY).height(Length::Fixed(event_height));

                // Sort segments by start_col to process them in order
                let mut sorted_segs = slot_segments.clone();
                sorted_segs.sort_by_key(|s| s.start_col);

                let mut current_col = 0;

                for seg in sorted_segs {
                    // Add spacers for empty columns before this segment
                    if seg.start_col > current_col {
                        let gap = seg.start_col - current_col;
                        for _ in 0..gap {
                            slot_row = slot_row.push(spacer(Length::Fill, Length::Shrink));
                        }
                    }

                    // Render the spanning chip (full or compact based on mode)
                    let span_cols = seg.end_col - seg.start_col + 1;
                    let unique_id = format!("{}:{}", seg.calendar_id, seg.uid);
                    let is_selected = selected_event_uid == Some(unique_id.as_str());
                    let is_being_dragged = dragging_event_uid == Some(unique_id.as_str());
                    let chip = if compact {
                        render_compact_date_event_chip(
                            seg.color.clone(),
                            seg.start_col == 0,
                            seg.end_col == 6,
                        )
                    } else {
                        render_date_event_chip(
                            seg.calendar_id.clone(),
                            seg.uid.clone(),
                            seg.summary.clone(),
                            seg.color.clone(),
                            seg.is_first_segment,
                            seg.start_col == 0,
                            seg.end_col == 6,
                            is_selected,
                            seg.event_start_date,
                            event_drag_active,
                            is_being_dragged,
                            seg.segment_end_date,
                        )
                    };

                    slot_row = slot_row.push(
                        container(chip)
                            .width(Length::FillPortion(span_cols as u16))
                    );

                    current_col = seg.end_col + 1;
                }

                // Add spacers for empty columns after the last segment
                if current_col < 7 {
                    for _ in current_col..7 {
                        slot_row = slot_row.push(spacer(Length::Fill, Length::Shrink));
                    }
                }

                week_content = week_content.push(slot_row);
            }

            // Build the week row with week number spacer
            let mut week_row = row().spacing(SPACING_TINY).height(Length::Fill);

            if show_week_numbers {
                week_row = week_row.push(horizontal_spacer(WEEK_NUMBER_WIDTH));
            }

            // The week content takes up the rest of the space
            week_row = week_row.push(
                container(week_content)
                    .width(Length::Fill)
            );

            overlay_column = overlay_column.push(week_row);
        } else {
            // Empty week row
            overlay_column = overlay_column.push(fill_spacer());
        }
    }

    Some(
        container(overlay_column)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    )
}
