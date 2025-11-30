//! Export Handler - Import/Export operations for calendar data.
//!
//! This handler manages importing and exporting calendar data in various formats,
//! primarily iCalendar (.ics) format.

use crate::caldav::CalendarEvent;
use crate::calendars::CalendarManager;
use icalendar::{Calendar, Component, Event, EventLike};
use std::error::Error;
use std::fs;
use std::path::Path;

/// Result type for export operations
pub type ExportResult<T> = Result<T, ExportError>;

/// Error types for export operations
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
pub struct ExportHandler;

impl ExportHandler {
    /// Export a single event to iCalendar format
    pub fn event_to_ical(event: &CalendarEvent) -> Calendar {
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
        calendar
    }

    /// Export all events from a calendar to iCalendar format
    pub fn calendar_to_ical(
        manager: &CalendarManager,
        calendar_id: &str,
    ) -> ExportResult<Calendar> {
        let calendar = manager
            .sources()
            .iter()
            .find(|c| c.info().id == calendar_id)
            .ok_or_else(|| ExportError::CalendarNotFound(calendar_id.to_string()))?;

        let events = calendar
            .fetch_events()
            .map_err(|e| ExportError::IoError(e.to_string()))?;

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

        Ok(ical)
    }

    /// Export a calendar to an iCalendar file
    pub fn export_to_file<P: AsRef<Path>>(
        manager: &CalendarManager,
        calendar_id: &str,
        path: P,
    ) -> ExportResult<()> {
        let ical = Self::calendar_to_ical(manager, calendar_id)?;
        let ical_string = ical.to_string();

        fs::write(path, ical_string).map_err(|e| ExportError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Export all calendars to a single iCalendar file
    pub fn export_all_to_file<P: AsRef<Path>>(
        manager: &CalendarManager,
        path: P,
    ) -> ExportResult<()> {
        let mut combined = Calendar::new();

        for calendar in manager.sources() {
            if !calendar.is_enabled() {
                continue;
            }

            if let Ok(events) = calendar.fetch_events() {
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
                }
            }
        }

        let ical_string = combined.to_string();
        fs::write(path, ical_string).map_err(|e| ExportError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Read an iCalendar file (placeholder for future import functionality)
    pub fn read_ical_file<P: AsRef<Path>>(path: P) -> ExportResult<String> {
        fs::read_to_string(path).map_err(|e| ExportError::IoError(e.to_string()))
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
