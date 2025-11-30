use chrono::NaiveTime;
use cosmic::iced::Length;
use cosmic::iced::widget::text::Wrapping;
use cosmic::iced_widget::text_input;
use cosmic::widget::{column, container, row};
use cosmic::{widget, Element};

use crate::components::color_picker::parse_hex_color;
use crate::message::Message;
use crate::ui_constants::{SPACING_TINY, SPACING_XXS, BORDER_RADIUS, COLOR_DEFAULT_GRAY};

/// Spacing between date event placeholders (must match DATE_EVENT_SPACING in month.rs)
const DATE_EVENT_PLACEHOLDER_SPACING: u16 = 2;

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

}

/// Render an all-day event chip with colored background bar.
///
/// Note: Multi-day events are now rendered in an overlay layer (see month.rs).
/// This function is only called for single-day all-day events, so span_position
/// will always be Single. The span logic is kept for future use if needed.
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

    // Note: Multi-day events are now rendered in the overlay layer.
    // This function is only called for single-day all-day events (SpanPosition::Single).
    // The span_position logic is kept for backwards compatibility with render_event_chip.
    let content: Element<'static, Message> = widget::text(summary)
        .size(11)
        .wrapping(Wrapping::None)
        .into();

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

/// Result containing a unified events column with placeholders and timed events
pub struct UnifiedEventsResult {
    /// Single column containing placeholders (for date events) followed by timed events
    pub events: Option<Element<'static, Message>>,
    /// Number of events not shown (for "+N more" indicator)
    pub overflow_count: usize,
}

/// Render an empty placeholder to maintain slot alignment
/// This creates an invisible spacer with the same height as an overlay event row
/// Must match DATE_EVENT_HEIGHT (19.0) in month.rs exactly
fn render_empty_slot_placeholder() -> Element<'static, Message> {
    container(widget::text(""))
        .width(Length::Fill)
        .height(Length::Fixed(19.0)) // Must match DATE_EVENT_HEIGHT in overlay
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
pub fn render_unified_events(
    events: Vec<DisplayEvent>,
    max_visible: usize,
    current_date: NaiveDate,
    week_max_slot: Option<usize>,
) -> UnifiedEventsResult {
    // Separate all-day and timed events
    let (all_day_events, mut timed_events): (Vec<_>, Vec<_>) =
        events.into_iter().partition(|e| e.all_day);

    // Sort timed events by start time
    timed_events.sort_by(|a, b| a.start_time.cmp(&b.start_time));

    // Calculate placeholder count from week_max_slot
    // The overlay renders slots 0..=max_slot, so we need max_slot+1 placeholders
    let total_placeholders = week_max_slot.map(|m| m + 1).unwrap_or(0);

    // Count total events for overflow
    let actual_date_events = all_day_events.len();
    let total_events = actual_date_events + timed_events.len();

    // Build a single column with consistent DATE_EVENT_SPACING (2px) to match overlay
    let mut col = column().spacing(DATE_EVENT_PLACEHOLDER_SPACING);
    let mut shown = 0;

    // First: render placeholders for date event slots (these align with overlay)
    for _ in 0..total_placeholders {
        if shown >= max_visible {
            break;
        }
        col = col.push(render_empty_slot_placeholder());
        shown += 1;
    }

    // Second: render timed events (date-time events) below the placeholders
    for event in timed_events {
        if shown >= max_visible {
            break;
        }
        col = col.push(render_event_chip(event, current_date));
        shown += 1;
    }

    let overflow_count = if total_events > max_visible {
        total_events - max_visible
    } else {
        0
    };

    let events = if total_placeholders > 0 || shown > total_placeholders {
        Some(col.into())
    } else {
        None
    };

    UnifiedEventsResult {
        events,
        overflow_count,
    }
}

/// Height of compact event indicators (thin lines)
const COMPACT_EVENT_HEIGHT: f32 = 6.0;

/// Result of rendering compact events
pub struct CompactEventsResult {
    /// The rendered element containing all compact event indicators
    pub element: Option<Element<'static, Message>>,
    /// Number of events not shown
    pub overflow_count: usize,
}

/// Render a compact timed event indicator (small colored dot)
fn render_compact_timed_indicator(color: cosmic::iced::Color) -> Element<'static, Message> {
    container(widget::text(""))
        .width(Length::Fixed(COMPACT_EVENT_HEIGHT))
        .height(Length::Fixed(COMPACT_EVENT_HEIGHT))
        .style(move |_theme: &cosmic::Theme| {
            container::Style {
                background: Some(cosmic::iced::Background::Color(color)),
                border: cosmic::iced::Border {
                    color: cosmic::iced::Color::TRANSPARENT,
                    width: 0.0,
                    radius: (COMPACT_EVENT_HEIGHT / 2.0).into(),
                },
                ..Default::default()
            }
        })
        .into()
}

/// Empty compact placeholder to maintain slot alignment
fn render_compact_empty_placeholder() -> Element<'static, Message> {
    container(widget::text(""))
        .width(Length::Fill)
        .height(Length::Fixed(COMPACT_EVENT_HEIGHT))
        .into()
}

/// Render events in compact mode (thin colored lines/dots without text)
/// Used when cell size is too small for full event chips
///
/// # Arguments
/// * `events` - Events to render
/// * `max_visible` - Maximum number of compact indicators to show
/// * `current_date` - The date of the cell (for calculating span position)
/// * `event_slots` - Slot assignments for date events
/// * `week_max_slot` - Maximum slot index for the week (for consistent vertical positioning)
pub fn render_compact_events(
    events: Vec<DisplayEvent>,
    max_visible: usize,
    _current_date: NaiveDate,
    _event_slots: &std::collections::HashMap<String, usize>,
    week_max_slot: Option<usize>,
) -> CompactEventsResult {
    // Separate all-day and timed events
    let (all_day_events, mut timed_events): (Vec<_>, Vec<_>) =
        events.into_iter().partition(|e| e.all_day);

    // Sort timed events by start time
    timed_events.sort_by(|a, b| a.start_time.cmp(&b.start_time));

    // Use the week's max_slot to determine placeholder count
    let total_placeholders = week_max_slot.map(|m| m + 1).unwrap_or(0);
    let total_events = all_day_events.len() + timed_events.len();
    let mut shown = 0;

    // Use same spacing as overlay for proper alignment
    let mut col = column().spacing(DATE_EVENT_PLACEHOLDER_SPACING);
    let mut has_content = false;

    // Render placeholders for date events (actual events are in overlay)
    for _ in 0..total_placeholders {
        if shown >= max_visible {
            break;
        }
        col = col.push(render_compact_empty_placeholder());
        shown += 1;
        has_content = true;
    }

    // Render timed events as small dots in a row
    if !timed_events.is_empty() && shown < max_visible {
        let mut dots_row = row().spacing(SPACING_TINY);
        let remaining_slots = max_visible - shown;

        for (i, event) in timed_events.iter().enumerate() {
            if i >= remaining_slots {
                break;
            }
            let color = parse_hex_color(&event.color).unwrap_or(COLOR_DEFAULT_GRAY);
            dots_row = dots_row.push(render_compact_timed_indicator(color));
            shown += 1;
        }

        col = col.push(dots_row);
        has_content = true;
    }

    let overflow_count = if total_events > shown {
        total_events - shown
    } else {
        0
    };

    CompactEventsResult {
        element: if has_content { Some(col.into()) } else { None },
        overflow_count,
    }
}
