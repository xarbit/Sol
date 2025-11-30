//! CalDAV calendar implementation for remote calendar synchronization.
//!
//! NOTE: Prepared for future CalDAV integration - not yet connected to UI.

#![allow(dead_code)] // Prepared for future CalDAV sync feature

use super::calendar_source::{CalendarInfo, CalendarSource, CalendarType};
use crate::caldav::{CalDavClient, CalendarEvent};
use std::error::Error;

/// A CalDAV-based calendar (supports WebDAV, iCloud, Google, Nextcloud, etc.)
#[derive(Debug)]
pub struct CalDavCalendar {
    info: CalendarInfo,
    client: CalDavClient,
    cached_events: Vec<CalendarEvent>,
}

impl CalDavCalendar {
    /// Create a new CalDAV calendar
    pub fn new(
        id: String,
        name: String,
        server_url: String,
        username: String,
        password: String,
    ) -> Self {
        let info = CalendarInfo::new(id, name, CalendarType::CalDav);
        let client = CalDavClient::new(server_url, username, password);

        CalDavCalendar {
            info,
            client,
            cached_events: Vec::new(),
        }
    }

    /// Create a CalDAV calendar with custom type (e.g., Google, iCloud)
    pub fn with_type(
        id: String,
        name: String,
        calendar_type: CalendarType,
        server_url: String,
        username: String,
        password: String,
    ) -> Self {
        let mut info = CalendarInfo::new(id, name, calendar_type);
        // Use the custom type's default color
        info.color = match calendar_type {
            CalendarType::Google => "#EA4335".to_string(),
            CalendarType::ICloud => "#007AFF".to_string(),
            CalendarType::Outlook => "#0078D4".to_string(),
            _ => "#8B5CF6".to_string(),
        };

        let client = CalDavClient::new(server_url, username, password);

        CalDavCalendar {
            info,
            client,
            cached_events: Vec::new(),
        }
    }

    /// Create a Google Calendar instance (uses CalDAV protocol)
    pub fn google(
        id: String,
        name: String,
        calendar_id: String,
        username: String,
        password: String,
    ) -> Self {
        let server_url = format!(
            "https://apidata.googleusercontent.com/caldav/v2/{}/events",
            calendar_id
        );
        Self::with_type(id, name, CalendarType::Google, server_url, username, password)
    }

    /// Create an iCloud Calendar instance
    pub fn icloud(
        id: String,
        name: String,
        username: String,
        password: String,
    ) -> Self {
        let server_url = format!(
            "https://caldav.icloud.com/{}/calendars",
            username
        );
        Self::with_type(id, name, CalendarType::ICloud, server_url, username, password)
    }

    /// Create a Nextcloud Calendar instance
    pub fn nextcloud(
        id: String,
        name: String,
        server_url: String,
        username: String,
        password: String,
        calendar_name: String,
    ) -> Self {
        let full_url = format!(
            "{}/remote.php/dav/calendars/{}/{}",
            server_url.trim_end_matches('/'),
            username,
            calendar_name
        );
        Self::new(id, name, full_url, username, password)
    }

    /// Get cached events without fetching from server
    pub fn cached_events(&self) -> &[CalendarEvent] {
        &self.cached_events
    }
}

impl CalendarSource for CalDavCalendar {
    fn info(&self) -> &CalendarInfo {
        &self.info
    }

    fn info_mut(&mut self) -> &mut CalendarInfo {
        &mut self.info
    }

    fn fetch_events(&self) -> Result<Vec<CalendarEvent>, Box<dyn Error>> {
        // Return cached events to avoid network calls on every render
        // Use sync() to refresh from server
        Ok(self.cached_events.clone())
    }

    fn add_event(&mut self, event: CalendarEvent) -> Result<(), Box<dyn Error>> {
        self.client.create_event(&event)?;
        self.cached_events.push(event);
        Ok(())
    }

    fn update_event(&mut self, event: CalendarEvent) -> Result<(), Box<dyn Error>> {
        self.client.update_event(&event)?;

        // Update in cache
        if let Some(existing) = self.cached_events.iter_mut().find(|e| e.uid == event.uid) {
            *existing = event;
        }

        Ok(())
    }

    fn delete_event(&mut self, uid: &str) -> Result<(), Box<dyn Error>> {
        self.client.delete_event(uid)?;
        self.cached_events.retain(|e| e.uid != uid);
        Ok(())
    }

    fn sync(&mut self) -> Result<(), Box<dyn Error>> {
        // Fetch fresh events from CalDAV server
        let events = self.client.fetch_events()?;
        self.cached_events = events;
        Ok(())
    }

    fn supports_read(&self) -> bool {
        true
    }

    fn supports_write(&self) -> bool {
        true
    }
}
