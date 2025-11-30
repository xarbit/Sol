use cosmic::iced::Length;
use cosmic::widget::{column, container, divider, scrollable};
use cosmic::Element;

use crate::calendars::CalendarSource;
use crate::components::{render_calendar_list, render_mini_calendar};
use crate::dialogs::ActiveDialog;
use crate::message::Message;
use crate::models::CalendarState;
use crate::ui_constants::{SIDEBAR_WIDTH, SPACING_LARGE, PADDING_STANDARD};

pub fn render_sidebar<'a>(
    calendar_state: &CalendarState,
    calendars: &'a [Box<dyn CalendarSource>],
    selected_day: Option<u32>,
    active_dialog: &'a ActiveDialog,
    selected_calendar_id: Option<&'a String>,
) -> Element<'a, Message> {
    let mini_calendar = render_mini_calendar(calendar_state, selected_day);

    // Use the calendar list component
    let calendars_section = render_calendar_list(calendars, active_dialog, selected_calendar_id);

    // Scrollable top section with calendars
    let scrollable_content = scrollable(
        column()
            .spacing(SPACING_LARGE)
            .padding(PADDING_STANDARD)
            .push(calendars_section)
    );

    // Bottom section with mini calendar (fixed at bottom)
    let bottom_section = column()
        .spacing(0)
        .push(divider::horizontal::default())
        .push(container(mini_calendar).padding(PADDING_STANDARD));

    // Combine: scrollable top + fixed bottom
    let sidebar_layout = column()
        .spacing(0)
        .push(container(scrollable_content).height(Length::Fill))
        .push(bottom_section);

    container(sidebar_layout)
        .width(Length::Fixed(SIDEBAR_WIDTH))
        .height(Length::Fill)
        .into()
}
