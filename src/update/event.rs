//! Event management handlers (quick events and event dialog)
//!
//! These handlers delegate to the EventHandler service for actual event operations.
//! This ensures consistent validation, routing, and cache management.

use chrono::{NaiveDate, NaiveTime, TimeZone, Timelike, Utc};
use cosmic::widget::{calendar::CalendarModel, text_editor};
use log::{debug, error, info, warn};
use uuid::Uuid;

use crate::app::{CosmicCalendar, EventDialogState};
use crate::caldav::{AlertTime, CalendarEvent, RepeatFrequency, TravelTime};
use crate::dialogs::{DialogAction, DialogManager, QuickEventResult};
use crate::services::EventHandler;

/// Extract the master UID from an occurrence UID
/// Occurrence UIDs have format "master-uid_YYYYMMDD" for recurring events
/// Returns the original UID if it doesn't match the occurrence pattern
pub fn extract_master_uid(uid: &str) -> &str {
    // Check if the UID ends with _YYYYMMDD (8 digits after underscore)
    if let Some(pos) = uid.rfind('_') {
        let suffix = &uid[pos + 1..];
        // Verify it's exactly 8 digits (date format)
        if suffix.len() == 8 && suffix.chars().all(|c| c.is_ascii_digit()) {
            return &uid[..pos];
        }
    }
    uid
}

/// Extract the occurrence date from an occurrence UID
/// Occurrence UIDs have format "master-uid_YYYYMMDD" for recurring events
/// Returns None if the UID doesn't match the occurrence pattern
pub fn extract_occurrence_date(uid: &str) -> Option<NaiveDate> {
    // Check if the UID ends with _YYYYMMDD (8 digits after underscore)
    if let Some(pos) = uid.rfind('_') {
        let suffix = &uid[pos + 1..];
        // Verify it's exactly 8 digits (date format)
        if suffix.len() == 8 && suffix.chars().all(|c| c.is_ascii_digit()) {
            // Parse YYYYMMDD
            let year: i32 = suffix[0..4].parse().ok()?;
            let month: u32 = suffix[4..6].parse().ok()?;
            let day: u32 = suffix[6..8].parse().ok()?;
            return NaiveDate::from_ymd_opt(year, month, day);
        }
    }
    None
}

