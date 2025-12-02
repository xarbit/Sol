//! Import progress dialog UI component
//!
//! Shows real-time progress during large calendar imports with:
//! - Spinning progress indicator
//! - Current event being imported
//! - Scrollable log of imported events
//! - Cancel button with rollback capability

use cosmic::iced::Length;
use cosmic::widget::{button, column, container, scrollable, text};
use cosmic::{widget, Element};

use crate::dialogs::ActiveDialog;
use crate::fl;
use crate::message::Message;

/// Render the import progress dialog
pub fn render_import_progress_dialog(active_dialog: &ActiveDialog) -> Element<'_, Message> {
    // Extract progress data
    let (current, total, current_event, import_log) = match active_dialog {
        ActiveDialog::ImportProgress {
            current,
            total,
            current_event,
            import_log,
            ..
        } => (current, total, current_event, import_log),
        _ => return widget::text("").into(),
    };

    // Progress text: "Importing event 5 of 100..."
    let progress_text = text(format!("Importing event {} of {}...", current, total))
        .size(16);

    // Current event being imported
    let current_event_text = text(format!("→ {}", current_event))
        .size(14);

    // Scrollable log of imported events
    let mut log_column = column().spacing(4);

    // Show last 10 entries in reverse order (newest first)
    for entry in import_log.iter().rev().take(10) {
        log_column = log_column.push(text(entry).size(12));
    }

    let log_scroll = scrollable(
        container(log_column)
            .padding(8)
    )
    .height(Length::Fixed(200.0));

    // Progress indicator (using text for now, could be spinner widget)
    let spinner = text("⟳").size(32); // Unicode spinner character

    // Main content
    let content = column()
        .spacing(16)
        .padding(16)
        .push(spinner)
        .push(progress_text)
        .push(current_event_text)
        .push(text("Import log:").size(14))
        .push(log_scroll);

    // Cancel button
    let cancel_button = button::destructive(fl!("button-cancel"))
        .on_press(Message::CancelImportProgress);

    // Dialog container
    container(
        column()
            .spacing(16)
            .push(content)
            .push(
                container(cancel_button)
                    .width(Length::Fill)
                    .center_x(Length::Fill)
            )
    )
    .width(Length::Fixed(450.0))
    .padding(16)
    .into()
}
