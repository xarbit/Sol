use cosmic::iced::Length;
use cosmic::widget::{button, column, container, row};
use cosmic::{widget, Element};

use crate::localized_names;
use crate::message::Message;
use crate::models::CalendarState;
use crate::ui_constants::{
    SPACING_MEDIUM, SPACING_XXS, SPACING_SMALL, SPACING_MINI_CALENDAR,
    PADDING_TINY, FONT_SIZE_SMALL, FONT_SIZE_BODY, MINI_CALENDAR_DAY_BUTTON_SIZE,
    MINI_CALENDAR_GRID_HEIGHT, ICON_PREVIOUS, ICON_NEXT
};

pub fn render_mini_calendar(
    calendar_state: &CalendarState,
    selected_day: Option<u32>,
) -> Element<'static, Message> {
    let month_year_text = calendar_state.month_year_text.clone();

    let header = row()
        .spacing(SPACING_MEDIUM)
        .push(
            button::icon(widget::icon::from_name(ICON_PREVIOUS))
                .on_press(Message::MiniCalendarPrevMonth)
                .padding(PADDING_TINY),
        )
        .push(container(widget::text::body(month_year_text).size(FONT_SIZE_BODY)).width(Length::Fill))
        .push(
            button::icon(widget::icon::from_name(ICON_NEXT))
                .on_press(Message::MiniCalendarNextMonth)
                .padding(PADDING_TINY),
        );

    let mut grid = column().spacing(SPACING_SMALL);

    // Weekday headers (abbreviated)
    let mut header_row = row().spacing(SPACING_XXS);
    let weekday_names = localized_names::get_weekday_names_short();
    for weekday in weekday_names {
        header_row = header_row.push(
            container(widget::text(weekday).size(FONT_SIZE_SMALL))
                .width(Length::Fixed(MINI_CALENDAR_DAY_BUTTON_SIZE))
                .center_x(Length::Fixed(MINI_CALENDAR_DAY_BUTTON_SIZE))
        );
    }

    grid = grid.push(header_row);

    // Use pre-calculated weeks from CalendarState
    for week in &calendar_state.weeks {
        let mut week_row = row().spacing(SPACING_XXS);
        for day_opt in week {
            if let Some(day) = day_opt {
                let is_today = calendar_state.is_today(*day);
                let is_selected = selected_day == Some(*day);

                let day_button = if is_today {
                    widget::button::suggested((*day).to_string())
                        .on_press(Message::SelectDay(*day))
                        .padding(PADDING_TINY)
                        .width(Length::Fixed(MINI_CALENDAR_DAY_BUTTON_SIZE))
                } else if is_selected {
                    widget::button::standard((*day).to_string())
                        .on_press(Message::SelectDay(*day))
                        .padding(PADDING_TINY)
                        .width(Length::Fixed(MINI_CALENDAR_DAY_BUTTON_SIZE))
                } else {
                    widget::button::text((*day).to_string())
                        .on_press(Message::SelectDay(*day))
                        .padding(PADDING_TINY)
                        .width(Length::Fixed(MINI_CALENDAR_DAY_BUTTON_SIZE))
                };
                week_row = week_row.push(day_button);
            } else {
                week_row = week_row.push(container(widget::text("")).width(Length::Fixed(MINI_CALENDAR_DAY_BUTTON_SIZE)));
            }
        }
        grid = grid.push(week_row);
    }

    // Wrap grid in fixed-height container to prevent layout shifts between months
    let grid_container = container(grid).height(Length::Fixed(MINI_CALENDAR_GRID_HEIGHT));

    column().spacing(SPACING_MINI_CALENDAR).push(header).push(grid_container).into()
}
