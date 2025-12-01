//! Header section rendering for the week view
//!
//! Contains day header row and all-day events section rendering.

use chrono::{Datelike, Local, NaiveDate};
use cosmic::iced::{alignment, Background, Border, Length};
use cosmic::iced_widget::keyed::Column as KeyedColumn;
use cosmic::widget::{column, container, mouse_area, row};
use cosmic::{widget, Element};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use crate::components::{parse_color_safe, ChipOpacity, DisplayEvent};
use crate::components::spacer::fixed_spacer;
use crate::locale::LocalePreferences;
use crate::localized_names;
use crate::message::Message;
use crate::models::WeekState;
use crate::styles::{today_filled_style, weekend_background};
use crate::ui_constants::{
    PADDING_SMALL, FONT_SIZE_SMALL, FONT_SIZE_MEDIUM, COLOR_DAY_CELL_BORDER,
    TIME_LABEL_WIDTH, BORDER_WIDTH_THIN, SPACING_TINY, BORDER_RADIUS,
};

use super::utils::{DAY_HEADER_HEIGHT, ALL_DAY_EVENT_HEIGHT, ALL_DAY_SPACING};

/// Hash a string to a u64 key for keyed columns
fn hash_key(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

/// Render the header section with day names, dates, and all-day events
pub fn render_header_section<'a>(
    week_state: &'a WeekState,
    locale: &'a LocalePreferences,
    all_day_events: &HashMap<NaiveDate, Vec<DisplayEvent>>,
    all_day_section_height: f32,
    selected_event_uid: Option<&str>,
) -> Element<'a, Message> {
    let mut header_col = column().spacing(0);

    // Day headers row
    let mut day_headers = row().spacing(0);

    // Time column placeholder for day headers
    day_headers = day_headers.push(fixed_spacer(TIME_LABEL_WIDTH, DAY_HEADER_HEIGHT));

    // Day headers
    for date in &week_state.days {
        let is_today = week_state.is_today(date);
        let is_weekend = locale.is_weekend(date.weekday());
        let day_name = localized_names::get_weekday_short(date.weekday());
        let day_number = format!("{}", date.day());

        let day_header = render_day_header(&day_name, &day_number, is_today);

        day_headers = day_headers.push(
            container(day_header)
                .width(Length::Fill)
                .height(Length::Fixed(DAY_HEADER_HEIGHT))
                .padding(PADDING_SMALL)
                .center_x(Length::Fill)
                .style(move |_theme: &cosmic::Theme| container::Style {
                    background: weekend_background(is_weekend),
                    border: Border {
                        width: BORDER_WIDTH_THIN,
                        color: COLOR_DAY_CELL_BORDER,
                        ..Default::default()
                    },
                    ..Default::default()
                })
        );
    }

    header_col = header_col.push(day_headers);

    // All-day events section
    let all_day_section = render_all_day_section(week_state, locale, all_day_events, all_day_section_height, selected_event_uid);
    header_col = header_col.push(all_day_section);

    header_col.into()
}

/// Render a single day header with day name and number
fn render_day_header<'a>(day_name: &str, day_number: &str, is_today: bool) -> Element<'a, Message> {
    let day_number_element: Element<'a, Message> = if is_today {
        container(
            widget::text(day_number.to_string()).size(FONT_SIZE_MEDIUM)
        )
        .padding(PADDING_SMALL)
        .style(|theme: &cosmic::Theme| today_filled_style(theme))
        .into()
    } else {
        widget::text(day_number.to_string()).size(FONT_SIZE_MEDIUM).into()
    };

    column()
        .spacing(SPACING_TINY)
        .align_x(alignment::Horizontal::Center)
        .push(widget::text(day_name.to_string()).size(FONT_SIZE_SMALL))
        .push(day_number_element)
        .into()
}

/// Render the all-day events section
fn render_all_day_section<'a>(
    week_state: &'a WeekState,
    locale: &'a LocalePreferences,
    all_day_events: &HashMap<NaiveDate, Vec<DisplayEvent>>,
    height: f32,
    selected_event_uid: Option<&str>,
) -> Element<'a, Message> {
    let mut all_day_row = row().spacing(0);

    // Time column with "all-day" label
    all_day_row = all_day_row.push(
        container(
            widget::text("").size(FONT_SIZE_SMALL)
        )
        .width(Length::Fixed(TIME_LABEL_WIDTH))
        .height(Length::Fixed(height))
        .padding(PADDING_SMALL)
        .align_y(alignment::Vertical::Top)
        .style(|_theme: &cosmic::Theme| container::Style {
            border: Border {
                width: BORDER_WIDTH_THIN,
                color: COLOR_DAY_CELL_BORDER,
                ..Default::default()
            },
            ..Default::default()
        })
    );

    // All-day events for each day
    for date in &week_state.days {
        let is_weekend = locale.is_weekend(date.weekday());
        let day_events = all_day_events.get(date).cloned().unwrap_or_default();
        let date_copy = *date;

        let events_column = render_all_day_events_for_day(*date, &day_events, selected_event_uid);

        let cell = container(events_column)
            .width(Length::Fill)
            .height(Length::Fixed(height))
            .padding([2, 2])
            .style(move |_theme: &cosmic::Theme| container::Style {
                background: weekend_background(is_weekend),
                border: Border {
                    width: BORDER_WIDTH_THIN,
                    color: COLOR_DAY_CELL_BORDER,
                    ..Default::default()
                },
                ..Default::default()
            });

        // Wrap in mouse_area for drag support (on_enter allows receiving drag updates)
        let clickable_cell = mouse_area(cell)
            .on_enter(Message::DragEventUpdate(date_copy));

        all_day_row = all_day_row.push(clickable_cell);
    }

    all_day_row.into()
}

/// Render all-day events for a single day as a vertical stack with click and drag support
/// Uses KeyedColumn to ensure proper widget reconciliation when events change
fn render_all_day_events_for_day(date: NaiveDate, events: &[DisplayEvent], selected_event_uid: Option<&str>) -> Element<'static, Message> {
    // Check if this date is in the past (all-day events are past at end of day)
    let today = Local::now().date_naive();
    let is_past = date < today; // All-day events don't have time - check by day

    // Use KeyedColumn for proper diffing when events are added/removed
    let keyed_children: Vec<(u64, Element<'static, Message>)> = events.iter().map(|event| {
        let color = parse_color_safe(&event.color);
        let uid = event.uid.clone();
        let key = hash_key(&uid);
        let is_selected = selected_event_uid == Some(&event.uid);

        // Selection highlight with past event dimming
        let (bg_opacity, border_width) = ChipOpacity::timed_event_opacity(is_selected, is_past);

        let chip = container(
            widget::text(event.summary.clone())
                .size(10)
        )
        .padding([2, 4])
        .width(Length::Fill)
        .height(Length::Fixed(ALL_DAY_EVENT_HEIGHT))
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

        // Wrap with mouse area for click and drag handling
        let clickable_chip: Element<'static, Message> = mouse_area(chip)
            .on_press(Message::DragEventStart(uid.clone(), date, event.summary.clone(), color_hex))
            .on_release(Message::DragEventEnd)
            .on_double_click(Message::OpenEditEventDialog(uid))
            .on_enter(Message::DragEventUpdate(date))
            .into();

        // Use event UID hash as the key for proper reconciliation
        (key, clickable_chip)
    }).collect();

    KeyedColumn::with_children(keyed_children)
        .spacing(ALL_DAY_SPACING as f32)
        .into()
}
