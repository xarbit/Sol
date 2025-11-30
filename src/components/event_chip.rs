use chrono::NaiveTime;
use cosmic::iced::Length;
use cosmic::iced::widget::text::Wrapping;
use cosmic::iced_widget::text_input;
use cosmic::widget::{column, container, row};
use cosmic::{widget, Element};

use crate::components::color_picker::parse_hex_color;
use crate::message::Message;
use crate::ui_constants::{SPACING_TINY, SPACING_XXS, BORDER_RADIUS, COLOR_DEFAULT_GRAY};

/// ID for the quick event text input - used for auto-focus
pub fn quick_event_input_id() -> text_input::Id {
    text_input::Id::new("quick_event_input")
}

use chrono::NaiveDate;

/// Size of the colored dot for timed events
const TIMED_EVENT_DOT_SIZE: f32 = 8.0;

/// Position within a multi-day event span
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanPosition {
    /// Single-day event (not spanning)
    Single,
    /// First day of a multi-day event
    First,
    /// Middle day(s) of a multi-day event
    Middle,
    /// Last day of a multi-day event
    Last,
}

/// Event with associated calendar color for display
#[derive(Debug, Clone)]
pub struct DisplayEvent {
    pub uid: String,
    pub summary: String,
    pub color: String,      // Hex color from calendar
    pub all_day: bool,      // Whether this is an all-day event
    pub start_time: Option<NaiveTime>, // Start time for timed events
    /// Start date of the event span (for multi-day events)
    pub span_start: Option<NaiveDate>,
    /// End date of the event span (for multi-day events)
    pub span_end: Option<NaiveDate>,
}

impl DisplayEvent {
    /// Check if this is a multi-day all-day event
    pub fn is_multi_day(&self) -> bool {
        self.all_day
            && self.span_start.is_some()
            && self.span_end.is_some()
            && self.span_start != self.span_end
    }

    /// Get the span position for a given date within this event
    pub fn span_position_for_date(&self, date: NaiveDate) -> SpanPosition {
        match (self.span_start, self.span_end) {
            (Some(start), Some(end)) if start != end => {
                if date == start {
                    SpanPosition::First
                } else if date == end {
                    SpanPosition::Last
                } else if date > start && date < end {
                    SpanPosition::Middle
                } else {
                    SpanPosition::Single
                }
            }
            _ => SpanPosition::Single,
        }
    }

    /// Get the number of days this event spans
    pub fn span_days(&self) -> i64 {
        match (self.span_start, self.span_end) {
            (Some(start), Some(end)) => (end - start).num_days() + 1,
            _ => 1,
        }
    }
}

/// Render an all-day event chip with colored background bar
/// The span_position determines the border radius:
/// - Single: rounded on all corners
/// - First: rounded on left, flat on right
/// - Middle: flat on both sides
/// - Last: flat on left, rounded on right
fn render_all_day_chip(
    summary: String,
    color: cosmic::iced::Color,
    span_position: SpanPosition,
) -> Element<'static, Message> {
    // Calculate border radius based on span position
    let radius = BORDER_RADIUS[0];
    let border_radius: [f32; 4] = match span_position {
        SpanPosition::Single => [radius, radius, radius, radius],
        SpanPosition::First => [radius, 0.0, 0.0, radius],
        SpanPosition::Middle => [0.0, 0.0, 0.0, 0.0],
        SpanPosition::Last => [0.0, radius, radius, 0.0],
    };

    // Padding: reduce/remove horizontal padding on sides that continue
    // [top, right, bottom, left]
    let padding: [u16; 4] = match span_position {
        SpanPosition::Single => [2, 4, 2, 4],
        SpanPosition::First => [2, 0, 2, 4],   // No right padding - continues right
        SpanPosition::Middle => [2, 0, 2, 0],  // No horizontal padding - continues both sides
        SpanPosition::Last => [2, 4, 2, 0],    // No left padding - continues left
    };

    // Only show text on First or Single position
    let show_text = matches!(span_position, SpanPosition::First | SpanPosition::Single);

    let content: Element<'static, Message> = if show_text {
        widget::text(summary)
            .size(11)
            .wrapping(Wrapping::None)
            .into()
    } else {
        // Empty text for continuation segments - keeps the bar but no text
        widget::text("")
            .size(11)
            .into()
    };

    container(content)
        .padding(padding)
        .width(Length::Fill)
        .clip(true)
        .style(move |_theme: &cosmic::Theme| {
            container::Style {
                background: Some(cosmic::iced::Background::Color(color.scale_alpha(0.3))),
                border: cosmic::iced::Border {
                    color: cosmic::iced::Color::TRANSPARENT,
                    width: 0.0,
                    radius: border_radius.into(),
                },
                text_color: Some(color),
                ..Default::default()
            }
        })
        .into()
}

