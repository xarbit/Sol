//! Local protocol implementation using SQLite database.
//!
//! This protocol stores events in a local SQLite database.
//! It's the default protocol for local calendars.

use std::sync::{Arc, Mutex};

use crate::caldav::CalendarEvent;
use crate::database::Database;
use super::{Protocol, ProtocolResult};

/// Local protocol using SQLite database for event storage.
#[allow(dead_code)] // Foundation for future protocol-based architecture
#[derive(Debug)]
pub struct LocalProtocol {
    /// Shared database connection
    db: Arc<Mutex<Database>>,
}

impl LocalProtocol {
    /// Create a new LocalProtocol with a shared database connection
    #[allow(dead_code)] // Part of protocol API
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        LocalProtocol { db }
    }
}

impl Protocol for LocalProtocol {
    fn fetch_events(&self, calendar_id: &str) -> ProtocolResult<Vec<CalendarEvent>> {
        let db = self.db.lock().map_err(|e| format!("Database lock error: {}", e))?;
        db.get_events_for_calendar(calendar_id)
    }

    fn add_event(&mut self, calendar_id: &str, event: &CalendarEvent) -> ProtocolResult<()> {
        let db = self.db.lock().map_err(|e| format!("Database lock error: {}", e))?;
        db.insert_event(calendar_id, event)
    }

    fn update_event(&mut self, _calendar_id: &str, event: &CalendarEvent) -> ProtocolResult<()> {
        let db = self.db.lock().map_err(|e| format!("Database lock error: {}", e))?;
        db.update_event(event)
    }

    fn delete_event(&mut self, _calendar_id: &str, uid: &str) -> ProtocolResult<bool> {
        let db = self.db.lock().map_err(|e| format!("Database lock error: {}", e))?;
        db.delete_event(uid)
    }

    fn sync(&mut self, _calendar_id: &str) -> ProtocolResult<()> {
        // Local protocol doesn't need to sync - data is always fresh from DB
        Ok(())
    }

    fn requires_network(&self) -> bool {
        false
    }

    fn protocol_type(&self) -> &'static str {
        "local"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::caldav::{AlertTime, RepeatFrequency, TravelTime};
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_local_protocol() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("sol_protocol_test.db");
        let _ = std::fs::remove_file(&db_path);

        let db = Database::open_at(db_path.clone()).unwrap();
        let db = Arc::new(Mutex::new(db));
        let mut protocol = LocalProtocol::new(db);

        // Create an event
        let event = CalendarEvent {
            uid: "protocol-test-1".to_string(),
            summary: "Protocol Test".to_string(),
            location: None,
            all_day: false,
            start: Utc.with_ymd_and_hms(2025, 11, 30, 10, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2025, 11, 30, 11, 0, 0).unwrap(),
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

        // Add event
        protocol.add_event("test-cal", &event).unwrap();

        // Fetch events
        let events = protocol.fetch_events("test-cal").unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].summary, "Protocol Test");

        // Delete event
        let deleted = protocol.delete_event("test-cal", "protocol-test-1").unwrap();
        assert!(deleted);

        // Verify deletion
        let events = protocol.fetch_events("test-cal").unwrap();
        assert_eq!(events.len(), 0);

        // Clean up
        let _ = std::fs::remove_file(&db_path);
    }
}
