mod calendar_source;
mod caldav_calendar;
mod config;
mod local_calendar;

pub use calendar_source::{CalendarSource, CalendarType, CalendarInfo};
pub use config::{CalendarConfig, CalendarManagerConfig};
pub use local_calendar::LocalCalendar;

use crate::caldav::CalendarEvent;
use crate::components::DisplayEvent;
use crate::database::Database;
use chrono::{Datelike, Timelike};
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};

/// Manager for all calendar sources
#[derive(Debug)]
pub struct CalendarManager {
    sources: Vec<Box<dyn CalendarSource>>,
    /// Shared database connection
    db: Arc<Mutex<Database>>,
}

impl CalendarManager {
    /// Create a new CalendarManager with a database connection
    pub fn new() -> Self {
        // Open or create the database
        let db = Database::open().expect("Failed to open database");
        let db = Arc::new(Mutex::new(db));

        CalendarManager {
            sources: Vec::new(),
            db,
        }
    }

    /// Create a new CalendarManager, loading calendars from config
    /// If no calendars exist, creates default ones
    pub fn with_defaults() -> Self {
        let mut manager = Self::new();
        let db = manager.db.clone();

        // Try to load calendars from config
        let config = CalendarManagerConfig::load().unwrap_or_default();

        if config.calendars.is_empty() {
            // No saved calendars, create defaults
            manager.add_source(Box::new(LocalCalendar::with_color(
                "personal".to_string(),
                "Personal".to_string(),
                "#3B82F6".to_string(),
                db.clone(),
            )));

            manager.add_source(Box::new(LocalCalendar::with_color(
                "work".to_string(),
                "Work".to_string(),
                "#8B5CF6".to_string(),
                db,
            )));

            // Save the defaults
            manager.save_config().ok();
        } else {
            // Load calendars from config
            for cal_config in &config.calendars {
                let mut calendar = LocalCalendar::new(
                    cal_config.id.clone(),
                    cal_config.name.clone(),
                    db.clone(),
                );
                // Apply saved settings
                calendar.info_mut().color = cal_config.color.clone();
                calendar.info_mut().enabled = cal_config.enabled;
                manager.add_source(Box::new(calendar));
            }
        }

        manager
    }

    /// Add a new local calendar
    pub fn add_local_calendar(&mut self, id: String, name: String, color: String) {
        let calendar = LocalCalendar::with_color(id, name, color, self.db.clone());
        self.add_source(Box::new(calendar));
        self.save_config().ok();
    }

    /// Remove a calendar by ID and delete all its events
    pub fn delete_calendar(&mut self, id: &str) -> bool {
        // First delete all events for this calendar from database
        if let Ok(db) = self.db.lock() {
            let _ = db.delete_events_for_calendar(id);
        }

        // Remove from sources
        if let Some(index) = self.sources.iter().position(|s| s.info().id == id) {
            self.sources.remove(index);

            // Update config file
            if let Ok(mut config) = CalendarManagerConfig::load() {
                config.remove_calendar(id);
                config.save().ok();
            }

            return true;
        }
        false
    }

    /// Get the shared database connection
    pub fn database(&self) -> Arc<Mutex<Database>> {
        self.db.clone()
    }

    /// Add a calendar source to the manager
    pub fn add_source(&mut self, source: Box<dyn CalendarSource>) {
        self.sources.push(source);
    }

    /// Remove a calendar source by ID
    pub fn remove_source(&mut self, id: &str) -> bool {
        if let Some(index) = self.sources.iter().position(|s| s.info().id == id) {
            self.sources.remove(index);
            true
        } else {
            false
        }
    }

    /// Get all calendar sources
    pub fn sources(&self) -> &[Box<dyn CalendarSource>] {
        &self.sources
    }

    /// Get a mutable reference to all sources
    pub fn sources_mut(&mut self) -> &mut [Box<dyn CalendarSource>] {
        &mut self.sources
    }

