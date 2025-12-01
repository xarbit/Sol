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
        Self::delete_event(calendar_manager, &uid)?;

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
    pub fn delete_event(
        calendar_manager: &mut CalendarManager,
        uid: &str,
    ) -> EventResult<()> {
        info!("EventHandler: Deleting event uid={}", uid);
        let mut deleted = false;

        for calendar in calendar_manager.sources_mut().iter_mut() {
            match calendar.delete_event(uid) {
                Ok(()) => {
                    info!("EventHandler: Deleted event uid={} from calendar '{}'",
                          uid, calendar.info().name);
                    // Sync after successful deletion
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

        Ok(())
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

    // ==================== Event Validation Tests ====================

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
    fn test_validate_event_whitespace_only_title() {
        let event = create_test_event("test-1", "   ");
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

    #[test]
    fn test_validate_event_same_start_and_end() {
        let mut event = create_test_event("test-1", "Zero Duration Event");
        event.end = event.start; // Same time - should be valid (zero duration)
        let result = EventHandler::validate_event(&event);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_event_with_all_fields() {
        let mut event = create_test_event("test-full", "Complete Event");
        event.location = Some("Test Location".to_string());
        event.all_day = true;
        event.notes = Some("Test notes".to_string());
        event.url = Some("https://example.com".to_string());
        event.invitees = vec!["test@example.com".to_string()];
        event.travel_time = TravelTime::ThirtyMinutes;
        event.repeat = RepeatFrequency::Weekly;
        event.alert = AlertTime::FifteenMinutes;
        event.alert_second = Some(AlertTime::OneHour);
        event.attachments = vec!["file.pdf".to_string()];

        let result = EventHandler::validate_event(&event);
        assert!(result.is_ok());
    }

    // ==================== EventError Display Tests ====================

    #[test]
    fn test_event_error_display_validation() {
        let error = EventError::ValidationError("Test validation error".to_string());
        assert_eq!(error.to_string(), "Validation error: Test validation error");
    }

    #[test]
    fn test_event_error_display_calendar_not_found() {
        let error = EventError::CalendarNotFound("cal-123".to_string());
        assert_eq!(error.to_string(), "Calendar not found: cal-123");
    }

    #[test]
    fn test_event_error_display_event_not_found() {
        let error = EventError::EventNotFound("event-456".to_string());
        assert_eq!(error.to_string(), "Event not found: event-456");
    }

    #[test]
    fn test_event_error_display_storage() {
        let error = EventError::StorageError("Database connection failed".to_string());
        assert_eq!(error.to_string(), "Storage error: Database connection failed");
    }

    #[test]
    fn test_event_error_display_sync() {
        let error = EventError::SyncError("Network timeout".to_string());
        assert_eq!(error.to_string(), "Sync error: Network timeout");
    }

    // ==================== EventError Debug Tests ====================

    #[test]
    fn test_event_error_debug() {
        let error = EventError::ValidationError("test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("ValidationError"));
    }

    // ==================== Edge Case Tests ====================

    #[test]
    fn test_validate_event_unicode_title() {
        let event = create_test_event("test-unicode", "会议 📅 Meeting");
        let result = EventHandler::validate_event(&event);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_event_very_long_title() {
        let long_title = "A".repeat(1000);
        let event = create_test_event("test-long", &long_title);
        let result = EventHandler::validate_event(&event);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_event_special_chars_in_uid() {
        let event = create_test_event("test-uid-with-special@chars.com", "Special UID Event");
        let result = EventHandler::validate_event(&event);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_event_multiline_title() {
        let event = create_test_event("test-multiline", "Line 1\nLine 2");
        let result = EventHandler::validate_event(&event);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_event_tab_only_title() {
        let event = create_test_event("test-tabs", "\t\t");
        let result = EventHandler::validate_event(&event);
        assert!(matches!(result, Err(EventError::ValidationError(_))));
    }
}
