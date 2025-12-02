//! Import result dialog UI component
//!
//! Shows the final result of a calendar import operation with:
//! - Success/failure indicator
//! - Import statistics (imported, skipped, failed)
//! - Ok button to close
//! - Revert button to undo the import

use cosmic::iced::Length;
use cosmic::widget::{button, column, dialog, text};
use cosmic::{widget, Element};

use crate::dialogs::ActiveDialog;
use crate::fl;
use crate::message::Message;

/// Render the import result dialog
pub fn render_import_result_dialog(active_dialog: &ActiveDialog) -> Element<'_, Message> {
    // Extract result data
    let (success, imported_count, skipped_count, failed_count, source_file_name, calendar_name, error_message) = match active_dialog {
        ActiveDialog::ImportResult {
            success,
            imported_count,
            skipped_count,
            failed_count,
            source_file_name,
            calendar_name,
            error_message,
            ..
        } => (success, imported_count, skipped_count, failed_count, source_file_name, calendar_name, error_message),
        _ => return widget::text("").into(),
    };

    // Status icon and message
    let (icon_name, status_message) = if *success {
        ("emblem-ok-symbolic", fl!("import-success"))
    } else {
        ("dialog-error-symbolic", fl!("import-failed"))
    };

    let icon = widget::icon::from_name(icon_name).size(64);
    let status_text = text(status_message).size(18);

    // Import statistics
    let mut stats = column().spacing(8);

    stats = stats.push(text(format!("📁 {}: {}", fl!("import-source-file"), source_file_name)).size(12));
    stats = stats.push(text(format!("📅 {}: {}", fl!("import-target-calendar"), calendar_name)).size(12));
    stats = stats.push(text("").size(4)); // Spacer

    if *imported_count > 0 {
        stats = stats.push(text(format!("✓ {}: {}", fl!("import-imported"), imported_count)).size(14));
    }
    if *skipped_count > 0 {
        stats = stats.push(text(format!("⊘ {}: {}", fl!("import-skipped"), skipped_count)).size(14));
    }
    if *failed_count > 0 {
        stats = stats.push(text(format!("✗ {}: {}", fl!("import-failed-count"), failed_count)).size(14));
    }

    // Error message if present
    if let Some(error_msg) = error_message {
        stats = stats.push(text("").size(4)); // Spacer
        stats = stats.push(text(format!("⚠️ {}: {}", fl!("error"), error_msg)).size(12));
    }

    // Main content
    let content = column()
        .spacing(16)
        .padding(16)
        .push(icon)
        .push(status_text)
        .push(stats);

    // Buttons: Ok (primary) and Revert (destructive, only if imported > 0)
    let ok_button = button::suggested(fl!("button-ok"))
        .on_press(Message::CloseDialog);

    let revert_button = if *imported_count > 0 {
        Some(button::destructive(fl!("button-revert"))
            .on_press(Message::RevertImport))
    } else {
        None
    };

    // Build dialog
    let mut dlg = dialog()
        .title(fl!("dialog-import-result-title"))
        .control(content)
        .primary_action(ok_button)
        .width(Length::Fixed(450.0));

    if let Some(revert_btn) = revert_button {
        dlg = dlg.secondary_action(revert_btn);
    }

    dlg.into()
}
