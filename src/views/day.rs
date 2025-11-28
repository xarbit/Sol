use cosmic::iced::{alignment, Border, Length};
use cosmic::widget::{column, container, row, scrollable};
use cosmic::{widget, Element};

use crate::locale::LocalePreferences;
use crate::message::Message;
use crate::models::DayState;
use crate::ui_constants::{
    SPACING_TINY, PADDING_SMALL, PADDING_MEDIUM,
    FONT_SIZE_SMALL, FONT_SIZE_MEDIUM, FONT_SIZE_LARGE, BORDER_RADIUS, COLOR_DAY_CELL_BORDER,
    HOUR_ROW_HEIGHT, TIME_LABEL_WIDTH, ALL_DAY_HEADER_HEIGHT
};

pub fn render_day_view(day_state: &DayState, locale: &LocalePreferences) -> Element<'static, Message> {
    let all_day_section = render_all_day_section(day_state);
    let time_grid = render_time_grid(locale);

    let content = column()
        .spacing(0)
        .push(all_day_section)
        .push(scrollable(time_grid));

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Render the all-day events section at the top
fn render_all_day_section(day_state: &DayState) -> Element<'static, Message> {
    let mut header_row = row().spacing(0);

    // Clone strings to own them for 'static lifetime
    let is_today = day_state.is_today();
    let day_text = day_state.day_text.clone();
    let date_number = day_state.date_number.clone();

    // Time column placeholder
    header_row = header_row.push(
        container(widget::text(""))
            .width(Length::Fixed(TIME_LABEL_WIDTH))
            .height(Length::Fixed(ALL_DAY_HEADER_HEIGHT))
    );

    // Create day header with larger size for single day view
    let day_number_container = if is_today {
        container(
            widget::text(date_number.clone()).size(FONT_SIZE_LARGE)
        )
        .padding(PADDING_MEDIUM)
        .style(|theme: &cosmic::Theme| {
            container::Style {
                background: Some(cosmic::iced::Background::Color(
                    theme.cosmic().accent_color().into()
                )),
                border: Border {
                    radius: BORDER_RADIUS.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
    } else {
        container(widget::text(date_number.clone()).size(FONT_SIZE_LARGE))
            .padding(PADDING_MEDIUM)
    };

    let day_header = column()
        .spacing(SPACING_TINY)
        .align_x(alignment::Horizontal::Center)
        .push(widget::text(day_text).size(FONT_SIZE_MEDIUM))
        .push(day_number_container);

    header_row = header_row.push(
        container(day_header)
            .width(Length::Fill)
            .height(Length::Fixed(ALL_DAY_HEADER_HEIGHT))
            .padding(PADDING_SMALL)
            .style(|_theme: &cosmic::Theme| container::Style {
                border: Border {
                    width: 0.5,
                    color: COLOR_DAY_CELL_BORDER,
                    ..Default::default()
                },
                ..Default::default()
            })
    );

    header_row.into()
}

/// Render the main time grid with hourly slots
fn render_time_grid(locale: &LocalePreferences) -> Element<'static, Message> {
    let mut grid = column().spacing(0);

    // Render 24 hours
    for hour in 0..24 {
        let mut hour_row = row().spacing(0);

        // Time label - use locale-aware formatting
        let time_label = locale.format_hour(hour);

        hour_row = hour_row.push(
            container(
                widget::text(time_label)
                    .size(FONT_SIZE_SMALL)
            )
            .width(Length::Fixed(TIME_LABEL_WIDTH))
            .height(Length::Fixed(HOUR_ROW_HEIGHT))
            .padding(PADDING_SMALL)
            .align_y(alignment::Vertical::Top)
            .style(|_theme: &cosmic::Theme| container::Style {
                border: Border {
                    width: 0.5,
                    color: COLOR_DAY_CELL_BORDER,
                    ..Default::default()
                },
                ..Default::default()
            })
        );

        // Day column - wider for better event visibility
        hour_row = hour_row.push(
            container(widget::text(""))
                .width(Length::Fill)
                .height(Length::Fixed(HOUR_ROW_HEIGHT))
                .style(|_theme: &cosmic::Theme| container::Style {
                    border: Border {
                        width: 0.5,
                        color: COLOR_DAY_CELL_BORDER,
                        ..Default::default()
                    },
                    ..Default::default()
                })
        );

        grid = grid.push(hour_row);
    }

    grid.into()
}
