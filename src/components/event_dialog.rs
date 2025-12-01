//! Event dialog component for creating and editing events
//! Uses COSMIC settings-style grouped sections with editable_input

use chrono::Weekday;
use cosmic::iced::widget::stack;
use cosmic::iced::Length;
use cosmic::widget::{button, calendar, column, container, mouse_area, popover, row, scrollable, settings, text, text_editor, toggler};
use cosmic::widget::editable_input;
use cosmic::{widget, Element};

use crate::app::{EventDialogField, EventDialogState};
use crate::caldav::{AlertTime, RepeatFrequency, TravelTime};
use crate::calendars::CalendarSource;
use crate::fl;
use crate::message::Message;

/// Helper to format TravelTime for display
fn travel_time_label(tt: &TravelTime) -> String {
    match tt {
        TravelTime::None => fl!("travel-time-none"),
        TravelTime::FiveMinutes => fl!("travel-time-5min"),
        TravelTime::TenMinutes => fl!("travel-time-10min"),
        TravelTime::FifteenMinutes => fl!("travel-time-15min"),
        TravelTime::ThirtyMinutes => fl!("travel-time-30min"),
        TravelTime::FortyFiveMinutes => fl!("travel-time-45min"),
        TravelTime::OneHour => fl!("travel-time-1hour"),
        TravelTime::OneHourThirty => fl!("travel-time-1hour30"),
        TravelTime::TwoHours => fl!("travel-time-2hours"),
        TravelTime::Custom(mins) => format!("{} min", mins),
    }
}

/// Helper to format RepeatFrequency for display
fn repeat_label(repeat: &RepeatFrequency) -> String {
    match repeat {
        RepeatFrequency::Never => fl!("repeat-never"),
        RepeatFrequency::Daily => fl!("repeat-daily"),
        RepeatFrequency::Weekly => fl!("repeat-weekly"),
        RepeatFrequency::Biweekly => fl!("repeat-biweekly"),
        RepeatFrequency::Monthly => fl!("repeat-monthly"),
        RepeatFrequency::Yearly => fl!("repeat-yearly"),
        RepeatFrequency::Custom(_) => fl!("repeat-custom"),
    }
}

/// Helper to format AlertTime for display
fn alert_label(alert: &AlertTime) -> String {
    match alert {
        AlertTime::None => fl!("alert-none"),
        AlertTime::AtTime => fl!("alert-at-time"),
        AlertTime::FiveMinutes => fl!("alert-5min"),
        AlertTime::TenMinutes => fl!("alert-10min"),
        AlertTime::FifteenMinutes => fl!("alert-15min"),
        AlertTime::ThirtyMinutes => fl!("alert-30min"),
        AlertTime::OneHour => fl!("alert-1hour"),
        AlertTime::TwoHours => fl!("alert-2hours"),
        AlertTime::OneDay => fl!("alert-1day"),
        AlertTime::TwoDays => fl!("alert-2days"),
        AlertTime::OneWeek => fl!("alert-1week"),
        AlertTime::Custom(mins) => format!("{} min before", mins),
    }
}

