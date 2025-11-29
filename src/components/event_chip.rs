use cosmic::iced::Length;
use cosmic::iced_widget::text_input;
use cosmic::widget::{column, container};
use cosmic::{widget, Element};

use crate::components::color_picker::parse_hex_color;
use crate::message::Message;
use crate::ui_constants::{SPACING_TINY, BORDER_RADIUS, COLOR_DEFAULT_GRAY};

/// Event with associated calendar color for display
#[derive(Debug, Clone)]
pub struct DisplayEvent {
    pub uid: String,
    pub summary: String,
    pub color: String, // Hex color from calendar
}

/// Render a small event chip showing the event title with calendar color
/// Takes ownership of the event data to avoid lifetime issues
pub fn render_event_chip(event: DisplayEvent) -> Element<'static, Message> {
    let color = parse_hex_color(&event.color).unwrap_or(COLOR_DEFAULT_GRAY);
    let summary = event.summary;

    let chip = container(
        widget::text(summary)
            .size(11)
            .width(Length::Fill)
    )
    .padding([2, 4])
    .width(Length::Fill)
    .style(move |_theme: &cosmic::Theme| {
        container::Style {
            background: Some(cosmic::iced::Background::Color(color.scale_alpha(0.3))),
            border: cosmic::iced::Border {
                color,
                width: 0.0,
                radius: BORDER_RADIUS.into(),
            },
            text_color: Some(color),
            ..Default::default()
        }
    });

    chip.into()
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

/// Render a column of events for a day cell
/// Takes ownership of the events to avoid lifetime issues
pub fn render_events_column(
    events: Vec<DisplayEvent>,
    max_visible: usize,
) -> Element<'static, Message> {
    let mut col = column().spacing(SPACING_TINY);
    let total = events.len();

    for (i, event) in events.into_iter().enumerate() {
        if i >= max_visible {
            // Show "+N more" indicator
            let remaining = total - max_visible;
            col = col.push(
                widget::text(format!("+{} more", remaining))
                    .size(10)
            );
            break;
        }
        col = col.push(render_event_chip(event));
    }

    col.into()
}
