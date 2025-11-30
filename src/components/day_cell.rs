use chrono::NaiveDate;
use cosmic::iced::{alignment, Length};
use cosmic::widget::{column, container, mouse_area};
use cosmic::{widget, Element};

use crate::components::{render_split_events, render_quick_event_input, DisplayEvent};
use crate::message::Message;
use crate::styles::{
    today_circle_style, selected_day_style, day_cell_style, adjacent_month_day_style,
    adjacent_month_selected_style, selection_highlight_style, adjacent_month_selection_style
};
use crate::ui_constants::{PADDING_DAY_CELL, SPACING_SMALL};

/// Size of the circle behind today's day number
const TODAY_CIRCLE_SIZE: f32 = 32.0;

/// Vertical-only padding for day cells (all-day events need edge-to-edge)
const PADDING_DAY_CELL_VERTICAL: [u16; 4] = [PADDING_DAY_CELL[0], 0, PADDING_DAY_CELL[2], 0];

/// Apply the appropriate style to a day cell container based on state
/// Today no longer gets special cell styling - the circle is on the day number
/// Selected gets a border, drag selection gets highlight, regular cells get weekend background
/// Uses vertical-only padding so all-day events can span edge-to-edge
fn apply_day_cell_style<'a>(
    content: impl Into<Element<'a, Message>>,
    is_selected: bool,
    is_in_selection: bool,
    is_weekend: bool,
) -> container::Container<'a, Message, cosmic::Theme> {
    let base = container(content)
        .padding(PADDING_DAY_CELL_VERTICAL) // Vertical padding only, horizontal handled per-element
        .width(Length::Fill)
        .height(Length::Fill);

    if is_selected {
        base.style(|theme: &cosmic::Theme| selected_day_style(theme))
    } else if is_in_selection {
        base.style(move |theme: &cosmic::Theme| selection_highlight_style(theme, is_weekend))
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
    /// Whether this day is from an adjacent month (shown grayed out)
    pub is_adjacent_month: bool,
    pub events: Vec<DisplayEvent>,
    /// Slot assignments for multi-day events: maps event UID to slot index
    /// Events with slots are rendered in slot order, others fill remaining space
    pub event_slots: std::collections::HashMap<String, usize>,
    /// If Some, show quick event input with (editing_text, calendar_color)
    pub quick_event: Option<(String, String)>,
    /// Whether this day is part of the current drag selection range
    pub is_in_selection: bool,
    /// Whether a drag selection is currently active
    pub selection_active: bool,
}

/// Render a day cell with events and optional quick event input
pub fn render_day_cell_with_events(config: DayCellConfig) -> Element<'static, Message> {
    let date = NaiveDate::from_ymd_opt(config.year, config.month, config.day);

    // Day number - with circle background if today (only for current month)
    let day_number: Element<'static, Message> = if config.is_today && !config.is_adjacent_month {
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

    // Right-align the day number with horizontal padding
    let header = container(day_number)
        .width(Length::Fill)
        .padding([0, PADDING_DAY_CELL[1], 0, PADDING_DAY_CELL[3]]) // horizontal padding for header
        .align_x(alignment::Horizontal::Right);

    // Build content with day number at top
    let mut content = column()
        .spacing(SPACING_SMALL) // More spacing between day number and events
        .width(Length::Fill)
        .push(header);

    // Events section - split into all-day (edge-to-edge) and timed (with padding)
    let has_events = !config.events.is_empty() || config.quick_event.is_some();
    if has_events {
        // Show quick event input if editing on this day
        if let Some((text, color)) = config.quick_event {
            let quick_event_container = container(render_quick_event_input(text, color))
                .width(Length::Fill);
            content = content.push(quick_event_container);
        }

        // Show existing events (max 5 visible in month view)
        if !config.events.is_empty() && date.is_some() {
            let split_events = render_split_events(config.events, 5, date.unwrap(), &config.event_slots);

            // All-day events: edge-to-edge, no horizontal padding
            if let Some(all_day) = split_events.all_day {
                let all_day_container = container(all_day)
                    .width(Length::Fill)
                    .clip(true);
                content = content.push(all_day_container);
            }

            // Timed events: with horizontal padding for indentation
            if let Some(timed) = split_events.timed {
                let timed_container = container(timed)
                    .width(Length::Fill)
                    .padding([0, PADDING_DAY_CELL[1], 0, PADDING_DAY_CELL[3]]) // horizontal padding only
                    .clip(true);
                content = content.push(timed_container);
            }

            // Show "+N more" if there are hidden events
            if split_events.overflow_count > 0 {
                content = content.push(
                    container(
                        widget::text(format!("+{} more", split_events.overflow_count))
                            .size(10)
                    )
                    .padding([0, PADDING_DAY_CELL[1], 0, PADDING_DAY_CELL[3]]) // horizontal padding only
                );
            }
        }
    }

    // Build styled container based on state
    let styled_container = if config.is_adjacent_month {
        // Adjacent month: grayed out style, but show selection/highlight if applicable
        // Uses vertical-only padding so all-day events can span edge-to-edge
        if config.is_selected {
            container(content)
                .padding(PADDING_DAY_CELL_VERTICAL)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|theme: &cosmic::Theme| adjacent_month_selected_style(theme))
        } else if config.is_in_selection {
            container(content)
                .padding(PADDING_DAY_CELL_VERTICAL)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|theme: &cosmic::Theme| adjacent_month_selection_style(theme))
        } else {
            container(content)
                .padding(PADDING_DAY_CELL_VERTICAL)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_theme: &cosmic::Theme| adjacent_month_day_style())
        }
    } else {
        // Current month: normal styling (selected gets border, selection gets highlight)
        apply_day_cell_style(
            content,
            config.is_selected,
            config.is_in_selection,
            config.is_weekend,
        )
    };

    // Handle mouse interactions
    if let Some(date) = date {
        // Build mouse area with drag selection support
        let mut area = mouse_area(styled_container)
            // Start drag selection on mouse press
            .on_press(Message::SelectionStart(date))
            // End drag selection on mouse release
            .on_release(Message::SelectionEnd)
            // Double-click opens quick event for single-day creation
            .on_double_click(Message::StartQuickEvent(date));

        // Only track mouse movement during active selection for performance
        if config.selection_active {
            area = area.on_enter(Message::SelectionUpdate(date));
        }

        area.into()
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

    // Apply consistent styling (selected gets border, no selection highlighting for simple cells)
    let styled_container = apply_day_cell_style(content, is_selected, false, is_weekend);

    // Single mouse_area wrapping the styled container
    mouse_area(styled_container)
        .on_press(Message::SelectDay(year, month, day))
        .into()
}