/// Render the event dialog (Create or Edit mode)
pub fn render_event_dialog<'a>(
    state: &'a EventDialogState,
    calendars: &'a [Box<dyn CalendarSource>],
) -> Element<'a, Message> {
    let is_edit_mode = state.editing_uid.is_some();

    // === Dialog Title ===
    let dialog_title = if is_edit_mode {
        fl!("event-edit")
    } else {
        fl!("event-new")
    };

    // Helper to check if a field is being edited
    let is_editing = |field: EventDialogField| -> bool {
        state.editing_field == Some(field)
    };

    // === Title Input using editable_input ===
    let title_input = editable_input(
        fl!("event-title-placeholder"),
        &state.title,
        is_editing(EventDialogField::Title),
        |editing| Message::EventDialogToggleEdit(EventDialogField::Title, editing),
    )
    .on_input(Message::EventDialogTitleChanged)
    .width(Length::Fill);

    // === Location Input using editable_input ===
    let location_input = editable_input(
        fl!("event-location-placeholder"),
        &state.location,
        is_editing(EventDialogField::Location),
        |editing| Message::EventDialogToggleEdit(EventDialogField::Location, editing),
    )
    .on_input(Message::EventDialogLocationChanged)
    .width(Length::Fill);

    let basic_section = settings::section()
        .add(
            settings::item::builder(fl!("event-title"))
                .control(title_input),
        )
        .add(
            settings::item::builder(fl!("event-location"))
                .control(location_input),
        );

    // === Date & Time Section ===
    let all_day_toggler = toggler(state.all_day)
        .on_toggle(Message::EventDialogAllDayToggled);

    // Start date display as text
    let start_date_text = text(&state.start_date_input).width(Length::Fixed(100.0));

    // Calendar picker button for start date with popover
    let start_date_picker_btn = button::custom(
        row()
            .spacing(8)
            .align_y(cosmic::iced::Alignment::Center)
            .push(start_date_text)
            .push(widget::icon::from_name("x-office-calendar-symbolic").size(16))
    )
    .on_press(Message::EventDialogToggleStartDatePicker)
    .padding([4, 8])
    .class(cosmic::theme::Button::Standard);

    // Start date calendar popover
    let start_date_with_picker: Element<'_, Message> = if state.start_date_picker_open {
        let calendar_popup = container(
            calendar(
                &state.start_date_calendar,
                Message::EventDialogStartDateChanged,
                || Message::EventDialogStartDateCalendarPrev,
                || Message::EventDialogStartDateCalendarNext,
                Weekday::Mon,
            )
        )
        .style(popup_container_style);
        popover(start_date_picker_btn)
            .popup(calendar_popup)
            .on_close(Message::EventDialogToggleStartDatePicker)
            .into()
    } else {
        start_date_picker_btn.into()
    };

    // Start time picker button
    let start_time_text = text(&state.start_time_input).width(Length::Fixed(60.0));
    let start_time_picker_btn = button::custom(
        row()
            .spacing(8)
            .align_y(cosmic::iced::Alignment::Center)
            .push(start_time_text)
            .push(widget::icon::from_name("preferences-system-time-symbolic").size(16))
    )
    .on_press(Message::EventDialogToggleStartTimePicker)
    .padding([4, 8])
    .class(cosmic::theme::Button::Standard);

    // Start time picker popover
    let start_time_with_picker: Element<'_, Message> = if state.start_time_picker_open {
        let time_popup = super::time_picker::render_time_picker(
            state.start_time,
            super::time_picker::start_time_hour_id(),
            super::time_picker::start_time_minute_id(),
            Message::EventDialogStartTimeHourChanged,
            Message::EventDialogStartTimeMinuteChanged,
            Message::EventDialogToggleStartTimePicker,
        );
        popover(start_time_picker_btn)
            .popup(time_popup)
            .on_close(Message::EventDialogToggleStartTimePicker)
            .into()
    } else {
        start_time_picker_btn.into()
    };

    // End date display as text
    let end_date_text = text(&state.end_date_input).width(Length::Fixed(100.0));

    // Calendar picker button for end date with popover
    let end_date_picker_btn = button::custom(
        row()
            .spacing(8)
            .align_y(cosmic::iced::Alignment::Center)
            .push(end_date_text)
            .push(widget::icon::from_name("x-office-calendar-symbolic").size(16))
    )
    .on_press(Message::EventDialogToggleEndDatePicker)
    .padding([4, 8])
    .class(cosmic::theme::Button::Standard);

    // End date calendar popover
    let end_date_with_picker: Element<'_, Message> = if state.end_date_picker_open {
        let calendar_popup = container(
            calendar(
                &state.end_date_calendar,
                Message::EventDialogEndDateChanged,
                || Message::EventDialogEndDateCalendarPrev,
                || Message::EventDialogEndDateCalendarNext,
                Weekday::Mon,
            )
        )
        .style(popup_container_style);
        popover(end_date_picker_btn)
            .popup(calendar_popup)
            .on_close(Message::EventDialogToggleEndDatePicker)
            .into()
    } else {
        end_date_picker_btn.into()
    };

    // End time picker button
    let end_time_text = text(&state.end_time_input).width(Length::Fixed(60.0));
    let end_time_picker_btn = button::custom(
        row()
            .spacing(8)
            .align_y(cosmic::iced::Alignment::Center)
            .push(end_time_text)
            .push(widget::icon::from_name("preferences-system-time-symbolic").size(16))
    )
    .on_press(Message::EventDialogToggleEndTimePicker)
    .padding([4, 8])
    .class(cosmic::theme::Button::Standard);

    // End time picker popover
    let end_time_with_picker: Element<'_, Message> = if state.end_time_picker_open {
        let time_popup = super::time_picker::render_time_picker(
            state.end_time,
            super::time_picker::end_time_hour_id(),
            super::time_picker::end_time_minute_id(),
            Message::EventDialogEndTimeHourChanged,
            Message::EventDialogEndTimeMinuteChanged,
            Message::EventDialogToggleEndTimePicker,
        );
        popover(end_time_picker_btn)
            .popup(time_popup)
            .on_close(Message::EventDialogToggleEndTimePicker)
            .into()
    } else {
        end_time_picker_btn.into()
    };

    let starts_row = if state.all_day {
        row().spacing(8).push(start_date_with_picker)
    } else {
        row()
            .spacing(8)
            .push(start_date_with_picker)
            .push(start_time_with_picker)
    };

    let ends_row = if state.all_day {
        row().spacing(8).push(end_date_with_picker)
    } else {
        row()
            .spacing(8)
            .push(end_date_with_picker)
            .push(end_time_with_picker)
    };

    let datetime_section = settings::section()
        .title(fl!("event-datetime-section"))
        .add(
            settings::item::builder(fl!("event-all-day"))
                .control(all_day_toggler),
        )
        .add(
            settings::item::builder(fl!("event-starts"))
                .control(starts_row),
        )
        .add(
            settings::item::builder(fl!("event-ends"))
                .control(ends_row),
        );

    // === Travel Time Section ===
    let travel_time_options = [
        TravelTime::None,
        TravelTime::FiveMinutes,
        TravelTime::FifteenMinutes,
        TravelTime::ThirtyMinutes,
        TravelTime::OneHour,
    ];

    let mut travel_buttons = row().spacing(4);
    for opt in travel_time_options.iter() {
        let is_selected = &state.travel_time == opt;
        let opt_clone = opt.clone();
        travel_buttons = travel_buttons.push(
            button::custom(text(travel_time_label(opt)).size(12))
                .on_press(Message::EventDialogTravelTimeChanged(opt_clone))
                .padding([4, 8])
                .class(if is_selected {
                    cosmic::theme::Button::Suggested
                } else {
                    cosmic::theme::Button::Standard
                }),
        );
    }

    // === Repeat Section ===
    let repeat_options = [
        RepeatFrequency::Never,
        RepeatFrequency::Daily,
        RepeatFrequency::Weekly,
        RepeatFrequency::Monthly,
        RepeatFrequency::Yearly,
    ];

    let mut repeat_buttons = row().spacing(4);
    for opt in repeat_options.iter() {
        let is_selected = &state.repeat == opt;
        let opt_clone = opt.clone();
        repeat_buttons = repeat_buttons.push(
            button::custom(text(repeat_label(opt)).size(12))
                .on_press(Message::EventDialogRepeatChanged(opt_clone))
                .padding([4, 8])
                .class(if is_selected {
                    cosmic::theme::Button::Suggested
                } else {
                    cosmic::theme::Button::Standard
                }),
        );
    }

    let schedule_section = settings::section()
        .title(fl!("event-schedule-section"))
        .add(
            settings::item::builder(fl!("event-travel-time"))
                .control(travel_buttons),
        )
        .add(
            settings::item::builder(fl!("event-repeat"))
                .control(repeat_buttons),
        );

    // === Calendar Section ===
    let mut calendar_section = settings::section()
        .title(fl!("event-calendar"));

    for calendar in calendars.iter() {
        let info = calendar.info();
        let is_selected = info.id == state.calendar_id;
        let cal_color = crate::components::color_picker::parse_hex_color(&info.color)
            .unwrap_or(crate::ui_constants::COLOR_DEFAULT_GRAY);
        let calendar_id = info.id.clone();

        let calendar_btn = button::custom(
            row()
                .spacing(8)
                .align_y(cosmic::iced::Alignment::Center)
                .push(
                    container(widget::text(""))
                        .width(12.0)
                        .height(12.0)
                        .style(move |_theme: &cosmic::Theme| container::Style {
                            background: Some(cosmic::iced::Background::Color(cal_color)),
                            border: cosmic::iced::Border {
                                radius: 3.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                )
                .push(text(&info.name)),
        )
        .on_press(Message::EventDialogCalendarChanged(calendar_id))
        .width(Length::Fill)
        .class(if is_selected {
            cosmic::theme::Button::Suggested
        } else {
            cosmic::theme::Button::Text
        });

        calendar_section = calendar_section.add(calendar_btn);
    }

    // === Alert Section ===
    let alert_options = [
        AlertTime::None,
        AlertTime::FifteenMinutes,
        AlertTime::ThirtyMinutes,
        AlertTime::OneHour,
        AlertTime::OneDay,
    ];

    let mut alert_buttons = row().spacing(4);
    for opt in alert_options.iter() {
        let is_selected = &state.alert == opt;
        let opt_clone = opt.clone();
        alert_buttons = alert_buttons.push(
            button::custom(text(alert_label(opt)).size(11))
                .on_press(Message::EventDialogAlertChanged(opt_clone))
                .padding([4, 6])
                .class(if is_selected {
                    cosmic::theme::Button::Suggested
                } else {
                    cosmic::theme::Button::Standard
                }),
        );
    }

    let alert_section = settings::section()
        .title(fl!("event-alert"))
        .add(settings::item::builder(fl!("event-alert")).control(alert_buttons));

    // === Invitees Section ===
    let mut invitee_chips = row().spacing(4);
    for (index, invitee) in state.invitees.iter().enumerate() {
        invitee_chips = invitee_chips.push(
            button::custom(
                row()
                    .spacing(4)
                    .push(text(invitee).size(12))
                    .push(text("×").size(12)),
            )
            .on_press(Message::EventDialogRemoveInvitee(index))
            .padding([2, 6])
            .class(cosmic::theme::Button::Standard),
        );
    }

    let invitee_input = editable_input(
        fl!("event-invitee-placeholder"),
        &state.invitee_input,
        true, // Always editable for input
        |_| Message::EventDialogAddInvitee, // Toggle acts as submit
    )
    .on_input(Message::EventDialogInviteeInputChanged)
    .on_submit(|_| Message::EventDialogAddInvitee)
    .width(Length::Fill);

    let invitees_content = column()
        .spacing(4)
        .push(invitee_chips)
        .push(invitee_input);

    let invitees_section = settings::section()
        .title(fl!("event-invitees"))
        .add(settings::item::builder(fl!("event-invitees")).control(invitees_content));

    // === Additional Info Section ===
    let url_input = editable_input(
        fl!("event-url-placeholder"),
        &state.url,
        is_editing(EventDialogField::Url),
        |editing| Message::EventDialogToggleEdit(EventDialogField::Url, editing),
    )
    .on_input(Message::EventDialogUrlChanged)
    .width(Length::Fill);

    // Notes uses text_editor for multi-line input
    let notes_editor = text_editor(&state.notes_content)
        .placeholder(fl!("event-notes-placeholder"))
        .on_action(Message::EventDialogNotesAction)
        .height(Length::Fixed(100.0));

    let additional_section = settings::section()
        .title(fl!("event-additional-section"))
        .add(
            settings::item::builder(fl!("event-url"))
                .control(url_input),
        )
        .add(
            settings::item::builder(fl!("event-notes"))
                .control(notes_editor),
        );

    // === Dialog Buttons ===
    let cancel_btn = button::text(fl!("button-cancel")).on_press(Message::CancelEventDialog);

    let confirm_btn = if is_edit_mode {
        button::suggested(fl!("button-save")).on_press(Message::ConfirmEventDialog)
    } else {
        button::suggested(fl!("button-create")).on_press(Message::ConfirmEventDialog)
    };

    let buttons = row()
        .spacing(8)
        .push(widget::horizontal_space())
        .push(cancel_btn)
        .push(confirm_btn);

    // === Build the form layout using settings view_column ===
    let form_content = settings::view_column(vec![
        basic_section.into(),
        datetime_section.into(),
        schedule_section.into(),
        calendar_section.into(),
        alert_section.into(),
        invitees_section.into(),
        additional_section.into(),
    ])
    .padding(0);

    let dialog_content = column()
        .spacing(12)
        .padding([16, 24]) // Add padding to content (top/bottom, left/right) to avoid scrollbar overlap
        .push(text::title4(dialog_title))
        .push(form_content)
        .push(buttons);

    // Wrap in scrollable for long content
    let scrollable_content = scrollable(dialog_content)
        .width(Length::Fill)
        .height(Length::Fill);

    // Dialog card with styling
    let dialog_card = container(scrollable_content)
        .padding([8, 0]) // Small vertical padding for the container, scrollbar stays outside content
        .width(Length::Fixed(580.0))
        .max_height(700.0)
        .style(dialog_container_style);

    // Clickable backdrop that closes the dialog
    let backdrop = mouse_area(
        container(widget::text(""))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme: &cosmic::Theme| container::Style {
                background: Some(cosmic::iced::Color::from_rgba(0.0, 0.0, 0.0, 0.5).into()),
                ..Default::default()
            }),
    )
    .on_press(Message::CloseDialog);

    // Center the dialog card
    let centered_dialog = container(dialog_card)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill);

    // Stack: backdrop on bottom, dialog on top
    stack![backdrop, centered_dialog].into()
}

