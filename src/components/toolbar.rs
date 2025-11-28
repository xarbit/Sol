use cosmic::iced::Length;
use cosmic::widget::{button, container, row};
use cosmic::{widget, Element};

use crate::fl;
use crate::message::Message;
use crate::ui_constants::{ICON_NEXT, ICON_PREVIOUS};
use crate::views::CalendarView;

/// Render the calendar toolbar with navigation and view switcher
pub fn render_toolbar<'a>(
    period_text: &'a str,
    current_view: CalendarView,
) -> Element<'a, Message> {
    let toolbar_left = row()
        .spacing(8)
        .push(widget::button::standard(fl!("nav-today")).on_press(Message::Today))
        .push(
            button::icon(widget::icon::from_name(ICON_PREVIOUS))
                .on_press(Message::PreviousPeriod)
                .padding(8)
        )
        .push(
            button::icon(widget::icon::from_name(ICON_NEXT))
                .on_press(Message::NextPeriod)
                .padding(8)
        )
        .push(widget::text::title4(period_text));

    let view_switcher = render_view_switcher(current_view);

    row()
        .padding(16)
        .push(toolbar_left)
        .push(container(widget::text("")).width(Length::Fill))
        .push(view_switcher)
        .into()
}

/// Render the view switcher (Day/Week/Month buttons)
fn render_view_switcher(current_view: CalendarView) -> Element<'static, Message> {
    row()
        .spacing(4)
        .push(
            if current_view == CalendarView::Day {
                widget::button::suggested(fl!("view-day")).on_press(Message::ChangeView(CalendarView::Day))
            } else {
                widget::button::standard(fl!("view-day")).on_press(Message::ChangeView(CalendarView::Day))
            }
        )
        .push(
            if current_view == CalendarView::Week {
                widget::button::suggested(fl!("view-week")).on_press(Message::ChangeView(CalendarView::Week))
            } else {
                widget::button::standard(fl!("view-week")).on_press(Message::ChangeView(CalendarView::Week))
            }
        )
        .push(
            if current_view == CalendarView::Month {
                widget::button::suggested(fl!("view-month")).on_press(Message::ChangeView(CalendarView::Month))
            } else {
                widget::button::standard(fl!("view-month")).on_press(Message::ChangeView(CalendarView::Month))
            }
        )
        .into()
}
