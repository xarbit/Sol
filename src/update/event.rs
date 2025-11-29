//! Event management handlers (create, delete quick events)

use chrono::{NaiveDate, NaiveTime, TimeZone, Utc};
use uuid::Uuid;

use crate::app::CosmicCalendar;
use crate::caldav::CalendarEvent;

/// Commit the quick event being edited - create a new event in the selected calendar
pub fn handle_commit_quick_event(app: &mut CosmicCalendar) {
    // Get the event data and clear the editing state
    let Some((date, text)) = app.quick_event_editing.take() else {
        return;
    };

    // Don't create empty events
    let text = text.trim();
    if text.is_empty() {
        return;
    }

    // Get the selected calendar ID
    let Some(calendar_id) = app.selected_calendar_id.clone() else {
        eprintln!("No calendar selected for new event");
        return;
    };

    // Create an all-day event for the selected date
    // Use midnight UTC for start, end of day for end
    let start_time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
    let end_time = NaiveTime::from_hms_opt(23, 59, 59).unwrap();

    let start = Utc.from_utc_datetime(&date.and_time(start_time));
    let end = Utc.from_utc_datetime(&date.and_time(end_time));

    let event = CalendarEvent {
        uid: Uuid::new_v4().to_string(),
        summary: text.to_string(),
        description: None,
        start,
        end,
        location: None,
    };

    // Find the calendar and add the event
    if let Some(calendar) = app
        .calendar_manager
        .sources_mut()
        .iter_mut()
        .find(|c| c.info().id == calendar_id)
    {
        if let Err(e) = calendar.add_event(event) {
            eprintln!("Failed to add event: {}", e);
            return;
        }
        // Sync to persist the event
        if let Err(e) = calendar.sync() {
            eprintln!("Failed to sync calendar: {}", e);
        }
    }

    // Refresh the cached events to show the new event
    app.refresh_cached_events();
}

/// Delete an event by its UID from all calendars
pub fn handle_delete_event(app: &mut CosmicCalendar, uid: String) {
    for calendar in app.calendar_manager.sources_mut().iter_mut() {
        if calendar.delete_event(&uid).is_ok() {
            // Sync to persist the deletion
            let _ = calendar.sync();
            break;
        }
    }
    // Refresh cached events to reflect deletion
    app.refresh_cached_events();
}

/// Start editing a quick event on a specific date
pub fn handle_start_quick_event(app: &mut CosmicCalendar, date: NaiveDate) {
    app.quick_event_editing = Some((date, String::new()));
}

/// Update the quick event text while editing
pub fn handle_quick_event_text_changed(app: &mut CosmicCalendar, text: String) {
    if let Some((date, _)) = app.quick_event_editing.take() {
        app.quick_event_editing = Some((date, text));
    }
}

/// Cancel quick event editing
pub fn handle_cancel_quick_event(app: &mut CosmicCalendar) {
    app.quick_event_editing = None;
}
