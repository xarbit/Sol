//! Event management handlers (quick events and event dialog)
//!
//! These handlers delegate to the EventHandler service for actual event operations.
//! This ensures consistent validation, routing, and cache management.

use chrono::{NaiveDate, NaiveTime, TimeZone, Utc};
use cosmic::widget::{calendar::CalendarModel, text_editor};
use log::{debug, error, info, warn};
use uuid::Uuid;

use crate::app::{CosmicCalendar, EventDialogState};
use crate::caldav::{AlertTime, CalendarEvent, RepeatFrequency, TravelTime};
use crate::dialogs::{DialogAction, DialogManager, QuickEventResult};
use crate::services::EventHandler;

/// Commit the quick event being edited - create a new event in the selected calendar
/// Uses DialogManager to get the event data from ActiveDialog::QuickEvent
/// Supports both single-day and multi-day events (from drag selection)
pub fn handle_commit_quick_event(app: &mut CosmicCalendar) {
    debug!("handle_commit_quick_event: Starting");

    // Get the event data from DialogManager and clear the dialog state
    let result = DialogManager::handle_action(
        &mut app.active_dialog,
        DialogAction::CommitQuickEvent,
    );

    let Some(QuickEventResult { start_date, end_date, text }) = result else {
        debug!("handle_commit_quick_event: No quick event editing state");
        return;
    };

    // Don't create empty events
    let text = text.trim();
    if text.is_empty() {
        debug!("handle_commit_quick_event: Empty text, ignoring");
        return;
    }

    // Get the selected calendar ID
    let Some(calendar_id) = app.selected_calendar_id.clone() else {
        warn!("handle_commit_quick_event: No calendar selected for new event");
        return;
    };

    let is_multi_day = start_date != end_date;
    if is_multi_day {
        info!(
            "handle_commit_quick_event: Creating multi-day event '{}' from {} to {} in calendar '{}'",
            text, start_date, end_date, calendar_id
        );
    } else {
        info!(
            "handle_commit_quick_event: Creating event '{}' on {} in calendar '{}'",
            text, start_date, calendar_id
        );
    }

    // Create an all-day event
    // Use midnight UTC for start, end of day for end
    let start_time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
    let end_time = NaiveTime::from_hms_opt(23, 59, 59).unwrap();

    let start = Utc.from_utc_datetime(&start_date.and_time(start_time));
    let end = Utc.from_utc_datetime(&end_date.and_time(end_time));

    let event = CalendarEvent {
        uid: Uuid::new_v4().to_string(),
        summary: text.to_string(),
        location: None,
        all_day: true,
        start,
        end,
        travel_time: TravelTime::None,
        repeat: RepeatFrequency::Never,
        invitees: vec![],
        alert: AlertTime::None,
        alert_second: None,
        attachments: vec![],
        url: None,
        notes: None,
    };

    // Use EventHandler to add the event (handles validation, storage, and sync)
    if let Err(e) = EventHandler::add_event(&mut app.calendar_manager, &calendar_id, event) {
        error!("handle_commit_quick_event: Failed to add event: {}", e);
        return;
    }

    info!("handle_commit_quick_event: Event created successfully");
    // Refresh the cached events to show the new event
    app.refresh_cached_events();
}

/// Delete an event by its UID from all calendars
pub fn handle_delete_event(app: &mut CosmicCalendar, uid: String) {
    info!("handle_delete_event: Deleting event uid={}", uid);

    // Use EventHandler to delete the event (searches all calendars)
    if let Err(e) = EventHandler::delete_event(&mut app.calendar_manager, &uid) {
        error!("handle_delete_event: Failed to delete event: {}", e);
    }
    // Refresh cached events to reflect deletion
    app.refresh_cached_events();
}

