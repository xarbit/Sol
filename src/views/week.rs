use cosmic::iced::{alignment, Border, Length};
use cosmic::widget::{column, container, row, scrollable};
use cosmic::{widget, Element};

use crate::message::Message;
use crate::models::WeekState;
use crate::ui_constants::{
    SPACING_TINY, PADDING_SMALL,
    FONT_SIZE_SMALL, FONT_SIZE_MEDIUM, BORDER_RADIUS, COLOR_DAY_CELL_BORDER,
    HOUR_ROW_HEIGHT, TIME_LABEL_WIDTH, ALL_DAY_HEADER_HEIGHT
};

pub fn render_week_view(week_state: &WeekState) -> Element<'static, Message> {
    let all_day_section = render_all_day_section(week_state);
    let time_grid = render_time_grid(week_state);

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
fn render_all_day_section(week_state: &WeekState) -> Element<'static, Message> {
    let mut header_row = row().spacing(0);

    // Time column placeholder
    header_row = header_row.push(
        container(widget::text(""))
            .width(Length::Fixed(TIME_LABEL_WIDTH))
            .height(Length::Fixed(ALL_DAY_HEADER_HEIGHT))
    );

    // Day headers
    for date in week_state.days.clone() {
        let is_today = week_state.is_today(&date);
        let day_name = format!("{}", date.format("%a"));
        let day_number = format!("{}", date.format("%d"));

        let day_number_container = if is_today {
            container(
                widget::text(day_number).size(FONT_SIZE_MEDIUM)
            )
            .padding(PADDING_SMALL)
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
            container(widget::text(day_number).size(FONT_SIZE_MEDIUM))
                .padding(PADDING_SMALL)
        };

        let day_header = column()
            .spacing(SPACING_TINY)
            .align_x(alignment::Horizontal::Center)
            .push(widget::text(day_name).size(FONT_SIZE_SMALL))
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
    }

    header_row.into()
}

/// Render the main time grid with hourly slots
fn render_time_grid(week_state: &WeekState) -> Element<'static, Message> {
    let mut grid = column().spacing(0);

    // Render 24 hours
    for hour in 0..24 {
        let mut hour_row = row().spacing(0);

        // Time label
        let time_label = if hour == 0 {
            "12 AM".to_string()
        } else if hour < 12 {
            format!("{} AM", hour)
        } else if hour == 12 {
            "12 PM".to_string()
        } else {
            format!("{} PM", hour - 12)
        };

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

        // Day columns
        for _ in 0..7 {
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
        }

        grid = grid.push(hour_row);
    }

    grid.into()
}
