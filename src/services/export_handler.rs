//! Export Handler - Import/Export operations for calendar data.
//!
//! This handler manages importing and exporting calendar data in various formats,
//! primarily iCalendar (.ics) format.

use crate::caldav::CalendarEvent;
use crate::calendars::CalendarManager;
use icalendar::{Calendar, Component, Event, EventLike};
use log::{debug, error, info};
use std::error::Error;
use std::fs;
use std::path::Path;

/// Result type for export operations
#[allow(dead_code)] // Part of export API for future use
pub type ExportResult<T> = Result<T, ExportError>;

/// Error types for export operations
#[allow(dead_code)] // Part of export API for future use
#[derive(Debug)]
pub enum ExportError {
    /// File I/O error
    IoError(String),
    /// Invalid file format
    FormatError(String),
    /// Parse error
    ParseError(String),
    /// Calendar not found
    CalendarNotFound(String),
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportError::IoError(msg) => write!(f, "I/O error: {}", msg),
            ExportError::FormatError(msg) => write!(f, "Format error: {}", msg),
            ExportError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ExportError::CalendarNotFound(id) => write!(f, "Calendar not found: {}", id),
        }
    }
}

impl Error for ExportError {}

/// Export Handler - import/export operations.
#[allow(dead_code)] // Foundation for future import/export feature
pub struct ExportHandler;

impl ExportHandler {
    /// Export a single event to iCalendar format
    #[allow(dead_code)] // Part of export API
    pub fn event_to_ical(event: &CalendarEvent) -> Calendar {
        debug!("ExportHandler: Converting event '{}' (uid={}) to iCal", event.summary, event.uid);

        let mut calendar = Calendar::new();

        let mut ical_event = Event::new();
        ical_event.summary(&event.summary);
        ical_event.uid(&event.uid);
        ical_event.starts(event.start);
        ical_event.ends(event.end);

        if let Some(ref location) = event.location {
            ical_event.location(location);
        }

        if let Some(ref notes) = event.notes {
            ical_event.description(notes);
        }

        if let Some(ref url) = event.url {
            ical_event.url(url);
        }

        calendar.push(ical_event);
        debug!("ExportHandler: Event conversion complete");
        calendar
    }

    /// Export all events from a calendar to iCalendar format
    #[allow(dead_code)] // Part of export API
    pub fn calendar_to_ical(
        manager: &CalendarManager,
        calendar_id: &str,
    ) -> ExportResult<Calendar> {
        info!("ExportHandler: Exporting calendar '{}' to iCal format", calendar_id);

        let calendar = manager
            .sources()
            .iter()
            .find(|c| c.info().id == calendar_id)
            .ok_or_else(|| {
                error!("ExportHandler: Calendar '{}' not found", calendar_id);
                ExportError::CalendarNotFound(calendar_id.to_string())
            })?;

        let events = calendar
            .fetch_events()
            .map_err(|e| {
                error!("ExportHandler: Failed to fetch events: {}", e);
                ExportError::IoError(e.to_string())
            })?;

        debug!("ExportHandler: Found {} events to export", events.len());

        let mut ical = Calendar::new();

        for event in events {
            let mut ical_event = Event::new();
            ical_event.summary(&event.summary);
            ical_event.uid(&event.uid);
            ical_event.starts(event.start);
            ical_event.ends(event.end);

            if let Some(ref location) = event.location {
                ical_event.location(location);
            }

            if let Some(ref notes) = event.notes {
                ical_event.description(notes);
            }

            if let Some(ref url) = event.url {
                ical_event.url(url);
            }

            ical.push(ical_event);
        }

        info!("ExportHandler: Successfully exported calendar '{}'", calendar_id);
        Ok(ical)
    }