/// Select an event for viewing/editing
/// Toggles selection - clicking the same event again deselects it
pub fn handle_select_event(app: &mut CosmicCalendar, uid: String) {
    debug!("handle_select_event: uid={}", uid);

    // Toggle selection: if already selected, deselect
    if app.selected_event_uid.as_ref() == Some(&uid) {
        app.selected_event_uid = None;
        debug!("handle_select_event: Deselected event");
    } else {
        app.selected_event_uid = Some(uid);
        debug!("handle_select_event: Selected event");
    }
}

// === Event Drag Handlers ===

/// Start dragging an event to move it to a new date
/// Takes the event UID, original date, summary (for preview), and color (for preview)
pub fn handle_drag_event_start(
    app: &mut CosmicCalendar,
    uid: String,
    original_date: NaiveDate,
    summary: String,
    color: String,
) {
    debug!("handle_drag_event_start: uid={}, date={}, summary={}", uid, original_date, summary);

    // Cancel any day selection in progress
    app.selection_state.cancel();

    // Start the drag operation with display info for the preview
    app.event_drag_state.start(uid, original_date, summary, color);
}

/// Update the drag target date as user drags over cells
pub fn handle_drag_event_update(app: &mut CosmicCalendar, target_date: NaiveDate) {
    app.event_drag_state.update(target_date);
}

/// End the drag operation - move the event if target differs from original
/// If the event wasn't moved (same date), treat it as a selection click
pub fn handle_drag_event_end(app: &mut CosmicCalendar) {
    // Get the event UID before ending the drag (for selection fallback)
    let event_uid = app.event_drag_state.event_uid.clone();

    // Try to end the drag and get move info
    let move_result = app.event_drag_state.end();

    match move_result {
        Some((uid, original_date, new_date)) => {
            // Event was dragged to a different date - move it
            info!("handle_drag_event_end: Moving event {} from {} to {}", uid, original_date, new_date);

            // Calculate the offset in days
            let offset = (new_date - original_date).num_days();

            // Find the event and move it
            if let Ok((event, calendar_id)) = EventHandler::find_event(&app.calendar_manager, &uid) {
                // Calculate new start and end times by adding the offset
                let new_start = event.start + chrono::Duration::days(offset);
                let new_end = event.end + chrono::Duration::days(offset);

                // Create updated event with new dates
                let updated_event = crate::caldav::CalendarEvent {
                    start: new_start,
                    end: new_end,
                    ..event
                };

                // Update the event
                if let Err(e) = EventHandler::update_event(&mut app.calendar_manager, &calendar_id, updated_event) {
                    error!("handle_drag_event_end: Failed to move event: {}", e);
                    return;
                }

                info!("handle_drag_event_end: Event moved successfully");
                app.refresh_cached_events();
            } else {
                warn!("handle_drag_event_end: Event not found: {}", uid);
            }
        }
        None => {
            // Event wasn't moved (clicked and released on same date) - treat as selection
            if let Some(uid) = event_uid {
                debug!("handle_drag_event_end: No move, selecting event {}", uid);
                // Toggle selection like regular click
                if app.selected_event_uid.as_ref() == Some(&uid) {
                    app.selected_event_uid = None;
                } else {
                    app.selected_event_uid = Some(uid);
                }
            }
        }
    }
}

/// Cancel the drag operation
pub fn handle_drag_event_cancel(app: &mut CosmicCalendar) {
    debug!("handle_drag_event_cancel: Cancelling drag");
    app.event_drag_state.cancel();
}

/// Start editing a quick event on a specific date
/// Uses DialogManager to open ActiveDialog::QuickEvent
pub fn handle_start_quick_event(app: &mut CosmicCalendar, date: NaiveDate) {
    debug!("handle_start_quick_event: Starting quick event for {}", date);
    DialogManager::handle_action(&mut app.active_dialog, DialogAction::StartQuickEvent(date));
}

/// Update the quick event text while editing
/// Uses DialogManager to update the text in ActiveDialog::QuickEvent
pub fn handle_quick_event_text_changed(app: &mut CosmicCalendar, text: String) {
    DialogManager::handle_action(&mut app.active_dialog, DialogAction::QuickEventTextChanged(text));
}

