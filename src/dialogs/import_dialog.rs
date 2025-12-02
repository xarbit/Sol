//! Import dialog UI component
//!
//! Displays a dialog for importing calendar events from .ics files.
//! Shows event count, allows calendar selection, and provides import/cancel actions.

use cosmic::iced::Length;
use cosmic::widget::{button, column, dialog, radio, row, text};
use cosmic::{widget, Element};

use crate::calendars::CalendarSource;
use crate::dialogs::ActiveDialog;
use crate::fl;
use crate::message::Message;

/// Render the import events dialog using COSMIC dialog widget
/// Takes the active dialog state and available calendars
pub fn render_import_dialog<'a>(
    active_dialog: &'a ActiveDialog,
    calendars: &'a [Box<dyn CalendarSource>],
) -> Element<'a, Message> {
    // Extract data from active_dialog
    let (events, source_file_name, selected_calendar_id) = match active_dialog {
        ActiveDialog::Import {
            events,
            source_file_name,
            selected_calendar_id,
        } => (events, source_file_name.as_str(), selected_calendar_id.as_ref()),
        _ => return widget::text("").into(), // Should not happen
    };

    let event_count = events.len();

    // File info section
    let file_info = column()
        .spacing(8)
        .push(text(fl!("import-source-file")).size(14))
        .push(text(source_file_name).size(12));

    // Event count info
    let event_info = column().spacing(8).push(
        text(fl!(
            "import-event-count",
            count = (event_count as i64)
        ))
        .size(14),
    );

    // Calendar selection with radio buttons
    let mut calendar_control =
        column().spacing(8).push(text(fl!("import-target-calendar")).size(14));

    if calendars.is_empty() {
        calendar_control = calendar_control.push(text("(No calendars available)").size(12));
    } else {
        // Determine selected calendar (use first if none selected)
        let selected_id: Option<&String> = selected_calendar_id
            .or_else(|| calendars.first().map(|cal| &cal.info().id));

        // Create radio button for each calendar
        for calendar in calendars {
            let info = calendar.info();
            let calendar_id = &info.id;
            let calendar_name = &info.name;

            let radio_button = radio(
                calendar_name.as_str(),
                calendar_id,
                selected_id,
                |id| Message::SelectImportCalendar(id.to_string()),
            );

            calendar_control = calendar_control.push(radio_button);
        }
    }

    // Show event details in a scrollable area if there are events
    let events_preview = if !events.is_empty() {
        let mut event_list = column().spacing(4);

        // Show up to 5 events as preview
        for event in events.iter().take(5) {
            let event_text = if event.all_day {
                format!("• {}", event.summary)
            } else {
                format!(
                    "• {} ({} - {})",
                    event.summary,
                    event.start.format("%H:%M"),
                    event.end.format("%H:%M")
                )
            };
            event_list = event_list.push(text(event_text).size(12));
        }

        if events.len() > 5 {
            event_list = event_list.push(
                text(fl!("import-more-events", count = ((events.len() - 5) as i64)))
                    .size(12),
            );
        }

        column()
            .spacing(8)
            .push(text(fl!("import-events-preview")).size(14))
            .push(event_list)
    } else {
        column()
    };

    // Determine if import button should be enabled
    let can_import = !events.is_empty();

    let primary_btn = if can_import {
        button::suggested(fl!("button-import")).on_press(Message::ConfirmImport)
    } else {
        button::suggested(fl!("button-import"))
    };

    // Use COSMIC's dialog widget
    dialog()
        .title(fl!("dialog-import-title"))
        .icon(widget::icon::from_name("document-open-symbolic").size(64))
        .control(file_info)
        .control(event_info)
        .control(calendar_control)
        .control(events_preview)
        .secondary_action(button::text(fl!("button-cancel")).on_press(Message::CancelImport))
        .primary_action(primary_btn)
        .width(Length::Fixed(450.0))
        .into()
}