/// Commit the quick event being edited - create a new event in the selected calendar
/// Uses DialogManager to get the event data from ActiveDialog::QuickEvent
/// Supports both single-day and multi-day events (from drag selection)
/// Also supports timed events (from time slot selection in week/day view)
pub fn handle_commit_quick_event(app: &mut CosmicCalendar) {
    debug!("handle_commit_quick_event: Starting");

    // Get the event data from DialogManager and clear the dialog state
    let result = DialogManager::handle_action(
        &mut app.active_dialog,
        DialogAction::CommitQuickEvent,
    );

    let Some(QuickEventResult { start_date, end_date, start_time: evt_start_time, end_time: evt_end_time, text }) = result else {
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

    // Determine if this is a timed event or all-day event
    let is_timed = evt_start_time.is_some();
    let is_multi_day = start_date != end_date;

    if is_timed {
        info!(
            "handle_commit_quick_event: Creating timed event '{}' on {} from {:?} to {:?} in calendar '{}'",
            text, start_date, evt_start_time, evt_end_time, calendar_id
        );
    } else if is_multi_day {
        info!(
            "handle_commit_quick_event: Creating multi-day event '{}' from {} to {} in calendar '{}'",
            text, start_date, end_date, calendar_id
        );
    } else {
        info!(
            "handle_commit_quick_event: Creating all-day event '{}' on {} in calendar '{}'",
            text, start_date, calendar_id
        );
    }

    // Set times based on whether this is a timed event
    let (start_time, end_time, all_day) = if let (Some(st), Some(et)) = (evt_start_time, evt_end_time) {
        // Timed event - use the specified times
        (st, et, false)
    } else {
        // All-day event - use midnight to end of day
        let midnight = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
        let end_of_day = NaiveTime::from_hms_opt(23, 59, 59).unwrap();
        (midnight, end_of_day, true)
    };

    let start = Utc.from_utc_datetime(&start_date.and_time(start_time));
    let end = Utc.from_utc_datetime(&end_date.and_time(end_time));

    let event = CalendarEvent {
        uid: Uuid::new_v4().to_string(),
        summary: text.to_string(),
        location: None,
        all_day,
        start,
        end,
        travel_time: TravelTime::None,
        repeat: RepeatFrequency::Never,
        repeat_until: None,
        exception_dates: vec![],
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
/// This implements a robust deletion with verification and guaranteed UI refresh
/// For recurring events, the occurrence UID (format: master-uid_YYYYMMDD) is converted
/// to the master UID before deletion, which deletes all occurrences.
pub fn handle_delete_event(app: &mut CosmicCalendar, uid: String) {
    // Extract master UID for recurring events (occurrence UIDs have format master-uid_YYYYMMDD)
    let master_uid = extract_master_uid(&uid);
    info!("handle_delete_event: Deleting event uid={} (master_uid={})", uid, master_uid);

    // Clear selection if deleting the selected event (check both occurrence and master UID)
    if let Some(selected) = &app.selected_event_uid {
        if selected == &uid || extract_master_uid(selected) == master_uid {
            app.selected_event_uid = None;
            debug!("handle_delete_event: Cleared selection for deleted event");
        }
    }

    // Use EventHandler to delete the event (searches all calendars)
    // Use master_uid to find the actual event in the database
    // Now returns Result<bool> with verification
    match EventHandler::delete_event(&mut app.calendar_manager, master_uid) {
        Ok(was_deleted) => {
            if was_deleted {
                info!("handle_delete_event: Event deleted and verified");
            } else {
                info!("handle_delete_event: Event was not found (may already be deleted)");
            }
        }
        Err(e) => {
            error!("handle_delete_event: Failed to delete event: {}", e);
            // Still refresh UI even on error to ensure consistency
        }
    }

    // Force complete cache refresh - clear and rebuild
    // This ensures UI state matches database state
    app.cached_week_events.clear();
    app.cached_month_events.clear();
    app.refresh_cached_events();

    info!("handle_delete_event: UI cache refreshed");
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
            // Extract master UID for recurring events (occurrence UIDs have format master-uid_YYYYMMDD)
            let master_uid = extract_master_uid(&uid);
            info!("handle_drag_event_end: Moving event {} (master_uid={}) from {} to {}", uid, master_uid, original_date, new_date);

            // Calculate the offset in days
            let offset = (new_date - original_date).num_days();

            // Find the event and move it (use master UID for recurring events)
            if let Ok((event, calendar_id)) = EventHandler::find_event(&app.calendar_manager, master_uid) {
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

/// Start editing a quick timed event with specific start and end times
/// Uses DialogManager to open ActiveDialog::QuickEvent with time info
pub fn handle_start_quick_timed_event(app: &mut CosmicCalendar, date: NaiveDate, start_time: NaiveTime, end_time: NaiveTime) {
    debug!("handle_start_quick_timed_event: Starting timed quick event for {} from {:?} to {:?}", date, start_time, end_time);
    DialogManager::handle_action(
        &mut app.active_dialog,
        DialogAction::StartQuickTimedEvent { date, start_time, end_time },
    );
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

    // Default to current time (rounded to 5 minutes) and +1 hour for end time
    let now = chrono::Local::now().time();
    let rounded_minute = (now.minute() / 5) * 5;
    let default_start_time = NaiveTime::from_hms_opt(now.hour(), rounded_minute, 0);
    let default_end_time = default_start_time.map(|t| {
        let new_hour = (t.hour() + 1) % 24;
        NaiveTime::from_hms_opt(new_hour, t.minute(), 0).unwrap_or(t)
    });

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
    // Extract master UID for recurring events (occurrence UIDs have format master-uid_YYYYMMDD)
    let master_uid = extract_master_uid(&uid);
    debug!("handle_open_edit_event_dialog: Opening edit dialog for uid={} (master_uid={})", uid, master_uid);

    // Use EventHandler to find the event across all calendars (use master UID)
    let (event, calendar_id) = match EventHandler::find_event(&app.calendar_manager, master_uid) {
        Ok(result) => result,
        Err(e) => {
            warn!("handle_open_edit_event_dialog: Event not found: {} (master_uid={})", e, master_uid);
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
        repeat_until: None, // TODO: Add to dialog state
        exception_dates: vec![], // Exception dates are preserved when editing existing events
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