/// Cancel quick event editing
/// Uses DialogManager to close the ActiveDialog::QuickEvent
pub fn handle_cancel_quick_event(app: &mut CosmicCalendar) {
    debug!("handle_cancel_quick_event: Cancelling");
    DialogManager::close(&mut app.active_dialog);
}

// === Event Dialog Handlers ===

/// Open the event dialog for creating a new event
pub fn handle_open_new_event_dialog(app: &mut CosmicCalendar) {
    debug!("handle_open_new_event_dialog: Opening new event dialog");
    let today = app.selected_date;

    // Default to 9:00 AM - 10:00 AM
    let default_start_time = NaiveTime::from_hms_opt(9, 0, 0);
    let default_end_time = NaiveTime::from_hms_opt(10, 0, 0);

    // Use selected calendar or first available
    let calendar_id = app
        .selected_calendar_id
        .clone()
        .or_else(|| {
            app.calendar_manager
                .sources()
                .first()
                .map(|c| c.info().id.clone())
        })
        .unwrap_or_default();

    app.event_dialog = Some(EventDialogState {
        editing_uid: None,
        title: String::new(),
        location: String::new(),
        all_day: false,
        start_date: today,
        start_date_input: today.format("%Y-%m-%d").to_string(),
        start_time: default_start_time,
        start_time_input: default_start_time
            .map(|t| t.format("%H:%M").to_string())
            .unwrap_or_else(|| "09:00".to_string()),
        end_date: today,
        end_date_input: today.format("%Y-%m-%d").to_string(),
        end_time: default_end_time,
        end_time_input: default_end_time
            .map(|t| t.format("%H:%M").to_string())
            .unwrap_or_else(|| "10:00".to_string()),
        travel_time: TravelTime::None,
        repeat: RepeatFrequency::Never,
        calendar_id,
        invitees: vec![],
        invitee_input: String::new(),
        alert: AlertTime::None,
        alert_second: None,
        attachments: vec![],
        url: String::new(),
        notes_content: text_editor::Content::new(),
        editing_field: None,
        start_date_picker_open: false,
        start_date_calendar: CalendarModel::new(today, today),
        end_date_picker_open: false,
        end_date_calendar: CalendarModel::new(today, today),
        start_time_picker_open: false,
        end_time_picker_open: false,
    });
}

/// Open the event dialog for editing an existing event
pub fn handle_open_edit_event_dialog(app: &mut CosmicCalendar, uid: String) {
    debug!("handle_open_edit_event_dialog: Opening edit dialog for uid={}", uid);

    // Use EventHandler to find the event across all calendars
    let (event, calendar_id) = match EventHandler::find_event(&app.calendar_manager, &uid) {
        Ok(result) => result,
        Err(e) => {
            warn!("handle_open_edit_event_dialog: Event not found: {}", e);
            return;
        }
    };

    info!("handle_open_edit_event_dialog: Found event '{}' in calendar '{}'", event.summary, calendar_id);

    // Convert UTC times to local dates/times
    let start_date = event.start.date_naive();
    let end_date = event.end.date_naive();
    let start_time = Some(event.start.time());
    let end_time = Some(event.end.time());

    let actual_start_time = if event.all_day { None } else { start_time };
    let actual_end_time = if event.all_day { None } else { end_time };

    app.event_dialog = Some(EventDialogState {
        editing_uid: Some(uid),
        title: event.summary,
        location: event.location.unwrap_or_default(),
        all_day: event.all_day,
        start_date,
        start_date_input: start_date.format("%Y-%m-%d").to_string(),
        start_time: actual_start_time,
        start_time_input: actual_start_time
            .map(|t| t.format("%H:%M").to_string())
            .unwrap_or_else(|| "09:00".to_string()),
        end_date,
        end_date_input: end_date.format("%Y-%m-%d").to_string(),
        end_time: actual_end_time,
        end_time_input: actual_end_time
            .map(|t| t.format("%H:%M").to_string())
            .unwrap_or_else(|| "10:00".to_string()),
        travel_time: event.travel_time,
        repeat: event.repeat,
        calendar_id,
        invitees: event.invitees,
        invitee_input: String::new(),
        alert: event.alert,
        alert_second: event.alert_second,
        attachments: event.attachments,
        url: event.url.unwrap_or_default(),
        notes_content: text_editor::Content::with_text(&event.notes.unwrap_or_default()),
        editing_field: None,
        start_date_picker_open: false,
        start_date_calendar: CalendarModel::new(start_date, start_date),
        end_date_picker_open: false,
        end_date_calendar: CalendarModel::new(end_date, end_date),
        start_time_picker_open: false,
        end_time_picker_open: false,
    });
}

