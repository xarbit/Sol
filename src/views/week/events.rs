//! Event rendering for the week view
//!
//! Contains timed event chip rendering and event overlay positioning.

use chrono::{Local, NaiveDate, Timelike};
use cosmic::iced::{Background, Border, Length};
use cosmic::iced_widget::keyed::Column as KeyedColumn;
use cosmic::widget::{container, mouse_area, row};
use cosmic::{widget, Element};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use crate::components::{parse_color_safe, ChipOpacity, DisplayEvent};
use crate::components::spacer::vertical_spacer;
use crate::message::Message;
use crate::ui_constants::{HOUR_ROW_HEIGHT, BORDER_RADIUS};

use super::utils::{event_time_range, PositionedEvent};

/// Spacing between event blocks in pixels (vertical gap)
const EVENT_BLOCK_SPACING: f32 = 2.0;

/// Hash a string to a u64 key for keyed columns
fn hash_key(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

/// Spacing between overlapping event columns in pixels (horizontal gap)
const EVENT_COLUMN_SPACING: u16 = 4;

/// Render the events overlay layer with events positioned based on their time spans
/// Uses a row of columns approach where each column renders its events independently
pub fn render_events_overlay_layer(
    date: NaiveDate,
    positioned_events: &[PositionedEvent],
    max_columns: usize,
    selected_event_uid: Option<&str>,
) -> Element<'static, Message> {
    // Each column renders its events independently with proper vertical positioning
    // This ensures overlapping events appear side-by-side
    let mut columns_row = row().spacing(EVENT_COLUMN_SPACING);

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
/// Uses KeyedColumn to ensure proper widget reconciliation when events change
fn render_column_events(
    date: NaiveDate,
    events: &[&PositionedEvent],
    selected_event_uid: Option<&str>,
) -> Element<'static, Message> {
    let mut keyed_children: Vec<(u64, Element<'static, Message>)> = Vec::new();
    let mut current_mins: u32 = 0;

    // Half spacing on each side of an event creates full spacing between consecutive events
    let half_spacing = EVENT_BLOCK_SPACING / 2.0;

    for (idx, pe) in events.iter().enumerate() {
        let (start_mins, end_mins) = event_time_range(&pe.event);
        let event_key = hash_key(&pe.event.uid);

        // Add spacer to position this event correctly (includes top margin)
        if start_mins > current_mins {
            let spacer_height = ((start_mins - current_mins) as f32 / 60.0) * HOUR_ROW_HEIGHT;
            keyed_children.push((
                hash_key(&format!("spacer-pre-{}", idx)),
                vertical_spacer(spacer_height).into()
            ));
        }

        // Add top margin spacer for this event
        keyed_children.push((
            hash_key(&format!("margin-top-{}", pe.event.uid)),
            vertical_spacer(half_spacing).into()
        ));

        // Render the event (subtract full spacing from height for top + bottom margins)
        let ev_height = ((end_mins - start_mins) as f32 / 60.0) * HOUR_ROW_HEIGHT - EVENT_BLOCK_SPACING;
        let event_block = render_positioned_event_block(
            date,
            &pe.event,
            ev_height.max(16.0), // Minimum height for visibility
            selected_event_uid,
        );
        // Key the event block with its UID hash for proper reconciliation
        keyed_children.push((event_key, event_block));

        // Add bottom margin spacer
        keyed_children.push((
            hash_key(&format!("margin-bottom-{}", pe.event.uid)),
            vertical_spacer(half_spacing).into()
        ));

        current_mins = end_mins;
    }

    // Fill remaining space to maintain column height
    let total_mins = 24 * 60;
    if current_mins < total_mins {
        let remaining_height = ((total_mins - current_mins) as f32 / 60.0) * HOUR_ROW_HEIGHT;
        keyed_children.push((
            hash_key("spacer-remaining"),
            vertical_spacer(remaining_height).into()
        ));
    }

    KeyedColumn::with_children(keyed_children)
        .spacing(0.0)
        .into()
}

/// Render a positioned event block with the specified height
fn render_positioned_event_block(
    date: NaiveDate,
    event: &DisplayEvent,
    height: f32,
    selected_event_uid: Option<&str>,
) -> Element<'static, Message> {
    let color = parse_color_safe(&event.color);
    let uid = event.uid.clone();
    let is_selected = selected_event_uid == Some(&event.uid);

    // Check if this event is in the past (considering time on today)
    let now = Local::now();
    let today = now.date_naive();
    let is_past = if date < today {
        // Past day - always dim
        true
    } else if date == today {
        // Today - check if event end time has passed
        if let Some(end_time) = event.end_time {
            let current_time = now.time();
            end_time <= current_time
        } else if let Some(start_time) = event.start_time {
            // No end time - use start time + 1 hour as heuristic
            let current_time = now.time();
            start_time <= current_time
        } else {
            false
        }
    } else {
        // Future day - not dim
        false
    };

    let (bg_opacity, border_width) = ChipOpacity::timed_event_opacity(is_selected, is_past);

    // Build the label with time and summary
    let time_str = event.start_time
        .map(|t| format!("{:02}:{:02}", t.hour(), t.minute()))
        .unwrap_or_default();
    let label = format!("{} {}", time_str, event.summary);

    let chip = container(
        widget::text(label.clone())
            .size(10)
    )
    .padding([2, 6])
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
