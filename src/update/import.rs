//! Import/Export Update Handlers
//!
//! Handles all import and export related messages.

use crate::app::CosmicCalendar;
use crate::dialogs::{ActiveDialog, DialogAction, DialogManager};
use crate::message::Message;
use crate::services::{EventHandler, ExportHandler};
use cosmic::app::Task;
use log::{debug, error, info, warn};
use std::path::PathBuf;

/// Handle import file message - parse the file and show import dialog
pub fn handle_import_file(app: &mut CosmicCalendar, path: PathBuf) -> Task<Message> {
    info!("handle_import_file: Importing from {:?}", path);

    // Get the file name for display
    let source_file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown file")
        .to_string();

    // Validate the iCalendar file for RFC 5545 compliance
    info!("handle_import_file: Validating file format");
    if let Err(e) = ExportHandler::validate_ical_file(&path) {
        error!("handle_import_file: Validation failed: {}", e);
        // TODO: Show error dialog with validation details
        return Task::none();
    }

    // Detect iCalendar dialect for better compatibility
    if let Ok(content) = std::fs::read_to_string(&path) {
        if let Some(dialect) = ExportHandler::detect_dialect(&content) {
            info!("handle_import_file: Detected iCalendar dialect: {}", dialect);
        }
    }

    // Parse the iCalendar file
    match ExportHandler::parse_ical_file(&path) {
        Ok(events) => {
            if events.is_empty() {
                error!("handle_import_file: No events found in file");
                // TODO: Show error dialog or notification
                return Task::none();
            }

            info!("handle_import_file: Parsed {} events", events.len());

            // Smart import logic:
            // - Single event: Add to default calendar and open event dialog pre-filled
            // - Multiple events: Open import dialog for calendar selection
            if events.len() == 1 {
                info!("handle_import_file: Single event - opening event dialog");

                // Get first (and only) event
                let event = events.into_iter().next().unwrap();

                // Get the first available calendar as default
                if let Some(calendar) = app.calendar_manager.sources().first() {
                    let calendar_id = calendar.info().id.clone();

                    // Add event to the default calendar
                    match EventHandler::add_event(&mut app.calendar_manager, &calendar_id, event.clone()) {
                        Ok(_) => {
                            info!("handle_import_file: Event added to calendar '{}'", calendar_id);
                            // Refresh the calendar view
                            app.refresh_cached_events();
                            // Open the event dialog for editing/review
                            return Task::done(cosmic::Action::App(Message::OpenEditEventDialog(calendar_id, event.uid)));
                        }
                        Err(e) => {
                            error!("handle_import_file: Failed to add event: {}", e);
                            // TODO: Show error notification
                        }
                    }
                } else {
                    error!("handle_import_file: No calendars available");
                    // TODO: Show error notification
                }
            } else {
                // Multiple events: Use import dialog for calendar selection
                info!("handle_import_file: Multiple events - opening import dialog");
                DialogManager::handle_action(
                    &mut app.active_dialog,
                    DialogAction::OpenImport {
                        events,
                        source_file_name,
                    },
                );
            }
        }
        Err(e) => {
            error!("handle_import_file: Failed to parse file: {}", e);
            // TODO: Show error dialog or notification
        }
    }

    Task::none()
}

/// Handle show import dialog message (events already parsed)
pub fn handle_show_import_dialog(
    app: &mut CosmicCalendar,
    events: Vec<crate::caldav::CalendarEvent>,
    source_file_name: String,
) -> Task<Message> {
    info!(
        "handle_show_import_dialog: {} events from {}",
        events.len(),
        source_file_name
    );

    if events.is_empty() {
        error!("handle_show_import_dialog: No events to import");
        return Task::none();
    }

    // Open the import dialog
    DialogManager::handle_action(
        &mut app.active_dialog,
        DialogAction::OpenImport {
            events,
            source_file_name,
        },
    );

    Task::none()
}

