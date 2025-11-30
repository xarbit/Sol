use std::collections::HashMap;

use chrono::{Datelike, NaiveDate};
use cosmic::iced::widget::stack;
use cosmic::iced::widget::text::Wrapping;
use cosmic::iced::{alignment, Length, Size};
use cosmic::widget::{column, container, mouse_area, row, responsive};
use cosmic::{widget, Element};

use crate::components::color_picker::parse_hex_color;
use crate::components::{render_day_cell_with_events, render_spanning_quick_event_input, DayCellConfig, DisplayEvent};
use crate::dialogs::ActiveDialog;
use crate::models::CalendarDay;
use crate::fl;
use crate::locale::LocalePreferences;
use crate::localized_names;
use crate::message::Message;
use crate::models::CalendarState;
use crate::selection::SelectionState;
use crate::components::should_use_compact;
use crate::ui_constants::{
    FONT_SIZE_MEDIUM, FONT_SIZE_SMALL, PADDING_SMALL, PADDING_MONTH_GRID,
    SPACING_TINY, WEEK_NUMBER_WIDTH, COLOR_DEFAULT_GRAY,
    DATE_EVENT_HEIGHT, COMPACT_EVENT_HEIGHT, DATE_EVENT_SPACING,
    DAY_CELL_HEADER_OFFSET, DAY_CELL_TOP_PADDING, BORDER_WIDTH_HIGHLIGHT,
};

/// Height of the spanning quick event input overlay
const SPANNING_INPUT_HEIGHT: f32 = 36.0;

/// Minimum width per day cell to use full weekday names
/// Below this threshold, short names are used
const MIN_CELL_WIDTH_FOR_FULL_NAMES: f32 = 100.0;

/// Fixed height for the weekday header row
const WEEKDAY_HEADER_HEIGHT: f32 = 32.0;

/// A segment of a date event (no specific time) to render in the overlay.
/// For single-day events, this represents the entire event.
/// For multi-day events, this represents one week's portion.
#[derive(Debug, Clone)]
struct DateEventSegment {
    /// Event UID for click/drag handling
    uid: String,
    /// Event summary/title
    summary: String,
    /// Event color (hex string)
    color: String,
    /// Week index (0-based)
    week_idx: usize,
    /// Slot index within the week (for vertical stacking)
    slot: usize,
    /// Start column (0-6)
    start_col: usize,
    /// End column (0-6)
    end_col: usize,
    /// Whether this is the first segment (shows text)
    /// For single-day events, this is always true
    is_first_segment: bool,
    /// The event's start date (for drag operations)
    event_start_date: NaiveDate,
}

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
    /// Currently selected event UID (for visual feedback)
    pub selected_event_uid: Option<&'a str>,
    /// Whether an event drag operation is currently active
    pub event_drag_active: bool,
    /// The UID of the event currently being dragged (for dimming original)
    pub dragging_event_uid: Option<&'a str>,
    /// The current drop target date during drag (for highlighting target cell)
    pub drag_target_date: Option<NaiveDate>,
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

/// Result of computing slot assignments for a week.
/// Contains both the event-to-slot mapping and per-day occupancy info.
struct WeekSlotInfo {
    /// Map of event UID -> slot index
    slots: HashMap<String, usize>,
    /// Occupied slots for each day (column) in the week: [day_0, day_1, ..., day_6]
    /// Each set contains the slot indices that are occupied by date events on that day
    day_occupied_slots: Vec<std::collections::HashSet<usize>>,
}