    /// Get all events from all enabled calendars
    pub fn get_all_events(&self) -> Vec<CalendarEvent> {
        let mut all_events = Vec::new();
        for source in &self.sources {
            if source.is_enabled() {
                if let Ok(events) = source.fetch_events() {
                    all_events.extend(events);
                }
            }
        }
        all_events
    }

    /// Get events for a specific date from all enabled calendars
    pub fn get_events_for_date(&self, date: chrono::NaiveDate) -> Vec<CalendarEvent> {
        self.get_all_events()
            .into_iter()
            .filter(|e| e.start.date_naive() == date)
            .collect()
    }

    /// Get events for a specific month from all enabled calendars
    pub fn get_events_for_month(&self, year: i32, month: u32) -> Vec<CalendarEvent> {
        self.get_all_events()
            .into_iter()
            .filter(|e| {
                let event_date = e.start.date_naive();
                event_date.year() == year && event_date.month() == month
            })
            .collect()
    }

    /// Get events for a specific month grouped by date, with calendar colors.
    /// Includes events from adjacent months that would be visible in the month view.
    /// Returns a HashMap where key is NaiveDate and value is Vec of DisplayEvents.
    pub fn get_display_events_for_month(&self, year: i32, month: u32) -> HashMap<chrono::NaiveDate, Vec<DisplayEvent>> {
        use chrono::NaiveDate;

        let mut events_by_date: HashMap<NaiveDate, Vec<DisplayEvent>> = HashMap::new();

        // Calculate date range for the month view (includes adjacent month days visible in the grid)
        // The grid can show up to 6 days from prev month and up to 13 days from next month
        let first_of_month = NaiveDate::from_ymd_opt(year, month, 1).unwrap();

        // Start from up to 6 days before (max days from prev month in grid)
        let range_start = first_of_month - chrono::Duration::days(6);
        // End up to 13 days after the month ends (max days from next month in grid)
        let days_in_month = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap().signed_duration_since(first_of_month).num_days()
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap().signed_duration_since(first_of_month).num_days()
        };
        let range_end = first_of_month + chrono::Duration::days(days_in_month + 13);

        for source in &self.sources {
            if !source.is_enabled() {
                continue;
            }

            let calendar_color = source.info().color.clone();

            if let Ok(events) = source.fetch_events() {
                for event in events {
                    let event_date = event.start.date_naive();
                    // Include events within the visible date range
                    if event_date >= range_start && event_date <= range_end {
                        // Extract start time for timed events
                        let start_time = if event.all_day {
                            None
                        } else {
                            Some(chrono::NaiveTime::from_hms_opt(
                                event.start.hour(),
                                event.start.minute(),
                                0,
                            ).unwrap_or_default())
                        };

                        let display_event = DisplayEvent {
                            uid: event.uid.clone(),
                            summary: event.summary.clone(),
                            color: calendar_color.clone(),
                            all_day: event.all_day,
                            start_time,
                        };
                        events_by_date
                            .entry(event_date)
                            .or_default()
                            .push(display_event);
                    }
                }
            }
        }

        events_by_date
    }

    /// Sync all calendar sources
    pub fn sync_all(&mut self) -> Result<(), Box<dyn Error>> {
        for source in &mut self.sources {
            if source.is_enabled() {
                source.sync()?;
            }
        }
        Ok(())
    }

    /// Save calendar configuration to config file (not database)
    /// Each calendar's current state (color, enabled, name) is saved
    pub fn save_config(&self) -> Result<(), Box<dyn Error>> {
        let mut config = CalendarManagerConfig::load().unwrap_or_default();

        for source in &self.sources {
            let info = source.info();
            config.update_calendar(CalendarConfig {
                id: info.id.clone(),
                name: info.name.clone(),
                color: info.color.clone(),
                enabled: info.enabled,
                calendar_type: format!("{:?}", info.calendar_type),
            });
        }

        config.save()?;
        Ok(())
    }
}

impl Default for CalendarManager {
    fn default() -> Self {
        Self::with_defaults()
    }
}