/// Handle select import calendar message
pub fn handle_select_import_calendar(app: &mut CosmicCalendar, calendar_id: String) -> Task<Message> {
    debug!(
        "handle_select_import_calendar: Selected calendar '{}'",
        calendar_id
    );

    // Update the dialog state with selected calendar
    DialogManager::handle_action(
        &mut app.active_dialog,
        DialogAction::SelectImportCalendar(calendar_id),
    );

    Task::none()
}

/// Handle confirm import message - perform the actual import
pub fn handle_confirm_import(app: &mut CosmicCalendar) -> Task<Message> {
    info!("handle_confirm_import: Confirming import");

    // Extract data from the import dialog
    let (events, source_file_name, selected_calendar_id) = match &app.active_dialog {
        ActiveDialog::Import {
            events,
            source_file_name,
            selected_calendar_id,
        } => (
            events.clone(),
            source_file_name.clone(),
            selected_calendar_id.clone(),
        ),
        _ => {
            error!("handle_confirm_import: Not in import dialog state");
            return Task::none();
        }
    };

    // Determine target calendar
    // If no calendar selected, use the first available calendar
    let target_calendar_id = if let Some(id) = selected_calendar_id {
        id
    } else {
        // Get first available calendar
        if let Some(cal) = app.calendar_manager.sources().first() {
            cal.info().id.clone()
        } else {
            error!("handle_confirm_import: No calendars available");
            DialogManager::close(&mut app.active_dialog);
            return Task::none();
        }
    };

    info!(
        "handle_confirm_import: Importing {} events into calendar '{}'",
        events.len(),
        target_calendar_id
    );

    // For large imports (>10 events), show progress dialog
    const PROGRESS_THRESHOLD: usize = 10;
    if events.len() > PROGRESS_THRESHOLD {
        info!("handle_confirm_import: Large import detected, showing progress dialog");
        // Open progress dialog
        app.active_dialog = ActiveDialog::ImportProgress {
            current: 0,
            total: events.len(),
            current_event: String::new(),
            import_log: Vec::new(),
            imported_uids: Vec::new(),
            calendar_id: target_calendar_id.clone(),
        };
    }

    // Import events one by one using the event handler
    let mut imported_count = 0;
    let mut skipped_count = 0;
    let mut imported_uids = Vec::new();

    for (index, event) in events.iter().enumerate() {
        // Check if event already exists in the TARGET calendar (by UID)
        let exists = app
            .calendar_manager
            .sources()
            .iter()
            .find(|cal| cal.info().id == target_calendar_id)
            .and_then(|cal| {
                cal.fetch_events()
                    .ok()
                    .map(|events| events.iter().any(|e| e.uid == event.uid))
            })
            .unwrap_or(false);

        if exists {
            debug!("handle_confirm_import: Skipping duplicate event uid={} in target calendar", event.uid);
            skipped_count += 1;
            continue;
        }

        // Add event to the target calendar
        match EventHandler::add_event(&mut app.calendar_manager, &target_calendar_id, event.clone()) {
            Ok(_) => {
                imported_count += 1;
                imported_uids.push(event.uid.clone());

                // Update progress dialog if open
                if let ActiveDialog::ImportProgress {
                    current,
                    current_event,
                    import_log,
                    ..
                } = &mut app.active_dialog
                {
                    *current = index + 1;
                    *current_event = event.summary.clone();
                    import_log.push(format!("✓ Imported: {}", event.summary));
                }
            }
            Err(e) => {
                error!("handle_confirm_import: Failed to import event: {}", e);

                // Log error in progress dialog if open
                if let ActiveDialog::ImportProgress {
                    import_log,
                    ..
                } = &mut app.active_dialog
                {
                    import_log.push(format!("✗ Failed: {}", event.summary));
                }
            }
        }
    }

    info!(
        "handle_confirm_import: Successfully imported {} events (skipped {} duplicates)",
        imported_count, skipped_count
    );

    // Refresh the calendar view
    app.refresh_cached_events();

    // Get calendar name for display
    let calendar_name = app.calendar_manager.sources()
        .iter()
        .find(|cal| cal.info().id == target_calendar_id)
        .map(|cal| cal.info().name.clone())
        .unwrap_or_else(|| target_calendar_id.clone());

    // Show import result dialog
    let success = imported_count > 0 || skipped_count > 0;
    let failed_count = events.len() - imported_count - skipped_count;

    app.active_dialog = ActiveDialog::ImportResult {
        success,
        imported_count,
        skipped_count,
        failed_count,
        source_file_name,
        calendar_name,
        imported_uids,
        calendar_id: target_calendar_id,
        error_message: None,
    };

    Task::none()
}

