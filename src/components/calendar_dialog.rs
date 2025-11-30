use cosmic::iced::widget::stack;
use cosmic::iced::Length;
use cosmic::widget::{button, column, container, mouse_area, row, text, text_input};
use cosmic::{widget, Element};

use crate::components::color_picker::{parse_hex_color, QUICK_PICKER_COLORS};
use crate::dialogs::ActiveDialog;
use crate::fl;
use crate::message::Message;
use crate::styles::color_button_style;
use crate::ui_constants::{
    BORDER_WIDTH_HIGHLIGHT, BORDER_WIDTH_SELECTED, COLOR_BORDER_LIGHT, COLOR_BORDER_SELECTED,
    COLOR_BUTTON_SIZE_SMALL, COLOR_DEFAULT_GRAY, PADDING_STANDARD, SPACING_COLOR_GRID,
};

/// Render the calendar dialog (Create or Edit mode)
/// Takes the active dialog state which should be CalendarCreate or CalendarEdit variant
pub fn render_calendar_dialog(active_dialog: &ActiveDialog) -> Element<'_, Message> {
    // Extract data from active_dialog
    let (is_edit_mode, name, current_color) = match active_dialog {
        ActiveDialog::CalendarCreate { name, color } => (false, name.as_str(), color.as_str()),
        ActiveDialog::CalendarEdit { name, color, .. } => (true, name.as_str(), color.as_str()),
        _ => return widget::text("").into(), // Should not happen
    };

    let name_input = text_input(fl!("dialog-calendar-name-placeholder"), name)
        .on_input(Message::CalendarDialogNameChanged)
        .on_submit(|_| Message::ConfirmCalendarDialog)
        .width(Length::Fill);

    // Color picker grid using shared color constant
    let mut color_grid = column().spacing(SPACING_COLOR_GRID);

    for row_colors in QUICK_PICKER_COLORS {
        let mut color_row = row().spacing(SPACING_COLOR_GRID);

        for hex in row_colors {
            let color = parse_hex_color(hex).unwrap_or(COLOR_DEFAULT_GRAY);
            let hex_owned = hex.to_string();
            let is_selected = current_color == hex;

            let border_width = if is_selected {
                BORDER_WIDTH_SELECTED
            } else {
                BORDER_WIDTH_HIGHLIGHT
            };
            let border_color = if is_selected {
                COLOR_BORDER_SELECTED
            } else {
                COLOR_BORDER_LIGHT
            };

            let color_button = button::custom(
                container(widget::text(""))
                    .width(COLOR_BUTTON_SIZE_SMALL)
                    .height(COLOR_BUTTON_SIZE_SMALL)
                    .style(move |_theme: &cosmic::Theme| {
                        color_button_style(color, COLOR_BUTTON_SIZE_SMALL, border_width, border_color)
                    }),
            )
            .on_press(Message::CalendarDialogColorChanged(hex_owned))
            .padding(0);

            color_row = color_row.push(color_button);
        }

        color_grid = color_grid.push(color_row);
    }

    // Dialog buttons - text changes based on mode
    let cancel_btn = button::text(fl!("button-cancel")).on_press(Message::CancelCalendarDialog);

    let confirm_btn = if is_edit_mode {
        button::suggested(fl!("button-save")).on_press(Message::ConfirmCalendarDialog)
    } else {
        button::suggested(fl!("button-create")).on_press(Message::ConfirmCalendarDialog)
    };

    let buttons = row()
        .spacing(8)
        .push(widget::horizontal_space())
        .push(cancel_btn)
        .push(confirm_btn);

    // Dialog title changes based on mode
    let title = if is_edit_mode {
        fl!("dialog-edit-calendar-title")
    } else {
        fl!("dialog-new-calendar-title")
    };

    // Dialog content
    let content = column()
        .spacing(16)
        .push(text::title4(title))
        .push(
            column()
                .spacing(8)
                .push(text(fl!("dialog-calendar-name")))
                .push(name_input),
        )
        .push(
            column()
                .spacing(8)
                .push(text(fl!("dialog-calendar-color")))
                .push(color_grid),
        )
        .push(buttons);

    // Dialog card with styling
    let dialog_card = container(content)
        .padding(PADDING_STANDARD)
        .width(Length::Fixed(320.0))
        .style(|theme: &cosmic::Theme| {
            let cosmic = theme.cosmic();
            container::Style {
                background: Some(cosmic::iced::Background::Color(cosmic.background.base.into())),
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
        });

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

/// Render the delete calendar confirmation dialog
/// Takes the active dialog state which should be CalendarDelete variant
pub fn render_delete_calendar_dialog(active_dialog: &ActiveDialog) -> Element<'_, Message> {
    // Extract calendar name from active_dialog
    let calendar_name = match active_dialog {
        ActiveDialog::CalendarDelete { calendar_name, .. } => calendar_name.as_str(),
        _ => return widget::text("").into(), // Should not happen
    };

    // Dialog buttons
    let cancel_btn = button::text(fl!("button-cancel")).on_press(Message::CancelDeleteCalendar);

    let delete_btn =
        button::destructive(fl!("button-delete")).on_press(Message::ConfirmDeleteCalendar);

    let buttons = row()
        .spacing(8)
        .push(widget::horizontal_space())
        .push(cancel_btn)
        .push(delete_btn);

    // Dialog content
    let content = column()
        .spacing(16)
        .push(text::title4(fl!("dialog-delete-calendar-title")))
        .push(text(fl!(
            "dialog-delete-calendar-message",
            name = calendar_name.to_string()
        )))
        .push(buttons);

    // Dialog card with styling
    let dialog_card = container(content)
        .padding(PADDING_STANDARD)
        .width(Length::Fixed(360.0))
        .style(|theme: &cosmic::Theme| {
            let cosmic = theme.cosmic();
            container::Style {
                background: Some(cosmic::iced::Background::Color(cosmic.background.base.into())),
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
        });

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
