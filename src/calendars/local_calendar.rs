use super::calendar_source::{CalendarInfo, CalendarSource, CalendarType};
use super::config::CalendarManagerConfig;
use crate::caldav::CalendarEvent;
use crate::database::Database;
use std::error::Error;
use std::sync::{Arc, Mutex};

/// A local calendar stored in SQLite database (events) with JSON config (metadata)
#[derive(Debug)]
pub struct LocalCalendar {
    info: CalendarInfo,
    /// Shared database connection for events
    db: Arc<Mutex<Database>>,
    /// Cached events for this calendar (refreshed on fetch)
    cached_events: Vec<CalendarEvent>,
}

impl LocalCalendar {
    /// Create a new local calendar with a shared database connection
    pub fn new(id: String, name: String, db: Arc<Mutex<Database>>) -> Self {
        let mut info = CalendarInfo::new(id.clone(), name, CalendarType::Local);

        // Load calendar metadata from config file if it exists
        if let Ok(config) = CalendarManagerConfig::load() {
            if let Some(saved) = config.get_calendar(&id) {
                info.name = saved.name.clone();
                info.color = saved.color.clone();
                info.enabled = saved.enabled;
            }
        }

        let mut calendar = LocalCalendar {
            info,
            db,
            cached_events: Vec::new(),
        };

        // Load events from database
        calendar.load_events_from_db();

        calendar
    }

    /// Create a local calendar with a custom color
    pub fn with_color(id: String, name: String, color: String, db: Arc<Mutex<Database>>) -> Self {
        let mut calendar = Self::new(id.clone(), name, db);

        // Check if calendar config already exists
        let config = CalendarManagerConfig::load().unwrap_or_default();
        let exists = config.get_calendar(&id).is_some();

        if !exists {
            // Set default color for new calendar
            calendar.info.color = color;
        }

        calendar
    }

    /// Load events from database into cache
    fn load_events_from_db(&mut self) {
        if let Ok(db) = self.db.lock() {
            if let Ok(events) = db.get_events_for_calendar(&self.info.id) {
                self.cached_events = events;
            }
        }
    }

    /// Get all events from this calendar (uses cache)
    #[allow(dead_code)] // Reserved for future event access
    pub fn get_events(&self) -> &[CalendarEvent] {
        &self.cached_events
    }

    /// Get events for a specific date
    #[allow(dead_code)] // Reserved for future day filtering
    pub fn get_events_for_date(&self, date: chrono::NaiveDate) -> Vec<&CalendarEvent> {
        self.cached_events
            .iter()
            .filter(|e| e.start.date_naive() == date)
            .collect()
    }

    /// Get events for a specific month
    #[allow(dead_code)] // Reserved for future month filtering
    pub fn get_events_for_month(&self, year: i32, month: u32) -> Vec<&CalendarEvent> {
        use chrono::Datelike;
        self.cached_events
            .iter()
            .filter(|e| {
                let date = e.start.date_naive();
                date.year() == year && date.month() == month
            })
            .collect()
    }
}

impl CalendarSource for LocalCalendar {
    fn info(&self) -> &CalendarInfo {
        &self.info
    }

    fn info_mut(&mut self) -> &mut CalendarInfo {
        &mut self.info
    }

    fn fetch_events(&self) -> Result<Vec<CalendarEvent>, Box<dyn Error>> {
        Ok(self.cached_events.clone())
    }

    fn add_event(&mut self, event: CalendarEvent) -> Result<(), Box<dyn Error>> {
        if let Ok(db) = self.db.lock() {
            db.insert_event(&self.info.id, &event)?;
        }
        // Update cache
        self.cached_events.push(event);
        Ok(())
    }

    fn update_event(&mut self, event: CalendarEvent) -> Result<(), Box<dyn Error>> {
        if let Ok(db) = self.db.lock() {
            db.update_event(&self.info.id, &event)?;
        }
        // Update cache
        if let Some(existing) = self.cached_events.iter_mut().find(|e| e.uid == event.uid) {
            *existing = event;
        }
        Ok(())
    }

    fn delete_event(&mut self, uid: &str) -> Result<(), Box<dyn Error>> {
        if let Ok(db) = self.db.lock() {
            db.delete_event(&self.info.id, uid)?;
        }
        // Update cache
        self.cached_events.retain(|e| e.uid != uid);
        Ok(())
    }

    fn sync(&mut self) -> Result<(), Box<dyn Error>> {
        // Refresh event cache from database
        self.load_events_from_db();
        Ok(())
    }

    fn supports_read(&self) -> bool {
        true
    }

    fn supports_write(&self) -> bool {
        true
    }
}