/// Render a timed event with colored dot + time + name
fn render_timed_event_chip(
    summary: String,
    start_time: Option<NaiveTime>,
    color: cosmic::iced::Color,
) -> Element<'static, Message> {
    // Colored dot
    let dot = container(widget::text(""))
        .width(Length::Fixed(TIMED_EVENT_DOT_SIZE))
        .height(Length::Fixed(TIMED_EVENT_DOT_SIZE))
        .style(move |_theme: &cosmic::Theme| {
            container::Style {
                background: Some(cosmic::iced::Background::Color(color)),
                border: cosmic::iced::Border {
                    color: cosmic::iced::Color::TRANSPARENT,
                    width: 0.0,
                    radius: (TIMED_EVENT_DOT_SIZE / 2.0).into(), // Circular
                },
                ..Default::default()
            }
        });

    // Format time if available
    let display_text = if let Some(time) = start_time {
        format!("{} {}", time.format("%H:%M"), summary)
    } else {
        summary
    };

    let text = widget::text(display_text)
        .size(11)
        .wrapping(Wrapping::None); // Prevent text from wrapping to next line

    // Wrap in container with clip to truncate long text
    container(
        row()
            .spacing(SPACING_XXS)
            .align_y(cosmic::iced::Alignment::Center)
            .push(dot)
            .push(text)
    )
    .width(Length::Fill)
    .clip(true) // Clip text that doesn't fit
    .into()
}

/// Render a small event chip showing the event title with calendar color
/// For all-day events: colored background bar with span-aware corners
/// For timed events: colored dot + time + name
///
/// # Arguments
/// * `event` - The display event with span metadata
/// * `current_date` - The date of the cell being rendered (for span position calculation)
pub fn render_event_chip(event: DisplayEvent, current_date: NaiveDate) -> Element<'static, Message> {
    let color = parse_hex_color(&event.color).unwrap_or(COLOR_DEFAULT_GRAY);

    if event.all_day {
        // Calculate span position for multi-day events
        let span_position = event.span_position_for_date(current_date);
        render_all_day_chip(event.summary, color, span_position)
    } else {
        render_timed_event_chip(event.summary, event.start_time, color)
    }
}