/// Style function for popup containers (calendar picker, etc.)
fn popup_container_style(theme: &cosmic::Theme) -> container::Style {
    let cosmic = theme.cosmic();
    container::Style {
        background: Some(cosmic::iced::Background::Color(
            cosmic.background.base.into(),
        )),
        border: cosmic::iced::Border {
            radius: cosmic.corner_radii.radius_m.into(),
            width: 1.0,
            color: cosmic.bg_divider().into(),
        },
        shadow: cosmic::iced::Shadow {
            color: cosmic::iced::Color::from_rgba(0.0, 0.0, 0.0, 0.2),
            offset: cosmic::iced::Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
        ..Default::default()
    }
}

/// Style function for dialog container
fn dialog_container_style(theme: &cosmic::Theme) -> container::Style {
    let cosmic = theme.cosmic();
    container::Style {
        background: Some(cosmic::iced::Background::Color(
            cosmic.background.base.into(),
        )),
        border: cosmic::iced::Border {
            radius: cosmic.corner_radii.radius_m.into(),
            width: 1.0,
            color: cosmic.bg_divider().into(),
        },
        shadow: cosmic::iced::Shadow {
            color: cosmic::iced::Color::from_rgba(0.0, 0.0, 0.0, 0.3),
            offset: cosmic::iced::Vector::new(0.0, 4.0),
            blur_radius: 16.0,
        },
        ..Default::default()
    }
}
