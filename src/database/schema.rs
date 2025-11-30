use chrono::{DateTime, Utc};
use log::{debug, info};
use rusqlite::{Connection, params, Result as SqlResult};
use std::error::Error;
use std::path::PathBuf;

use crate::caldav::CalendarEvent;

/// Current database schema version for migrations
const SCHEMA_VERSION: i32 = 2;

/// Database connection wrapper with encryption support
pub struct Database {
    conn: Connection,
}

impl std::fmt::Debug for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Database")
            .field("path", &Database::get_database_path())
            .finish()
    }
}

impl Database {
    /// Open or create the database at the default location
    pub fn open() -> Result<Self, Box<dyn Error>> {
        let path = Self::get_database_path();
        info!("Database: Opening database at {:?}", path);
        Self::open_at(path)
    }

    /// Open or create the database at a specific path
    pub fn open_at(path: PathBuf) -> Result<Self, Box<dyn Error>> {
        debug!("Database: Opening database at {:?}", path);

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&path)?;
        let mut db = Database { conn };

        // Initialize schema
        db.init_schema()?;

        info!("Database: Successfully opened database");
        Ok(db)
    }

    /// Open an encrypted database with a passphrase
    pub fn open_encrypted(passphrase: &str) -> Result<Self, Box<dyn Error>> {
        let path = Self::get_database_path();
        Self::open_encrypted_at(path, passphrase)
    }

    /// Open an encrypted database at a specific path
    pub fn open_encrypted_at(path: PathBuf, passphrase: &str) -> Result<Self, Box<dyn Error>> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&path)?;

        // Set encryption key using SQLCipher pragma
        // The key must be set before any other operations
        conn.pragma_update(None, "key", passphrase)?;

        let mut db = Database { conn };
        db.init_schema()?;

        Ok(db)
    }

    /// Get the default database file path
    pub fn get_database_path() -> PathBuf {
        let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("sol-calendar");
        path.push("sol.db");
        path
    }

    /// Initialize the database schema
    fn init_schema(&mut self) -> Result<(), Box<dyn Error>> {
        // Check current schema version
        let version = self.get_schema_version()?;

        if version == 0 {
            // Fresh database, create tables
            self.create_tables()?;
            self.set_schema_version(SCHEMA_VERSION)?;
        } else if version < SCHEMA_VERSION {
            // Run migrations
            self.migrate(version)?;
        }

        Ok(())
    }

    /// Get the current schema version
    fn get_schema_version(&self) -> Result<i32, Box<dyn Error>> {
        // Try to get version from meta table (stored as text)
        let result: SqlResult<String> = self.conn.query_row(
            "SELECT value FROM meta WHERE key = 'schema_version'",
            [],
            |row| row.get(0),
        );

        match result {
            Ok(version_str) => Ok(version_str.parse().unwrap_or(0)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(0),
            Err(rusqlite::Error::SqliteFailure(_, _)) => Ok(0), // Table doesn't exist
            Err(e) => Err(Box::new(e)),
        }
    }

    /// Set the schema version
    fn set_schema_version(&self, version: i32) -> Result<(), Box<dyn Error>> {
        self.conn.execute(
            "INSERT OR REPLACE INTO meta (key, value) VALUES ('schema_version', ?1)",
            params![version.to_string()],
        )?;
        Ok(())
    }

    /// Create all database tables
    /// Note: Calendar metadata (name, color, enabled) is stored in config file, not database
    fn create_tables(&self) -> Result<(), Box<dyn Error>> {
        self.conn.execute_batch(
            r#"
            -- Metadata table for schema version
            CREATE TABLE IF NOT EXISTS meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            -- Events table (calendar metadata is in config file)
            CREATE TABLE IF NOT EXISTS events (
                uid TEXT PRIMARY KEY,
                calendar_id TEXT NOT NULL,
                summary TEXT NOT NULL,
                location TEXT,
                all_day INTEGER NOT NULL DEFAULT 0,
                start_time TEXT NOT NULL,
                end_time TEXT NOT NULL,
                travel_time TEXT NOT NULL DEFAULT 'None',
                repeat TEXT NOT NULL DEFAULT 'Never',
                invitees TEXT NOT NULL DEFAULT '[]',
                alert TEXT NOT NULL DEFAULT 'None',
                alert_second TEXT,
                attachments TEXT NOT NULL DEFAULT '[]',
                url TEXT,
                notes TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            -- Index for efficient date range queries
            CREATE INDEX IF NOT EXISTS idx_events_start_time ON events(start_time);
            CREATE INDEX IF NOT EXISTS idx_events_calendar_id ON events(calendar_id);
            CREATE INDEX IF NOT EXISTS idx_events_calendar_date ON events(calendar_id, start_time);
            "#,
        )?;

        Ok(())
    }

    /// Run migrations from old version to current
    fn migrate(&mut self, from_version: i32) -> Result<(), Box<dyn Error>> {
        if from_version < 2 {
            // Migrate from v1 to v2: Add new event fields
            // We need to recreate the table since SQLite doesn't support adding
            // columns with non-null defaults easily, and we're changing structure
            self.conn.execute_batch(
                r#"
                -- Create new events table with all fields
                CREATE TABLE IF NOT EXISTS events_new (
                    uid TEXT PRIMARY KEY,
                    calendar_id TEXT NOT NULL,
                    summary TEXT NOT NULL,
                    location TEXT,
                    all_day INTEGER NOT NULL DEFAULT 0,
                    start_time TEXT NOT NULL,
                    end_time TEXT NOT NULL,
                    travel_time TEXT NOT NULL DEFAULT 'None',
                    repeat TEXT NOT NULL DEFAULT 'Never',
                    invitees TEXT NOT NULL DEFAULT '[]',
                    alert TEXT NOT NULL DEFAULT 'None',
                    alert_second TEXT,
                    attachments TEXT NOT NULL DEFAULT '[]',
                    url TEXT,
                    notes TEXT,
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
                );

                -- Copy existing data (description becomes notes)
                INSERT INTO events_new (uid, calendar_id, summary, location, all_day, start_time, end_time, notes, created_at, updated_at)
                SELECT uid, calendar_id, summary, location, all_day, start_time, end_time, description, created_at, updated_at
                FROM events;

                -- Drop old table and rename new one
                DROP TABLE events;
                ALTER TABLE events_new RENAME TO events;

                -- Recreate indexes
                CREATE INDEX IF NOT EXISTS idx_events_start_time ON events(start_time);
                CREATE INDEX IF NOT EXISTS idx_events_calendar_id ON events(calendar_id);
                CREATE INDEX IF NOT EXISTS idx_events_calendar_date ON events(calendar_id, start_time);
                "#,
            )?;
        }

        self.set_schema_version(SCHEMA_VERSION)?;
        Ok(())
    }

    // ==================== Event Operations ====================
    // Note: Calendar metadata (name, color, enabled) is stored in config file

    /// Insert a new event
    pub fn insert_event(&self, calendar_id: &str, event: &CalendarEvent) -> Result<(), Box<dyn Error>> {
        let travel_time = serde_json::to_string(&event.travel_time)?;
        let repeat = serde_json::to_string(&event.repeat)?;
        let invitees = serde_json::to_string(&event.invitees)?;
        let alert = serde_json::to_string(&event.alert)?;
        let alert_second = event.alert_second.as_ref().map(|a| serde_json::to_string(a)).transpose()?;
        let attachments = serde_json::to_string(&event.attachments)?;

        self.conn.execute(
            r#"
            INSERT INTO events (uid, calendar_id, summary, location, all_day, start_time, end_time,
                               travel_time, repeat, invitees, alert, alert_second, attachments, url, notes)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
            "#,
            params![
                event.uid,
                calendar_id,
                event.summary,
                event.location,
                event.all_day,
                event.start.to_rfc3339(),
                event.end.to_rfc3339(),
                travel_time,
                repeat,
                invitees,
                alert,
                alert_second,
                attachments,
                event.url,
                event.notes,
            ],
        )?;
        Ok(())
    }

    /// Update an existing event
    pub fn update_event(&self, event: &CalendarEvent) -> Result<(), Box<dyn Error>> {
        let travel_time = serde_json::to_string(&event.travel_time)?;
        let repeat = serde_json::to_string(&event.repeat)?;
        let invitees = serde_json::to_string(&event.invitees)?;
        let alert = serde_json::to_string(&event.alert)?;
        let alert_second = event.alert_second.as_ref().map(|a| serde_json::to_string(a)).transpose()?;
        let attachments = serde_json::to_string(&event.attachments)?;

        self.conn.execute(
            r#"
            UPDATE events SET
                summary = ?2,
                location = ?3,
                all_day = ?4,
                start_time = ?5,
                end_time = ?6,
                travel_time = ?7,
                repeat = ?8,
                invitees = ?9,
                alert = ?10,
                alert_second = ?11,
                attachments = ?12,
                url = ?13,
                notes = ?14,
                updated_at = datetime('now')
            WHERE uid = ?1
            "#,
            params![
                event.uid,
                event.summary,
                event.location,
                event.all_day,
                event.start.to_rfc3339(),
                event.end.to_rfc3339(),
                travel_time,
                repeat,
                invitees,
                alert,
                alert_second,
                attachments,
                event.url,
                event.notes,
            ],
        )?;
        Ok(())
    }

    /// Delete an event by UID
    pub fn delete_event(&self, uid: &str) -> Result<bool, Box<dyn Error>> {
        let rows = self.conn.execute("DELETE FROM events WHERE uid = ?1", params![uid])?;
        Ok(rows > 0)
    }

    /// Get all events for a calendar
    pub fn get_events_for_calendar(&self, calendar_id: &str) -> Result<Vec<CalendarEvent>, Box<dyn Error>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT uid, summary, location, all_day, start_time, end_time,
                      travel_time, repeat, invitees, alert, alert_second,
                      attachments, url, notes
               FROM events WHERE calendar_id = ?1"#
        )?;

        let events = stmt.query_map(params![calendar_id], |row| {
            let start_str: String = row.get(4)?;
            let end_str: String = row.get(5)?;
            let travel_time_str: String = row.get(6)?;
            let repeat_str: String = row.get(7)?;
            let invitees_str: String = row.get(8)?;
            let alert_str: String = row.get(9)?;
            let alert_second_str: Option<String> = row.get(10)?;
            let attachments_str: String = row.get(11)?;

            Ok(CalendarEvent {
                uid: row.get(0)?,
                summary: row.get(1)?,
                location: row.get(2)?,
                all_day: row.get(3)?,
                start: DateTime::parse_from_rfc3339(&start_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                end: DateTime::parse_from_rfc3339(&end_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                travel_time: serde_json::from_str(&travel_time_str).unwrap_or_default(),
                repeat: serde_json::from_str(&repeat_str).unwrap_or_default(),
                invitees: serde_json::from_str(&invitees_str).unwrap_or_default(),
                alert: serde_json::from_str(&alert_str).unwrap_or_default(),
                alert_second: alert_second_str.and_then(|s| serde_json::from_str(&s).ok()),
                attachments: serde_json::from_str(&attachments_str).unwrap_or_default(),
                url: row.get(12)?,
                notes: row.get(13)?,
            })
        })?
        .collect::<SqlResult<Vec<_>>>()?;

        Ok(events)
    }

    /// Delete all events for a calendar
    pub fn delete_events_for_calendar(&self, calendar_id: &str) -> Result<usize, Box<dyn Error>> {
        let rows = self.conn.execute(
            "DELETE FROM events WHERE calendar_id = ?1",
            params![calendar_id],
        )?;
        Ok(rows)
    }

    /// Delete all events from all calendars
    /// Used for development/testing to start fresh
    pub fn clear_all_events(&self) -> Result<usize, Box<dyn Error>> {
        let rows = self.conn.execute("DELETE FROM events", [])?;
        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::caldav::{AlertTime, RepeatFrequency, TravelTime};
    use chrono::TimeZone;

    #[test]
    fn test_database_creation() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("sol_test.db");

        // Clean up any existing test database
        let _ = std::fs::remove_file(&db_path);

        let db = Database::open_at(db_path.clone()).unwrap();

        // Verify schema version is set
        let version = db.get_schema_version().unwrap();
        assert_eq!(version, SCHEMA_VERSION);

        // Clean up
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn test_event_operations() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("sol_test_events.db");
        let _ = std::fs::remove_file(&db_path);

        let db = Database::open_at(db_path.clone()).unwrap();

        // Insert an event (calendar metadata is in config file, not database)
        let event = CalendarEvent {
            uid: "event1".to_string(),
            summary: "Test Event".to_string(),
            location: Some("Test Location".to_string()),
            all_day: false,
            start: Utc.with_ymd_and_hms(2025, 11, 29, 10, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2025, 11, 29, 11, 0, 0).unwrap(),
            travel_time: TravelTime::None,
            repeat: RepeatFrequency::Never,
            invitees: vec![],
            alert: AlertTime::FifteenMinutes,
            alert_second: None,
            attachments: vec![],
            url: None,
            notes: Some("A test event".to_string()),
        };

        db.insert_event("cal1", &event).unwrap();

        // Retrieve events for calendar
        let events = db.get_events_for_calendar("cal1").unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].summary, "Test Event");

        // Delete event
        let deleted = db.delete_event("event1").unwrap();
        assert!(deleted);

        let events = db.get_events_for_calendar("cal1").unwrap();
        assert_eq!(events.len(), 0);

        // Test delete all events for calendar
        db.insert_event("cal1", &event).unwrap();
        let rows = db.delete_events_for_calendar("cal1").unwrap();
        assert_eq!(rows, 1);

        // Clean up
        let _ = std::fs::remove_file(&db_path);
    }
}