    /// Export a calendar to an iCalendar file
    #[allow(dead_code)] // Part of export API
    pub fn export_to_file<P: AsRef<Path>>(
        manager: &CalendarManager,
        calendar_id: &str,
        path: P,
    ) -> ExportResult<()> {
        info!("ExportHandler: Exporting calendar '{}' to file {:?}", calendar_id, path.as_ref());

        let ical = Self::calendar_to_ical(manager, calendar_id)?;
        let ical_string = ical.to_string();

        fs::write(&path, ical_string).map_err(|e| {
            error!("ExportHandler: Failed to write file: {}", e);
            ExportError::IoError(e.to_string())
        })?;

        info!("ExportHandler: Successfully exported to {:?}", path.as_ref());
        Ok(())
    }

    /// Export all calendars to a single iCalendar file
    #[allow(dead_code)] // Part of export API
    pub fn export_all_to_file<P: AsRef<Path>>(
        manager: &CalendarManager,
        path: P,
    ) -> ExportResult<()> {
        info!("ExportHandler: Exporting all calendars to file {:?}", path.as_ref());

        let mut combined = Calendar::new();
        let mut total_events = 0;

        for calendar in manager.sources() {
            if !calendar.is_enabled() {
                debug!("ExportHandler: Skipping disabled calendar '{}'", calendar.info().name);
                continue;
            }

            if let Ok(events) = calendar.fetch_events() {
                debug!("ExportHandler: Adding {} events from '{}'", events.len(), calendar.info().name);
                for event in events {
                    let mut ical_event = Event::new();
                    ical_event.summary(&event.summary);
                    ical_event.uid(&event.uid);
                    ical_event.starts(event.start);
                    ical_event.ends(event.end);

                    if let Some(ref location) = event.location {
                        ical_event.location(location);
                    }

                    if let Some(ref notes) = event.notes {
                        ical_event.description(notes);
                    }

                    combined.push(ical_event);
                    total_events += 1;
                }
            }
        }

        let ical_string = combined.to_string();
        fs::write(&path, ical_string).map_err(|e| {
            error!("ExportHandler: Failed to write file: {}", e);
            ExportError::IoError(e.to_string())
        })?;

        info!("ExportHandler: Exported {} events to {:?}", total_events, path.as_ref());
        Ok(())
    }

    /// Read an iCalendar file (placeholder for future import functionality)
    #[allow(dead_code)] // Part of export API
    pub fn read_ical_file<P: AsRef<Path>>(path: P) -> ExportResult<String> {
        info!("ExportHandler: Reading iCal file {:?}", path.as_ref());
        fs::read_to_string(&path).map_err(|e| {
            error!("ExportHandler: Failed to read file: {}", e);
            ExportError::IoError(e.to_string())
        })
    }

    // TODO: Implement import functionality
    // This requires parsing iCalendar format and creating CalendarEvents
    // pub fn import_from_file<P: AsRef<Path>>(
    //     manager: &mut CalendarManager,
    //     calendar_id: &str,
    //     path: P,
    // ) -> ExportResult<usize> { ... }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::caldav::{AlertTime, RepeatFrequency, TravelTime};
    use chrono::{TimeZone, Utc};

    fn create_test_event() -> CalendarEvent {
        CalendarEvent {
            uid: "test-export-1".to_string(),
            summary: "Test Export Event".to_string(),
            location: Some("Test Location".to_string()),
            all_day: false,
            start: Utc.with_ymd_and_hms(2025, 12, 1, 10, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2025, 12, 1, 11, 0, 0).unwrap(),
            travel_time: TravelTime::None,
            repeat: RepeatFrequency::Never,
            repeat_until: None,
            exception_dates: vec![],
            invitees: vec![],
            alert: AlertTime::None,
            alert_second: None,
            attachments: vec![],
            url: None,
            notes: Some("Test notes".to_string()),
        }
    }

    #[test]
    fn test_event_to_ical() {
        let event = create_test_event();
        let ical = ExportHandler::event_to_ical(&event);
        let ical_string = ical.to_string();

        assert!(ical_string.contains("BEGIN:VCALENDAR"));
        assert!(ical_string.contains("BEGIN:VEVENT"));
        assert!(ical_string.contains("Test Export Event"));
        assert!(ical_string.contains("END:VEVENT"));
        assert!(ical_string.contains("END:VCALENDAR"));
    }
}
