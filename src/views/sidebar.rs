use cosmic::iced::Length;
use cosmic::widget::{column, container, divider, row, scrollable};
use cosmic::{widget, Element};

use crate::components::render_mini_calendar;
use crate::message::Message;
use crate::models::CalendarState;

pub fn render_sidebar(
    calendar_state: &CalendarState,
    selected_day: Option<u32>,
) -> Element<'static, Message> {
    let mini_calendar = render_mini_calendar(calendar_state, selected_day);

    let calendars_section = column()
        .spacing(8)
        .padding(12)
        .push(widget::text::body("Calendars").size(14))
        .push(
            row()
                .spacing(8)
                .push(widget::checkbox("", true))
                .push(widget::text("Personal")),
        )
        .push(
            row()
                .spacing(8)
                .push(widget::checkbox("", true))
                .push(widget::text("Work")),
        );

    // Scrollable top section with calendars
    let scrollable_content = scrollable(
        column()
            .spacing(20)
            .padding(16)
            .push(calendars_section)
    );

    // Bottom section with mini calendar (fixed at bottom)
    let bottom_section = column()
        .spacing(0)
        .push(divider::horizontal::default())
        .push(container(mini_calendar).padding(16));

    // Combine: scrollable top + fixed bottom
    let sidebar_layout = column()
        .spacing(0)
        .push(container(scrollable_content).height(Length::Fill))
        .push(bottom_section);

    container(sidebar_layout)
        .width(Length::Fixed(280.0))
        .height(Length::Fill)
        .into()
}
