//! Event Handler - Middleware for event operations.
//!
//! The EventHandler sits between the UI/update layer and the protocol layer.
//! It provides a unified interface for event CRUD operations, handling:
//!
//! - Routing events to the correct protocol (local vs remote)
//! - Event validation before saving
//! - Sync and conflict resolution
//! - Cache invalidation coordination
//!
//! # Architecture
//!
//! ```text
//! UI (views/components)
//!         │
//!         ▼
//! Update Handlers (update/event.rs)
//!         │
//!         ▼
//! ┌───────────────────────────────────────┐
//! │         EventHandler (this module)     │
//! │  - Validates events                    │
//! │  - Routes to correct calendar/protocol │
//! │  - Handles sync                        │
//! └───────────────────────────────────────┘
//!         │
//!         ▼
//! CalendarManager → CalendarSource → Protocol
//! ```

use crate::caldav::CalendarEvent;
use crate::calendars::CalendarManager;
use log::{debug, error, info, trace, warn};
use std::error::Error;

/// Result type for event handler operations
pub type EventResult<T> = Result<T, EventError>;

/// Error types for event operations
#[derive(Debug)]
pub enum EventError {
    /// Event validation failed
    ValidationError(String),
    /// Calendar not found
    CalendarNotFound(String),
    /// Event not found
    EventNotFound(String),
    /// Storage/protocol error
    StorageError(String),
    /// Sync error
    SyncError(String),
}

impl std::fmt::Display for EventError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            EventError::CalendarNotFound(id) => write!(f, "Calendar not found: {}", id),
            EventError::EventNotFound(uid) => write!(f, "Event not found: {}", uid),
            EventError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            EventError::SyncError(msg) => write!(f, "Sync error: {}", msg),
        }
    }
}

impl Error for EventError {}

/// Event Handler - centralized middleware for event operations.
///
/// This struct provides methods for all event CRUD operations,
/// routing them through the appropriate calendar and protocol.
pub struct EventHandler;

impl EventHandler {
    /// Validate an event before saving.
    /// Returns Ok(()) if valid, or an error describing what's wrong.
    pub fn validate_event(event: &CalendarEvent) -> EventResult<()> {
        trace!("EventHandler: Validating event uid={}", event.uid);

        // Title/summary is required
        if event.summary.trim().is_empty() {
            warn!("EventHandler: Validation failed - empty title for uid={}", event.uid);
            return Err(EventError::ValidationError(
                "Event title is required".to_string(),
            ));
        }

        // End time must be after start time
        if event.end < event.start {
            warn!("EventHandler: Validation failed - end before start for uid={}", event.uid);
            return Err(EventError::ValidationError(
                "End time must be after start time".to_string(),
            ));
        }

        // UID must not be empty
        if event.uid.is_empty() {
            warn!("EventHandler: Validation failed - empty UID");
            return Err(EventError::ValidationError(
                "Event UID is required".to_string(),
            ));
        }

        debug!("EventHandler: Event validation passed for uid={}", event.uid);
        Ok(())
    }

    /// Add a new event to a calendar.
    ///
    /// This method:
    /// 1. Validates the event
    /// 2. Finds the target calendar
    /// 3. Adds the event via the calendar's protocol
    /// 4. Syncs the calendar
    pub fn add_event(
        calendar_manager: &mut CalendarManager,
        calendar_id: &str,
        event: CalendarEvent,
    ) -> EventResult<()> {
        info!("EventHandler: Adding event '{}' (uid={}) to calendar '{}'",
              event.summary, event.uid, calendar_id);

        // Validate event
        Self::validate_event(&event)?;

        // Find the calendar
        let calendar = calendar_manager
            .sources_mut()
            .iter_mut()
            .find(|c| c.info().id == calendar_id)
            .ok_or_else(|| {
                error!("EventHandler: Calendar '{}' not found", calendar_id);
                EventError::CalendarNotFound(calendar_id.to_string())
            })?;

        debug!("EventHandler: Found calendar '{}', adding event", calendar.info().name);

        // Add event via calendar (which routes to protocol)
        calendar
            .add_event(event.clone())
            .map_err(|e| {
                error!("EventHandler: Failed to add event: {}", e);
                EventError::StorageError(e.to_string())
            })?;

        // Sync to persist
        debug!("EventHandler: Syncing calendar after add");
        calendar
            .sync()
            .map_err(|e| {
                error!("EventHandler: Sync failed after add: {}", e);
                EventError::SyncError(e.to_string())
            })?;

        info!("EventHandler: Successfully added event uid={}", event.uid);
        Ok(())
    }

    /// Update an existing event.
    ///
    /// This method:
    /// 1. Validates the event
    /// 2. Finds and removes the old event (from any calendar)
    /// 3. Adds the updated event to the target calendar
    /// 4. Syncs affected calendars
    pub fn update_event(
        calendar_manager: &mut CalendarManager,
        calendar_id: &str,
        event: CalendarEvent,
    ) -> EventResult<()> {
        info!("EventHandler: Updating event '{}' (uid={}) in calendar '{}'",
              event.summary, event.uid, calendar_id);

        // Validate event
        Self::validate_event(&event)?;

        let uid = event.uid.clone();

        // First, delete the old event from wherever it exists
        debug!("EventHandler: Deleting old version of event uid={}", uid);
        let _ = Self::delete_event(calendar_manager, &uid)?;

        // Add to the (possibly new) target calendar
        let calendar = calendar_manager
            .sources_mut()
            .iter_mut()
            .find(|c| c.info().id == calendar_id)
            .ok_or_else(|| {
                error!("EventHandler: Target calendar '{}' not found for update", calendar_id);
                EventError::CalendarNotFound(calendar_id.to_string())
            })?;

        debug!("EventHandler: Adding updated event to calendar '{}'", calendar.info().name);
        calendar
            .add_event(event.clone())
            .map_err(|e| {
                error!("EventHandler: Failed to add updated event: {}", e);
                EventError::StorageError(e.to_string())
            })?;

        calendar
            .sync()
            .map_err(|e| {
                error!("EventHandler: Sync failed after update: {}", e);
                EventError::SyncError(e.to_string())
            })?;

        info!("EventHandler: Successfully updated event uid={}", uid);
        Ok(())
    }