/// Render the quick event input field for inline editing
/// Takes ownership of the data to avoid lifetime issues
pub fn render_quick_event_input(
    text: String,
    calendar_color: String,
) -> Element<'static, Message> {
    let color = parse_hex_color(&calendar_color).unwrap_or(COLOR_DEFAULT_GRAY);

    let input = text_input("New event...", &text)
        .on_input(Message::QuickEventTextChanged)
        .on_submit(Message::CommitQuickEvent)
        .size(11)
        .padding([2, 4])
        .width(Length::Fill);

    container(input)
        .width(Length::Fill)
        .style(move |_theme: &cosmic::Theme| {
            container::Style {
                background: Some(cosmic::iced::Background::Color(color.scale_alpha(0.2))),
                border: cosmic::iced::Border {
                    color,
                    width: 1.0,
                    radius: BORDER_RADIUS.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

/// Render a spanning quick event input that covers multiple day columns
/// Used for multi-day event creation from drag selection
///
/// # Arguments
/// * `text` - Current input text
/// * `calendar_color` - Hex color of the selected calendar
/// * `span_columns` - Number of day columns to span (1-7)
/// * `show_week_numbers` - Whether week numbers column is visible (affects left padding)
pub fn render_spanning_quick_event_input(
    text: String,
    calendar_color: String,
    _span_columns: usize, // Reserved for future layout adjustments
) -> Element<'static, Message> {
    let color = parse_hex_color(&calendar_color).unwrap_or(COLOR_DEFAULT_GRAY);

    let input = text_input("New event...", &text)
        .id(quick_event_input_id())
        .on_input(Message::QuickEventTextChanged)
        .on_submit(Message::CommitQuickEvent)
        .size(14)
        .padding([6, 10])
        .width(Length::Fill);

    // The input spans across the specified number of columns
    // We use Length::Fill and let the parent container handle the width
    container(input)
        .width(Length::Fill)
        .padding([4, 6])
        .style(move |_theme: &cosmic::Theme| {
            container::Style {
                background: Some(cosmic::iced::Background::Color(color.scale_alpha(0.3))),
                border: cosmic::iced::Border {
                    color,
                    width: 2.0,
                    radius: crate::ui_constants::BORDER_RADIUS.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

/// Render a column of events for a day cell (month view)
/// All-day events are shown first as colored bars,
/// then timed events sorted chronologically with colored dots.
///
/// # Arguments
/// * `events` - Events to render
/// * `max_visible` - Maximum number of events to show
/// * `current_date` - The date of the cell (for calculating span position of multi-day events)
#[allow(dead_code)] // Keep for potential future use
pub fn render_events_column(
    events: Vec<DisplayEvent>,
    max_visible: usize,
    current_date: NaiveDate,
) -> Element<'static, Message> {
    // Separate all-day and timed events
    let (mut all_day_events, mut timed_events): (Vec<_>, Vec<_>) =
        events.into_iter().partition(|e| e.all_day);

    // Sort timed events by start time
    timed_events.sort_by(|a, b| a.start_time.cmp(&b.start_time));

    // Combine: all-day first, then timed
    let sorted_events: Vec<DisplayEvent> = all_day_events
        .drain(..)
        .chain(timed_events.drain(..))
        .collect();

    let mut col = column().spacing(SPACING_TINY);
    let total = sorted_events.len();

    for (i, event) in sorted_events.into_iter().enumerate() {
        if i >= max_visible {
            // Show "+N more" indicator
            let remaining = total - max_visible;
            col = col.push(
                widget::text(format!("+{} more", remaining))
                    .size(10)
            );
            break;
        }
        col = col.push(render_event_chip(event, current_date));
    }

    col.into()
}

/// Result of splitting events into all-day and timed categories
pub struct SplitEventsResult {
    /// All-day events column (edge-to-edge, no margin)
    pub all_day: Option<Element<'static, Message>>,
    /// Timed events column (with spacing/margin)
    pub timed: Option<Element<'static, Message>>,
    /// Number of events not shown (for "+N more" indicator)
    pub overflow_count: usize,
}

/// Render events split into two separate containers:
/// - All-day events: edge-to-edge colored bars (no margin)
/// - Timed events: dot + time format with margin/spacing
///
/// This allows different visual treatment for each event type in day cells.
/// Multi-day events are rendered in their assigned slots for visual consistency.
///
/// # Arguments
/// * `events` - Events to render
/// * `max_visible` - Maximum number of events to show
/// * `current_date` - The date of the cell (for calculating span position of multi-day events)
/// * `event_slots` - Slot assignments for multi-day events (UID -> slot index)
pub fn render_split_events(
    events: Vec<DisplayEvent>,
    max_visible: usize,
    current_date: NaiveDate,
    event_slots: &std::collections::HashMap<String, usize>,
) -> SplitEventsResult {
    // Separate all-day and timed events
    let (all_day_events, mut timed_events): (Vec<_>, Vec<_>) =
        events.into_iter().partition(|e| e.all_day);

    // Sort timed events by start time
    timed_events.sort_by(|a, b| a.start_time.cmp(&b.start_time));

    // Find the maximum slot number to know how many slot positions we need
    let max_slot = event_slots.values().copied().max();

    // Build a slot-ordered list of all-day events
    // We need to render events in strict slot order, with empty space for gaps
    let mut ordered_all_day: Vec<Option<DisplayEvent>> = Vec::new();

    if let Some(max_s) = max_slot {
        // Initialize with None for each slot
        ordered_all_day.resize(max_s + 1, None);

        // Place slotted events in their slots
        for event in all_day_events.iter() {
            if let Some(&slot) = event_slots.get(&event.uid) {
                if slot < ordered_all_day.len() {
                    ordered_all_day[slot] = Some(event.clone());
                }
            }
        }

        // Collect unslotted all-day events (single-day events)
        let unslotted: Vec<DisplayEvent> = all_day_events
            .into_iter()
            .filter(|e| !event_slots.contains_key(&e.uid))
            .collect();

        // Append unslotted events after the slotted ones
        for event in unslotted {
            ordered_all_day.push(Some(event));
        }
    } else {
        // No slotted events, just use all events as-is
        for event in all_day_events {
            ordered_all_day.push(Some(event));
        }
    }

    let total_events = ordered_all_day.iter().filter(|e| e.is_some()).count() + timed_events.len();
    let mut shown = 0;

    // Render all-day events in slot order (with gaps preserved)
    let all_day = if !ordered_all_day.is_empty() {
        let mut col = column().spacing(SPACING_TINY);
        for slot_event in ordered_all_day {
            if shown >= max_visible {
                break;
            }
            match slot_event {
                Some(event) => {
                    col = col.push(render_event_chip(event, current_date));
                }
                None => {
                    // Empty placeholder to maintain slot alignment
                    col = col.push(render_empty_slot_placeholder());
                }
            }
            shown += 1;
        }
        Some(col.into())
    } else {
        None
    };

    // Render timed events
    let timed = if !timed_events.is_empty() && shown < max_visible {
        let mut col = column().spacing(SPACING_TINY);
        for event in timed_events {
            if shown >= max_visible {
                break;
            }
            col = col.push(render_event_chip(event, current_date));
            shown += 1;
        }
        Some(col.into())
    } else {
        None
    };

    let overflow_count = if total_events > max_visible {
        total_events - max_visible
    } else {
        0
    };

    SplitEventsResult {
        all_day,
        timed,
        overflow_count,
    }
}

/// Render an empty placeholder to maintain slot alignment
/// This creates an invisible spacer with the same height as an event chip
fn render_empty_slot_placeholder() -> Element<'static, Message> {
    container(widget::text(""))
        .padding([2, 4])
        .width(Length::Fill)
        .height(Length::Fixed(19.0)) // Match event chip height (11px text + 2*2 padding + spacing)
        .into()
}

/// Render a spanning multi-day event chip that stretches across columns
/// Used in the overlay layer for multi-day all-day events
///
/// # Arguments
/// * `summary` - Event title
/// * `color_hex` - Hex color string for the event (from calendar)
/// * `span_position` - Position within the span (First, Middle, Last, Single)
pub fn render_spanning_event_chip(
    summary: String,
    color_hex: &str,
    span_position: SpanPosition,
) -> Element<'static, Message> {
    let color = parse_hex_color(color_hex).unwrap_or(COLOR_DEFAULT_GRAY);

    // Only show text on the first segment (or single segment)
    let show_text = matches!(span_position, SpanPosition::First | SpanPosition::Single);

    // Border radius based on position:
    // First: rounded left, flat right
    // Middle: flat both sides
    // Last: flat left, rounded right
    // Single: rounded both sides
    let radius = BORDER_RADIUS[0];
    let border_radius = match span_position {
        SpanPosition::Single => [radius, radius, radius, radius],
        SpanPosition::First => [radius, 0.0, 0.0, radius],
        SpanPosition::Middle => [0.0, 0.0, 0.0, 0.0],
        SpanPosition::Last => [0.0, radius, radius, 0.0],
    };

    let content: Element<'static, Message> = if show_text {
        widget::text(summary)
            .size(11)
            .wrapping(Wrapping::None)
            .into()
    } else {
        // Empty container for continuation segments
        widget::text("").into()
    };

    container(content)
        .padding([2, 4])
        .width(Length::Fill)
        .clip(true)
        .style(move |_theme: &cosmic::Theme| {
            container::Style {
                background: Some(cosmic::iced::Background::Color(color.scale_alpha(0.3))),
                border: cosmic::iced::Border {
                    color: cosmic::iced::Color::TRANSPARENT,
                    width: 0.0,
                    radius: border_radius.into(),
                },
                text_color: Some(color),
                ..Default::default()
            }
        })
        .into()
}
