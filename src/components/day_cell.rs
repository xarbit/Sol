use chrono::NaiveDate;
use cosmic::iced::{alignment, Length, Size};
use cosmic::widget::{column, container, mouse_area, responsive};
use cosmic::{widget, Element};

use crate::components::{
    render_compact_events, render_unified_events_with_selection, render_quick_event_input, DisplayEvent,
    calculate_display_mode, EventDisplayMode,
};
use crate::message::Message;
use crate::styles::{
    today_circle_style, selected_day_style, day_cell_style, adjacent_month_day_style,
    adjacent_month_selected_style, selection_highlight_style, adjacent_month_selection_style,
    drag_target_style,
};
use crate::ui_constants::{PADDING_DAY_CELL, SPACING_SMALL, DAY_HEADER_HEIGHT};

/// Size of the circle behind today's day number
const TODAY_CIRCLE_SIZE: f32 = 32.0;

/// Vertical-only padding for day cells (all-day events need edge-to-edge)
const PADDING_DAY_CELL_VERTICAL: [u16; 4] = [PADDING_DAY_CELL[0], 0, PADDING_DAY_CELL[2], 0];

/// Apply the appropriate style to a day cell container based on state
/// Today no longer gets special cell styling - the circle is on the day number
/// Selected gets a border, drag selection gets highlight, drop target gets accent highlight,
/// regular cells get weekend background
/// Uses vertical-only padding so all-day events can span edge-to-edge
fn apply_day_cell_style<'a>(
    content: impl Into<Element<'a, Message>>,
    is_selected: bool,
    is_in_selection: bool,
    is_drag_target: bool,
    is_weekend: bool,
) -> container::Container<'a, Message, cosmic::Theme> {
    let base = container(content)
        .padding(PADDING_DAY_CELL_VERTICAL) // Vertical padding only, horizontal handled per-element
        .width(Length::Fill)
        .height(Length::Fill);

    if is_drag_target {
        // Drop target takes priority - show where the event will land
        base.style(move |theme: &cosmic::Theme| drag_target_style(theme, is_weekend))
    } else if is_selected {
        base.style(move |theme: &cosmic::Theme| selected_day_style(theme, is_weekend))
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
    /// Slot assignments for date events: maps event UID to slot index
    pub event_slots: std::collections::HashMap<String, usize>,
    /// Maximum slot index used in this week (for consistent vertical offset)
    /// All day cells in the same week should have the same max_slot value
    pub week_max_slot: Option<usize>,
    /// Slots occupied by date events on THIS specific day (for Tetris-style rendering)
    /// Timed events will fill empty slots instead of being pushed below
    pub day_occupied_slots: std::collections::HashSet<usize>,
    /// If Some, show quick event input with (editing_text, calendar_color)
    pub quick_event: Option<(String, String)>,
    /// Whether this day is part of the current drag selection range
    pub is_in_selection: bool,
    /// Whether a drag selection is currently active
    pub selection_active: bool,
    /// Currently selected event UID (for visual feedback)
    pub selected_event_uid: Option<String>,
    /// Whether an event drag operation is currently active
    pub event_drag_active: bool,
    /// The UID of the event currently being dragged (for dimming its original position)
    pub dragging_event_uid: Option<String>,
    /// Whether this cell is the current drop target
    pub is_drag_target: bool,
}