/// Compute slot assignments for all date events in a week using greedy interval scheduling.
/// Returns both the event-to-slot mapping and per-day slot occupancy.
/// Both single-day and multi-day date events get slots assigned.
/// Events are assigned to the first available slot where they don't overlap with other events.
fn compute_week_date_event_slots(
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
    // Store: (start_col, end_col, uid) - column range within this week
    let mut date_events: Vec<(usize, usize, String)> = Vec::new();
    let mut seen_uids: std::collections::HashSet<String> = std::collections::HashSet::new();

    for (col, date) in week_dates.iter().enumerate() {
        if let Some(day_events) = events_by_date.get(date) {
            for event in day_events {
                if !event.all_day || seen_uids.contains(&event.uid) {
                    continue;
                }

                seen_uids.insert(event.uid.clone());

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

                date_events.push((start_col, end_col, event.uid.clone()));
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
fn compute_week_event_slots(
    week: &[CalendarDay],
    events_by_date: &HashMap<NaiveDate, Vec<DisplayEvent>>,
) -> WeekSlotInfo {
    // Use the same algorithm as the overlay to ensure consistent slot assignments
    compute_week_date_event_slots(week, events_by_date)
}

/// Collect all date event segments across all weeks for overlay rendering.
/// Both single-day and multi-day date events are handled.
/// Each segment represents one week's portion of an event (or the whole event for single-day).
fn collect_date_event_segments(
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
                    // Only process date events we haven't seen this week
                    if !event.all_day || week_seen.contains(&event.uid) {
                        continue;
                    }

                    week_seen.insert(event.uid.clone());

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
                    let is_first_segment = !global_seen.contains_key(&event.uid);
                    global_seen.insert(event.uid.clone(), true);

                    // Get slot for this event
                    let slot = week_slot_info.slots.get(&event.uid).copied().unwrap_or(0);

                    segments.push(DateEventSegment {
                        uid: event.uid.clone(),
                        summary: event.summary.clone(),
                        color: event.color.clone(),
                        week_idx,
                        slot,
                        start_col,
                        end_col,
                        is_first_segment,
                        event_start_date,
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
fn render_date_events_overlay<'a>(
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
    overlay_column = overlay_column.push(
        container(widget::text(""))
            .height(Length::Fixed(WEEKDAY_HEADER_HEIGHT))
    );

    for week_idx in 0..num_weeks {
        let week_segments = segments_by_week.get(&week_idx);

        if let Some(segs) = week_segments {
            // Find max slot for this week
            let max_slot = segs.iter().map(|s| s.slot).max().unwrap_or(0);

            // Build week content: header offset + slot rows
            let mut week_content = column().spacing(DATE_EVENT_SPACING);

            // Spacer for day header area
            week_content = week_content.push(
                container(widget::text(""))
                    .height(Length::Fixed(DAY_CELL_HEADER_OFFSET + DAY_CELL_TOP_PADDING))
            );

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
                            slot_row = slot_row.push(
                                container(widget::text(""))
                                    .width(Length::Fill)
                            );
                        }
                    }

                    // Render the spanning chip (full or compact based on mode)
                    let span_cols = seg.end_col - seg.start_col + 1;
                    let is_selected = selected_event_uid == Some(seg.uid.as_str());
                    let is_being_dragged = dragging_event_uid == Some(seg.uid.as_str());
                    let chip = if compact {
                        render_compact_date_event_chip(
                            seg.color.clone(),
                            seg.start_col == 0,
                            seg.end_col == 6,
                        )
                    } else {
                        render_date_event_chip(
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
                        slot_row = slot_row.push(
                            container(widget::text(""))
                                .width(Length::Fill)
                        );
                    }
                }

                week_content = week_content.push(slot_row);
            }

            // Build the week row with week number spacer
            let mut week_row = row().spacing(SPACING_TINY).height(Length::Fill);

            if show_week_numbers {
                week_row = week_row.push(
                    container(widget::text(""))
                        .width(Length::Fixed(WEEK_NUMBER_WIDTH))
                );
            }

            // The week content takes up the rest of the space
            week_row = week_row.push(
                container(week_content)
                    .width(Length::Fill)
            );

            overlay_column = overlay_column.push(week_row);
        } else {
            // Empty week row
            overlay_column = overlay_column.push(
                container(widget::text(""))
                    .height(Length::Fill)
            );
        }
    }

    Some(
        container(overlay_column)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    )
}

/// Render a compact date event chip (thin colored line without text)
/// Used when cell size is too small for full event chips
fn render_compact_date_event_chip(
    color_hex: String,
    is_event_start: bool,
    is_event_end: bool,
) -> Element<'static, Message> {
    let color = parse_hex_color(&color_hex).unwrap_or(COLOR_DEFAULT_GRAY);

    // Smaller radius for compact mode
    let radius = 2.0;
    let border_radius: [f32; 4] = match (is_event_start, is_event_end) {
        (true, true) => [radius, radius, radius, radius],
        (true, false) => [radius, 0.0, 0.0, radius],
        (false, true) => [0.0, radius, radius, 0.0],
        (false, false) => [0.0, 0.0, 0.0, 0.0],
    };

    container(widget::text(""))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme: &cosmic::Theme| {
            container::Style {
                background: Some(cosmic::iced::Background::Color(color.scale_alpha(0.6))),
                border: cosmic::iced::Border {
                    color: cosmic::iced::Color::TRANSPARENT,
                    width: 0.0,
                    radius: border_radius.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

/// Render a single date event chip for the overlay.
/// Works for both single-day and multi-day date events.
/// Includes click/drag handling for event selection and movement.
fn render_date_event_chip(
    uid: String,
    summary: String,
    color_hex: String,
    show_text: bool,
    is_event_start: bool,
    is_event_end: bool,
    is_selected: bool,
    event_start_date: NaiveDate,
    is_drag_active: bool,
    is_being_dragged: bool,
) -> Element<'static, Message> {
    let color = parse_hex_color(&color_hex).unwrap_or(COLOR_DEFAULT_GRAY);

    // Border radius based on whether this is start/end of event
    // Single-day events have both start and end = true (fully rounded)
    // Multi-day events have appropriate rounding based on position
    let radius = 4.0;
    let border_radius: [f32; 4] = match (is_event_start, is_event_end) {
        (true, true) => [radius, radius, radius, radius],   // Single-day or start+end in same week
        (true, false) => [radius, 0.0, 0.0, radius],        // Starts here, continues right
        (false, true) => [0.0, radius, radius, 0.0],        // Continues from left, ends here
        (false, false) => [0.0, 0.0, 0.0, 0.0],             // Continues through
    };

    // Clone summary for the drag preview message (needed because text widget moves it)
    let drag_summary = summary.clone();

    let content: Element<'static, Message> = if show_text {
        widget::text(summary)
            .size(11)
            .wrapping(Wrapping::None)
            .into()
    } else {
        widget::text("")
            .size(11)
            .into()
    };

    // Dim opacity when being dragged to show it's in motion
    let base_opacity = if is_being_dragged { 0.15 } else if is_selected { 0.5 } else { 0.3 };
    let text_opacity = if is_being_dragged { 0.4 } else { 1.0 };

    let chip = container(content)
        .padding([2, 4, 2, 4])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme: &cosmic::Theme| {
            container::Style {
                background: Some(cosmic::iced::Background::Color(
                    color.scale_alpha(base_opacity)
                )),
                border: cosmic::iced::Border {
                    color: if is_selected { color } else { cosmic::iced::Color::TRANSPARENT },
                    width: if is_selected { BORDER_WIDTH_HIGHLIGHT } else { 0.0 },
                    radius: border_radius.into(),
                },
                text_color: Some(color.scale_alpha(text_opacity)),
                ..Default::default()
            }
        });

    // Wrap with mouse area for drag and click handling
    // Use DragEventStart on press (like timed events) - if released without moving,
    // handle_drag_event_end will treat it as a selection click
    // Pass summary and color_hex for the floating drag preview
    let mut area = mouse_area(chip)
        .on_press(Message::DragEventStart(uid.clone(), event_start_date, drag_summary, color_hex))
        .on_release(Message::DragEventEnd)
        .on_double_click(Message::OpenEditEventDialog(uid));

    // Track mouse movement during active drag to update target
    if is_drag_active {
        area = area.on_enter(Message::DragEventUpdate(event_start_date));
    }

    area.into()
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
        // Compute slot assignments for date events in this week
        let week_slot_info = events
            .as_ref()
            .map(|e| compute_week_event_slots(week, e.events_by_date));

        // Extract the slots map for compatibility with existing code
        let event_slots = week_slot_info.as_ref()
            .map(|info| info.slots.clone())
            .unwrap_or_default();

        // Compute the max slot for this week - all day cells need this for consistent placeholders
        let week_max_slot = event_slots.values().copied().max();

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
        for (day_col, calendar_day) in week.iter().enumerate() {
            let CalendarDay { year, month, day, is_current_month } = *calendar_day;

            // Check if today (need to compare full date)
            let today = chrono::Local::now();
            let is_today = year == today.year()
                && month == today.month()
                && day == today.day();

            // Check if this day is selected (works for both current and adjacent month days)
            // Don't show cell selection if an event is selected - event selection takes priority
            let cell_date = NaiveDate::from_ymd_opt(year, month, day);
            let has_event_selected = events.as_ref()
                .and_then(|e| e.selected_event_uid)
                .is_some();
            let is_selected = !has_event_selected && selected_date.is_some() && cell_date == selected_date;

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

            // Get selected event UID from events if available
            let selected_event_uid = events.as_ref()
                .and_then(|e| e.selected_event_uid)
                .map(|s| s.to_string());

            // Check if event drag is active
            let event_drag_active = events.as_ref()
                .map(|e| e.event_drag_active)
                .unwrap_or(false);

            // Get the UID of the event being dragged (for dimming its original position)
            let dragging_event_uid = events.as_ref()
                .and_then(|e| e.dragging_event_uid)
                .map(|s| s.to_string());

            // Check if this cell is the current drop target
            let is_drag_target = cell_date.is_some() && events.as_ref()
                .and_then(|e| e.drag_target_date)
                .map(|target| cell_date == Some(target))
                .unwrap_or(false);

            // Get occupied slots for this specific day (for Tetris-style rendering)
            let day_occupied_slots = week_slot_info.as_ref()
                .and_then(|info| info.day_occupied_slots.get(day_col).cloned())
                .unwrap_or_default();

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
                week_max_slot,
                day_occupied_slots,
                quick_event: quick_event_data,
                is_in_selection,
                selection_active,
                selected_event_uid,
                event_drag_active,
                dragging_event_uid,
                is_drag_target,
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

    // Build the final view with overlays
    let base = container(grid)
        .width(Length::Fill)
        .height(Length::Fill);

    // Collect overlays to stack
    let mut layers: Vec<Element<'a, Message>> = vec![base.into()];

    // Add date events overlay (renders all date events as spanning bars)
    // Wrapped in responsive to determine compact mode based on cell size
    if let Some(ref e) = events {
        // Clone data needed for the responsive closure
        let weeks = calendar_state.weeks_full.clone();
        let events_by_date = e.events_by_date.clone();
        let week_number_offset = if show_week_numbers { WEEK_NUMBER_WIDTH } else { 0.0 };
        let selected_uid = e.selected_event_uid.map(|s| s.to_string());
        let event_drag_active = e.event_drag_active;
        let dragging_uid = e.dragging_event_uid.map(|s| s.to_string());

        let responsive_overlay = responsive(move |size: Size| {
            // Calculate approximate cell width (7 days + spacing)
            let available_for_days = size.width - week_number_offset - (SPACING_TINY as f32 * 6.0);
            let cell_width = available_for_days / 7.0;

            // Calculate approximate cell height (total height minus header, divided by weeks)
            let num_weeks = weeks.len().max(1) as f32;
            let available_height = size.height - WEEKDAY_HEADER_HEIGHT - (SPACING_TINY as f32 * num_weeks);
            let cell_height = available_height / num_weeks;

            // Determine if we should use compact mode
            let compact = should_use_compact(cell_width, cell_height);

            if let Some(overlay) = render_date_events_overlay(
                &weeks,
                &events_by_date,
                show_week_numbers,
                compact,
                selected_uid.as_deref(),
                event_drag_active,
                dragging_uid.as_deref(),
            ) {
                overlay
            } else {
                // Return empty container if no events
                container(widget::text(""))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }
        });

        layers.push(responsive_overlay.into());
    }

    // Add quick event input overlay on top if active
    if has_quick_event_overlay {
        if let Some(quick_overlay) = events.as_ref().and_then(|e| {
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
        }) {
            layers.push(quick_overlay);
        }
    }

    // Stack all layers
    if layers.len() == 1 {
        layers.pop().unwrap()
    } else {
        stack(layers).into()
    }
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