/// Confirm the event dialog - create or update the event
pub fn handle_confirm_event_dialog(app: &mut CosmicCalendar) {
    let Some(dialog) = app.event_dialog.take() else {
        return;
    };

    let is_edit = dialog.editing_uid.is_some();
    debug!("handle_confirm_event_dialog: {} event", if is_edit { "Updating" } else { "Creating" });

    // Validate: title is required
    let title = dialog.title.trim();
    if title.is_empty() {
        warn!("handle_confirm_event_dialog: Empty title, returning dialog");
        // Put dialog back - can't save without title
        app.event_dialog = Some(dialog);
        return;
    }

    // Build start and end times
    let start_time = if dialog.all_day {
        NaiveTime::from_hms_opt(0, 0, 0).unwrap()
    } else {
        dialog.start_time.unwrap_or_else(|| NaiveTime::from_hms_opt(9, 0, 0).unwrap())
    };

    let end_time = if dialog.all_day {
        NaiveTime::from_hms_opt(23, 59, 59).unwrap()
    } else {
        dialog.end_time.unwrap_or_else(|| NaiveTime::from_hms_opt(10, 0, 0).unwrap())
    };

    let start = Utc.from_utc_datetime(&dialog.start_date.and_time(start_time));
    let end = Utc.from_utc_datetime(&dialog.end_date.and_time(end_time));

    let event = CalendarEvent {
        uid: dialog.editing_uid.clone().unwrap_or_else(|| Uuid::new_v4().to_string()),
        summary: title.to_string(),
        location: if dialog.location.is_empty() {
            None
        } else {
            Some(dialog.location)
        },
        all_day: dialog.all_day,
        start,
        end,
        travel_time: dialog.travel_time,
        repeat: dialog.repeat,
        invitees: dialog.invitees,
        alert: dialog.alert,
        alert_second: dialog.alert_second,
        attachments: dialog.attachments,
        url: if dialog.url.is_empty() {
            None
        } else {
            Some(dialog.url)
        },
        notes: {
            let notes_text = dialog.notes_content.text();
            if notes_text.trim().is_empty() {
                None
            } else {
                Some(notes_text)
            }
        },
    };

    // Use EventHandler for create or update
    let result = if dialog.editing_uid.is_some() {
        info!("handle_confirm_event_dialog: Updating event '{}' in calendar '{}'", title, dialog.calendar_id);
        // Update existing event (EventHandler handles delete + add)
        EventHandler::update_event(&mut app.calendar_manager, &dialog.calendar_id, event)
    } else {
        info!("handle_confirm_event_dialog: Creating event '{}' in calendar '{}'", title, dialog.calendar_id);
        // Create new event
        EventHandler::add_event(&mut app.calendar_manager, &dialog.calendar_id, event)
    };

    match result {
        Ok(()) => {
            info!("handle_confirm_event_dialog: Event saved successfully");
            // Refresh cached events
            app.refresh_cached_events();
        }
        Err(e) => {
            error!("handle_confirm_event_dialog: Failed to save event: {}", e);
        }
    }
}

/// Cancel the event dialog
pub fn handle_cancel_event_dialog(app: &mut CosmicCalendar) {
    debug!("handle_cancel_event_dialog: Cancelling event dialog");
    app.event_dialog = None;
}
