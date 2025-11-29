use cosmic::iced::{alignment, Length};
use cosmic::widget::{column, container, row};
use cosmic::{widget, Element};

use crate::components::render_day_cell;
use crate::message::Message;
use crate::models::CalendarState;
use crate::ui_constants::{
    FONT_SIZE_MEDIUM, FONT_SIZE_SMALL, PADDING_SMALL, PADDING_MONTH_GRID,
    SPACING_TINY, WEEKDAYS_FULL, WEEK_NUMBER_WIDTH
};

pub fn render_month_view(
    calendar_state: &CalendarState,
    selected_day: Option<u32>,
) -> Element<'static, Message> {
    let mut grid = column().spacing(SPACING_TINY).padding(PADDING_MONTH_GRID);

    // Weekday headers with week number column
    let mut header_row = row().spacing(SPACING_TINY);

    // Week number header
    header_row = header_row.push(
        container(widget::text("Wk").size(FONT_SIZE_SMALL))
            .width(Length::Fixed(WEEK_NUMBER_WIDTH))
            .padding(PADDING_SMALL)
            .align_y(alignment::Vertical::Center)
    );

    // Weekday headers - use iterator to avoid repetition
    for weekday in WEEKDAYS_FULL {
        header_row = header_row.push(
            container(widget::text(weekday).size(FONT_SIZE_MEDIUM))
                .width(Length::Fill)
                .padding(PADDING_SMALL)
                .center_x(Length::Fill),
        );
    }

    grid = grid.push(header_row);

    // Get week numbers for the month
    let week_numbers = calendar_state.week_numbers();

    // Use pre-calculated weeks from CalendarState cache
    for (week_index, week) in calendar_state.weeks.iter().enumerate() {
        let mut week_row = row().spacing(SPACING_TINY).height(Length::Fill);

        // Week number cell
        let week_number = week_numbers.get(week_index).copied().unwrap_or(0);
        week_row = week_row.push(
            container(
                widget::text(format!("{}", week_number))
                    .size(FONT_SIZE_SMALL)
            )
            .width(Length::Fixed(WEEK_NUMBER_WIDTH))
            .height(Length::Fill)
            .padding(PADDING_SMALL)
            .align_y(alignment::Vertical::Center)
        );

        // Day cells
        for day_opt in week {
            if let Some(day) = day_opt {
                let is_today = calendar_state.is_today(*day);
                let is_selected = selected_day == Some(*day);

                // Directly push cell without extra container wrapper
                week_row = week_row.push(
                    container(render_day_cell(*day, is_today, is_selected))
                        .width(Length::Fill)
                        .height(Length::Fill)
                );
            } else {
                // Empty cell - minimal structure
                week_row = week_row.push(
                    container(widget::text(""))
                        .width(Length::Fill)
                        .height(Length::Fill)
                );
            }
        }
        grid = grid.push(week_row);
    }

    container(grid)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
