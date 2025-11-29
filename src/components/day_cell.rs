use chrono::NaiveDate;
use cosmic::iced::{alignment, Length};
use cosmic::widget::{column, container, mouse_area};
use cosmic::{widget, Element};

use crate::components::{render_events_column, render_quick_event_input, DisplayEvent};
use crate::message::Message;
use crate::styles::{today_circle_style, selected_day_style, day_cell_style};
use crate::ui_constants::{PADDING_DAY_CELL, SPACING_TINY, SPACING_SMALL};

/// Size of the circle behind today's day number
const TODAY_CIRCLE_SIZE: f32 = 32.0;

/// Apply the appropriate style to a day cell container based on state
/// Today no longer gets special cell styling - the circle is on the day number
/// Selected gets a border, regular cells get weekend background if applicable
fn apply_day_cell_style<'a>(
    content: impl Into<Element<'a, Message>>,
    is_selected: bool,
    is_weekend: bool,
) -> container::Container<'a, Message, cosmic::Theme> {
    let base = container(content)
        .padding(PADDING_DAY_CELL)
        .width(Length::Fill)
        .height(Length::Fill);

    if is_selected {
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

    // Day number - with circle background if today
    let day_number: Element<'static, Message> = if config.is_today {
        // Today: blue circle behind the day number
        container(
            widget::text(config.day.to_string())
        )
        .width(Length::Fixed(TODAY_CIRCLE_SIZE))
        .height(Length::Fixed(TODAY_CIRCLE_SIZE))
        .center_x(Length::Fixed(TODAY_CIRCLE_SIZE))
        .center_y(Length::Fixed(TODAY_CIRCLE_SIZE))
        .style(|theme: &cosmic::Theme| today_circle_style(theme, TODAY_CIRCLE_SIZE))
        .into()
    } else {
        // Regular day number
        widget::text(config.day.to_string()).into()
    };

    // Right-align the day number
    let header = container(day_number)
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Right);

    // Build content with day number at top
    let mut content = column()
        .spacing(SPACING_SMALL) // More spacing between day number and events
        .width(Length::Fill)
        .push(header);

    // Events section in its own container
    let has_events = !config.events.is_empty() || config.quick_event.is_some();
    if has_events {
        let mut events_content = column()
            .spacing(SPACING_TINY)
            .width(Length::Fill);

        // Show quick event input if editing on this day
        if let Some((text, color)) = config.quick_event {
            events_content = events_content.push(render_quick_event_input(text, color));
        }

        // Show existing events (max 3 visible in month view)
        if !config.events.is_empty() {
            events_content = events_content.push(render_events_column(config.events, 3));
        }

        // Wrap events in a clipping container to prevent overflow
        let events_container = container(events_content)
            .width(Length::Fill)
            .clip(true);

        content = content.push(events_container);
    }

    // Build styled container based on state (selected gets border)
    let styled_container = apply_day_cell_style(
        content,
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
    // Day number - with circle background if today
    let day_number: Element<'static, Message> = if is_today {
        // Today: blue circle behind the day number
        container(
            widget::text(day.to_string())
        )
        .width(Length::Fixed(TODAY_CIRCLE_SIZE))
        .height(Length::Fixed(TODAY_CIRCLE_SIZE))
        .center_x(Length::Fixed(TODAY_CIRCLE_SIZE))
        .center_y(Length::Fixed(TODAY_CIRCLE_SIZE))
        .style(|theme: &cosmic::Theme| today_circle_style(theme, TODAY_CIRCLE_SIZE))
        .into()
    } else {
        // Regular day number
        widget::text(day.to_string()).into()
    };

    // Right-aligned content
    let content = container(day_number)
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Right);

    // Apply consistent styling (selected gets border)
    let styled_container = apply_day_cell_style(content, is_selected, is_weekend);

    // Single mouse_area wrapping the styled container
    mouse_area(styled_container)
        .on_press(Message::SelectDay(year, month, day))
        .into()
}