/// Handle cancel import message
pub fn handle_cancel_import(app: &mut CosmicCalendar) -> Task<Message> {
    debug!("handle_cancel_import: Canceling import");
    DialogManager::close(&mut app.active_dialog);
    Task::none()
}

/// Handle cancel import progress message - rollback imported events
pub fn handle_cancel_import_progress(app: &mut CosmicCalendar) -> Task<Message> {
    info!("handle_cancel_import_progress: Canceling import and rolling back");

    // Extract imported UIDs and calendar ID from progress dialog
    let (imported_uids, _calendar_id) = match &app.active_dialog {
        ActiveDialog::ImportProgress {
            imported_uids,
            calendar_id,
            ..
        } => (imported_uids.clone(), calendar_id.clone()),
        _ => {
            warn!("handle_cancel_import_progress: Not in import progress state");
            DialogManager::close(&mut app.active_dialog);
            return Task::none();
        }
    };

    info!(
        "handle_cancel_import_progress: Rolling back {} imported events",
        imported_uids.len()
    );

    // Delete all imported events (rollback)
    for uid in &imported_uids {
        match EventHandler::delete_event(&mut app.calendar_manager, uid) {
            Ok(_) => {
                debug!("handle_cancel_import_progress: Rolled back event uid={}", uid);
            }
            Err(e) => {
                error!("handle_cancel_import_progress: Failed to rollback event uid={}: {}", uid, e);
            }
        }
    }

    info!(
        "handle_cancel_import_progress: Rollback complete - deleted {} events",
        imported_uids.len()
    );

    // Refresh the calendar view
    app.refresh_cached_events();

    // Close the progress dialog
    DialogManager::close(&mut app.active_dialog);

    // TODO: Show cancellation notification
    Task::none()
}

/// Handle revert import message - rollback completed import
pub fn handle_revert_import(app: &mut CosmicCalendar) -> Task<Message> {
    info!("handle_revert_import: Reverting import");

    // Extract imported UIDs and calendar ID from result dialog
    let (imported_uids, _calendar_id) = match &app.active_dialog {
        ActiveDialog::ImportResult {
            imported_uids,
            calendar_id,
            ..
        } => (imported_uids.clone(), calendar_id.clone()),
        _ => {
            warn!("handle_revert_import: Not in import result state");
            DialogManager::close(&mut app.active_dialog);
            return Task::none();
        }
    };

    info!(
        "handle_revert_import: Reverting {} imported events",
        imported_uids.len()
    );

    // Delete all imported events (revert)
    for uid in &imported_uids {
        match EventHandler::delete_event(&mut app.calendar_manager, uid) {
            Ok(_) => {
                debug!("handle_revert_import: Reverted event uid={}", uid);
            }
            Err(e) => {
                error!("handle_revert_import: Failed to revert event uid={}: {}", uid, e);
            }
        }
    }

    info!(
        "handle_revert_import: Revert complete - deleted {} events",
        imported_uids.len()
    );

    // Refresh the calendar view
    app.refresh_cached_events();

    // Close the result dialog
    DialogManager::close(&mut app.active_dialog);

    // TODO: Show revert notification
    Task::none()
}