/// Render a day cell with events and optional quick event input
/// Uses responsive widget to dynamically adapt event display based on cell size
pub fn render_day_cell_with_events(config: DayCellConfig) -> Element<'static, Message> {
    let date = NaiveDate::from_ymd_opt(config.year, config.month, config.day);

    // Use responsive to get actual cell dimensions and adapt display
    let cell_content = responsive(move |size: Size| {
        let display_mode = calculate_display_mode(size);

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
        // Use fixed height to ensure consistent positioning that matches the overlay
        let header = container(day_number)
            .width(Length::Fill)
            .height(Length::Fixed(DAY_HEADER_HEIGHT))
            .padding([0, PADDING_DAY_CELL[1], 0, PADDING_DAY_CELL[3]]) // horizontal padding for header
            .align_x(alignment::Horizontal::Right);

        // Build content with day number at top
        let mut content = column()
            .spacing(SPACING_SMALL) // More spacing between day number and events
            .width(Length::Fill)
            .push(header);

        // Events section - adapts based on display mode
        // Check if we have events OR if there are slots reserved for date events spanning through this day
        let has_slot_reservations = config.week_max_slot.is_some();
        let has_events = !config.events.is_empty() || config.quick_event.is_some() || has_slot_reservations;

        if has_events {
            // Show quick event input if editing on this day (only in full mode)
            if let (Some((ref text, ref color)), EventDisplayMode::Full { .. }) = (&config.quick_event, display_mode) {
                let quick_event_container = container(render_quick_event_input(text.clone(), color.clone()))
                    .width(Length::Fill);
                content = content.push(quick_event_container);
            }

            // Show existing events based on display mode
            // Use unified events renderer that puts placeholders + timed events in a single column
            if (!config.events.is_empty() || has_slot_reservations) && date.is_some() {
                let current_date = date.unwrap();
                let max_visible = display_mode.max_visible();
                let show_overflow = display_mode.show_overflow();

                if display_mode.is_compact() {
                    // Compact mode: Tetris-style thin color indicators without text
                    let compact_events = render_compact_events(
                        config.events.clone(),
                        max_visible,
                        current_date,
                        &config.day_occupied_slots,
                        config.week_max_slot,
                    );

                    if let Some(compact_element) = compact_events.element {
                        content = content.push(compact_element);
                    }

                    // Show overflow count as small number if there are hidden events
                    // (only if cell is tall enough)
                    if show_overflow && compact_events.overflow_count > 0 {
                        content = content.push(
                            container(
                                widget::text(format!("+{}", compact_events.overflow_count))
                                    .size(8)
                            )
                            .padding([0, PADDING_DAY_CELL[1], 0, PADDING_DAY_CELL[3]])
                        );
                    }
                } else {
                    // Full mode: Tetris-style rendering with timed events filling empty slots
                    let unified = render_unified_events_with_selection(
                        config.events.clone(),
                        max_visible,
                        current_date,
                        config.week_max_slot,
                        &config.day_occupied_slots,
                        config.selected_event_uid.as_deref(),
                        config.event_drag_active,
                        config.dragging_event_uid.as_deref(),
                    );

                    // Single container for all events (placeholders + timed)
                    // Edge-to-edge width, clip overflow
                    if let Some(events_element) = unified.events {
                        let events_container = container(events_element)
                            .width(Length::Fill)
                            .clip(true);
                        content = content.push(events_container);
                    }

                    // Show "+N more" if there are hidden events (only if cell is tall enough)
                    if show_overflow && unified.overflow_count > 0 {
                        content = content.push(
                            container(
                                widget::text(format!("+{} more", unified.overflow_count))
                                    .size(10)
                            )
                            .padding([0, PADDING_DAY_CELL[1], 0, PADDING_DAY_CELL[3]])
                        );
                    }
                }
            }
        }

        // Build styled container based on state
        let styled: Element<'static, Message> = if config.is_adjacent_month {
            // Adjacent month: grayed out style, but show selection/highlight if applicable
            if config.is_selected {
                container(content)
                    .padding(PADDING_DAY_CELL_VERTICAL)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(|theme: &cosmic::Theme| adjacent_month_selected_style(theme))
                    .into()
            } else if config.is_in_selection {
                container(content)
                    .padding(PADDING_DAY_CELL_VERTICAL)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(|theme: &cosmic::Theme| adjacent_month_selection_style(theme))
                    .into()
            } else {
                container(content)
                    .padding(PADDING_DAY_CELL_VERTICAL)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(|_theme: &cosmic::Theme| adjacent_month_day_style())
                    .into()
            }
        } else {
            // Current month: normal styling (selected gets border, selection gets highlight, drop target gets accent)
            apply_day_cell_style(
                content,
                config.is_selected,
                config.is_in_selection,
                config.is_drag_target,
                config.is_weekend,
            ).into()
        };

        styled
    });

    // Handle mouse interactions
    if let Some(date) = date {
        // Build mouse area with drag selection and event drag support
        let mut area = mouse_area(cell_content)
            // Start drag selection on mouse press (only if not dragging an event)
            .on_press(Message::SelectionStart(date))
            // Double-click opens quick event for single-day creation
            .on_double_click(Message::StartQuickEvent(date));

        // Handle release: either end selection or end event drag
        if config.event_drag_active {
            area = area.on_release(Message::DragEventEnd);
        } else {
            area = area.on_release(Message::SelectionEnd);
        }

        // Track mouse movement during active selection or event drag
        if config.selection_active {
            area = area.on_enter(Message::SelectionUpdate(date));
        } else if config.event_drag_active {
            area = area.on_enter(Message::DragEventUpdate(date));
        }

        area.into()
    } else {
        cell_content.into()
    }
}