    /// Delete an event by UID from all calendars.
    ///
    /// This searches all calendars for the event and deletes it.
    /// Returns true if an event was deleted, false if not found.
    pub fn delete_event(
        calendar_manager: &mut CalendarManager,
        uid: &str,
    ) -> EventResult<bool> {
        info!("EventHandler: Deleting event uid={}", uid);
        let mut deleted = false;

        for calendar in calendar_manager.sources_mut().iter_mut() {
            match calendar.delete_event(uid) {
                Ok(()) => {
                    info!("EventHandler: Deleted event uid={} from calendar '{}'",
                          uid, calendar.info().name);
                    // Sync after successful deletion to force cache refresh
                    if let Err(e) = calendar.sync() {
                        warn!("EventHandler: Sync failed after delete: {}", e);
                    }
                    deleted = true;
                    break;
                }
                Err(_) => {
                    trace!("EventHandler: Event uid={} not in calendar '{}'",
                           uid, calendar.info().name);
                    continue;
                }
            }
        }

        if !deleted {
            debug!("EventHandler: Event uid={} not found in any calendar (may already be deleted)", uid);
        }

        // Force sync ALL calendars to ensure consistent state
        // This handles edge cases where caches might be stale
        for calendar in calendar_manager.sources_mut().iter_mut() {
            if let Err(e) = calendar.sync() {
                warn!("EventHandler: Sync failed for calendar '{}': {}", calendar.info().name, e);
            }
        }

        // Verify deletion by checking if event still exists
        let still_exists = Self::find_event(calendar_manager, uid).is_ok();
        if still_exists {
            error!("EventHandler: Event uid={} still exists after deletion attempt!", uid);
            return Err(EventError::StorageError(format!(
                "Event {} was not successfully deleted from database",
                uid
            )));
        }

        info!("EventHandler: Successfully verified event uid={} is deleted", uid);
        Ok(deleted)
    }

    /// Find an event by UID across all calendars.
    ///
    /// Returns the event and the calendar ID it was found in.
    pub fn find_event(
        calendar_manager: &CalendarManager,
        uid: &str,
    ) -> EventResult<(CalendarEvent, String)> {
        debug!("EventHandler: Searching for event uid={}", uid);

        for calendar in calendar_manager.sources() {
            if let Ok(events) = calendar.fetch_events() {
                if let Some(event) = events.iter().find(|e| e.uid == uid) {
                    debug!("EventHandler: Found event uid={} in calendar '{}'",
                           uid, calendar.info().name);
                    return Ok((event.clone(), calendar.info().id.clone()));
                }
            }
        }

        debug!("EventHandler: Event uid={} not found in any calendar", uid);
        Err(EventError::EventNotFound(uid.to_string()))
    }

    /// Sync all calendars.
    #[allow(dead_code)] // Reserved for CalDAV/remote calendar sync
    pub fn sync_all(calendar_manager: &mut CalendarManager) -> EventResult<()> {
        info!("EventHandler: Syncing all calendars");
        let mut sync_count = 0;

        for calendar in calendar_manager.sources_mut().iter_mut() {
            if calendar.is_enabled() {
                debug!("EventHandler: Syncing calendar '{}'", calendar.info().name);
                calendar
                    .sync()
                    .map_err(|e| {
                        error!("EventHandler: Sync failed for '{}': {}", calendar.info().name, e);
                        EventError::SyncError(e.to_string())
                    })?;
                sync_count += 1;
            }
        }

        info!("EventHandler: Synced {} calendars", sync_count);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::caldav::{AlertTime, RepeatFrequency, TravelTime};
    use chrono::{TimeZone, Utc};

    fn create_test_event(uid: &str, summary: &str) -> CalendarEvent {
        CalendarEvent {
            uid: uid.to_string(),
            summary: summary.to_string(),
            location: None,
            all_day: false,
            start: Utc.with_ymd_and_hms(2025, 11, 30, 10, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2025, 11, 30, 11, 0, 0).unwrap(),
            travel_time: TravelTime::None,
            repeat: RepeatFrequency::Never,
            invitees: vec![],
            alert: AlertTime::None,
            alert_second: None,
            attachments: vec![],
            url: None,
            notes: None,
        }
    }

    #[test]
    fn test_validate_event_success() {
        let event = create_test_event("test-1", "Valid Event");
        assert!(EventHandler::validate_event(&event).is_ok());
    }

    #[test]
    fn test_validate_event_empty_title() {
        let event = create_test_event("test-1", "");
        let result = EventHandler::validate_event(&event);
        assert!(matches!(result, Err(EventError::ValidationError(_))));
    }

    #[test]
    fn test_validate_event_empty_uid() {
        let mut event = create_test_event("", "Test Event");
        event.uid = String::new();
        let result = EventHandler::validate_event(&event);
        assert!(matches!(result, Err(EventError::ValidationError(_))));
    }

    #[test]
    fn test_validate_event_end_before_start() {
        let mut event = create_test_event("test-1", "Test Event");
        event.end = Utc.with_ymd_and_hms(2025, 11, 30, 9, 0, 0).unwrap(); // Before start
        let result = EventHandler::validate_event(&event);
        assert!(matches!(result, Err(EventError::ValidationError(_))));
    }
}
