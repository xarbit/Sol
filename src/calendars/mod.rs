mod calendar_source;
mod caldav_calendar;
mod config;
mod local_calendar;

pub use calendar_source::{CalendarSource, CalendarInfo, CalendarType};
pub use caldav_calendar::CalDavCalendar;
pub use config::{CalendarConfig, CalendarManagerConfig};
pub use local_calendar::LocalCalendar;

use crate::caldav::CalendarEvent;
use chrono::Datelike;
use std::error::Error;

/// Manager for all calendar sources
#[derive(Debug)]
pub struct CalendarManager {
    sources: Vec<Box<dyn CalendarSource>>,
}

impl CalendarManager {
    pub fn new() -> Self {
        CalendarManager {
            sources: Vec::new(),
        }
    }

    /// Create a new CalendarManager with default calendars
    pub fn with_defaults() -> Self {
        let mut manager = Self::new();

        // Add default local calendars
        manager.add_source(Box::new(LocalCalendar::with_color(
            "personal".to_string(),
            "Personal".to_string(),
            "#3B82F6".to_string(),
        )));

        manager.add_source(Box::new(LocalCalendar::with_color(
            "work".to_string(),
            "Work".to_string(),
            "#8B5CF6".to_string(),
        )));

        manager
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

    /// Sync all calendar sources
    pub fn sync_all(&mut self) -> Result<(), Box<dyn Error>> {
        for source in &mut self.sources {
            if source.is_enabled() {
                source.sync()?;
            }
        }
        Ok(())
    }

    /// Save calendar configuration to disk
    /// Each calendar's current state (color, enabled, name) is saved
    pub fn save_config(&self) -> Result<(), Box<dyn Error>> {
        let mut config = CalendarManagerConfig::default();

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
        Self::new()
    }
}
