use chrono::NaiveDate;
use cosmic::iced::{alignment, Length};
use cosmic::widget::{column, container, mouse_area};
use cosmic::{widget, Element};

use crate::components::{render_events_column, render_quick_event_input, DisplayEvent};
use crate::message::Message;
use crate::styles::{today_outlined_style, selected_day_style, day_cell_style};
use crate::ui_constants::{PADDING_DAY_CELL, SPACING_TINY};

/// Apply the appropriate style to a day cell container based on state
fn apply_day_cell_style<'a>(
    content: impl Into<Element<'a, Message>>,
    is_today: bool,
    is_selected: bool,
    is_weekend: bool,
) -> container::Container<'a, Message, cosmic::Theme> {
    let base = container(content)
        .padding(PADDING_DAY_CELL)
        .width(Length::Fill)
        .height(Length::Fill);

    if is_today {
        base.style(|theme: &cosmic::Theme| today_outlined_style(theme))
    } else if is_selected {
        base.style(|theme: &cosmic::Theme| selected_day_style(theme))
    } else {
        base.style(move |_theme: &cosmic::Theme| day_cell_style(is_weekend))
    }
}

/// Configuration for rendering a day cell with events
pub struct DayCellConfig {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub is_today: bool,
    pub is_selected: bool,
    pub is_weekend: bool,
    pub events: Vec<DisplayEvent>,
    /// If Some, show quick event input with (editing_text, calendar_color)
    pub quick_event: Option<(String, String)>,
}

/// Render a day cell with events and optional quick event input
pub fn render_day_cell_with_events(config: DayCellConfig) -> Element<'static, Message> {
    let date = NaiveDate::from_ymd_opt(config.year, config.month, config.day);

    // Day number header - right aligned
    let day_text = if config.is_today || config.is_selected {
        widget::text::title4(config.day.to_string())
    } else {
        widget::text(config.day.to_string())
    };

    let header = container(day_text)
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Right);

    // Events section
    let mut content = column()
        .spacing(SPACING_TINY)
        .width(Length::Fill)
        .push(header);

    // Show quick event input if editing on this day
    if let Some((text, color)) = config.quick_event {
        content = content.push(render_quick_event_input(text, color));
    }

    // Show existing events (max 3 visible in month view)
    if !config.events.is_empty() {
        content = content.push(render_events_column(config.events, 3));
    }

    // Build styled container based on state
    let styled_container = apply_day_cell_style(
        content,
        config.is_today,
        config.is_selected,
        config.is_weekend,
    );

    // Double-click to create quick event, single click to select
    if let Some(date) = date {
        mouse_area(styled_container)
            .on_press(Message::SelectDay(config.year, config.month, config.day))
            .on_double_click(Message::StartQuickEvent(date))
            .into()
    } else {
        styled_container.into()
    }
}

/// Simple day cell render for backward compatibility (mini calendar, etc.)
pub fn render_day_cell(
    year: i32,
    month: u32,
    day: u32,
    is_today: bool,
    is_selected: bool,
    is_weekend: bool,
) -> Element<'static, Message> {
    // Day text styled based on state
    let day_text = if is_today || is_selected {
        widget::text::title4(day.to_string())
    } else {
        widget::text(day.to_string())
    };

    // Right-aligned content
    let content = container(day_text)
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Right);

    // Apply consistent styling
    let styled_container = apply_day_cell_style(content, is_today, is_selected, is_weekend);

    // Single mouse_area wrapping the styled container
    mouse_area(styled_container)
        .on_press(Message::SelectDay(year, month, day))
        .into()
}
