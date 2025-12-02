//! Month view event chip rendering
//!
//! Contains functions for rendering date event chips in the overlay.

use chrono::{Local, NaiveDate};
use cosmic::iced::widget::text::Wrapping;
use cosmic::iced::Length;
use cosmic::widget::{container, mouse_area};
use cosmic::{widget, Element};

use crate::components::color_picker::parse_color_safe;
use crate::components::{span_border_radius_from_flags, ChipOpacity};
use crate::message::Message;
use crate::ui_constants::{BORDER_RADIUS_SMALL, BORDER_RADIUS_VALUE, BORDER_WIDTH_HIGHLIGHT};

/// Render a compact date event chip (thin colored line without text)
/// Used when cell size is too small for full event chips
pub fn render_compact_date_event_chip(
    color_hex: String,
    is_event_start: bool,
    is_event_end: bool,
) -> Element<'static, Message> {
    let color = parse_color_safe(&color_hex);

    // Smaller radius for compact mode
    let border_radius = span_border_radius_from_flags(is_event_start, is_event_end, BORDER_RADIUS_SMALL);

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
/// Events on past dates are rendered with reduced opacity.
pub fn render_date_event_chip(
    calendar_id: String,
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
    event_date: NaiveDate,
) -> Element<'static, Message> {
    let color = parse_color_safe(&color_hex);

    // Border radius based on whether this is start/end of event
    let border_radius = span_border_radius_from_flags(is_event_start, is_event_end, BORDER_RADIUS_VALUE);

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

    // Check if this event is in the past (before today)
    let today = Local::now().date_naive();
    let is_past = event_date < today;

    // Dim opacity when being dragged or for past events
    let opacity = ChipOpacity::from_state_with_past(is_selected, is_being_dragged, is_past);

    let chip = container(content)
        .padding([2, 4, 2, 4])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme: &cosmic::Theme| {
            container::Style {
                background: Some(cosmic::iced::Background::Color(
                    color.scale_alpha(opacity.background)
                )),
                border: cosmic::iced::Border {
                    color: if is_selected { color } else { cosmic::iced::Color::TRANSPARENT },
                    width: if is_selected { BORDER_WIDTH_HIGHLIGHT } else { 0.0 },
                    radius: border_radius.into(),
                },
                text_color: Some(color.scale_alpha(opacity.text)),
                ..Default::default()
            }
        });

    // Wrap with mouse area for drag and click handling
    // Use DragEventStart on press (like timed events) - if released without moving,
    // handle_drag_event_end will treat it as a selection click
    // Pass calendar_id, summary and color_hex for the floating drag preview
    let mut area = mouse_area(chip)
        .on_press(Message::DragEventStart(calendar_id.clone(), uid.clone(), event_start_date, drag_summary, color_hex))
        .on_release(Message::DragEventEnd)
        .on_double_click(Message::OpenEditEventDialog(calendar_id, uid));

    // Track mouse movement during active drag to update target
    if is_drag_active {
        area = area.on_enter(Message::DragEventUpdate(event_start_date));
    }

    area.into()
}
