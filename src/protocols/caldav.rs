//! CalDAV protocol implementation for remote calendar servers.
//!
//! This protocol handles communication with CalDAV servers like:
//! - Nextcloud
//! - Google Calendar (via CalDAV)
//! - Apple iCloud
//! - Any RFC 4791 compliant server

use crate::caldav::{CalDavClient, CalendarEvent};
use super::{Protocol, ProtocolResult};

/// CalDAV protocol for remote calendar servers.
#[allow(dead_code)] // Foundation for future CalDAV support
#[derive(Debug)]
pub struct CalDavProtocol {
    /// CalDAV HTTP client
    client: CalDavClient,
    /// Cached events (to avoid repeated network calls)
    cached_events: Vec<CalendarEvent>,
}

impl CalDavProtocol {
    /// Create a new CalDavProtocol with server credentials
    #[allow(dead_code)] // Part of protocol API
    pub fn new(server_url: String, username: String, password: String) -> ProtocolResult<Self> {
        let client = CalDavClient::new(server_url, username, password)?;
        Ok(CalDavProtocol {
            client,
            cached_events: Vec::new(),
        })
    }

    /// Create a CalDavProtocol for Google Calendar
    #[allow(dead_code)] // Part of protocol API
    pub fn google(calendar_id: &str, username: &str, password: &str) -> ProtocolResult<Self> {
        let server_url = format!(
            "https://apidata.googleusercontent.com/caldav/v2/{}/events",
            calendar_id
        );
        Self::new(server_url, username.to_string(), password.to_string())
    }

    /// Create a CalDavProtocol for Apple iCloud
    #[allow(dead_code)] // Part of protocol API
    pub fn icloud(username: &str, password: &str) -> ProtocolResult<Self> {
        let server_url = format!(
            "https://caldav.icloud.com/{}/calendars/",
            username
        );
        Self::new(server_url, username.to_string(), password.to_string())
    }

    /// Create a CalDavProtocol for Nextcloud
    #[allow(dead_code)] // Part of protocol API
    pub fn nextcloud(server_url: &str, username: &str, password: &str, calendar_name: &str) -> ProtocolResult<Self> {
        // Security: Validate HTTPS before constructing full URL
        if !server_url.starts_with("https://") {
            return Err(format!(
                "Nextcloud server URL must use HTTPS for secure transmission. Got: {}",
                server_url
            ).into());
        }

        let url = format!(
            "{}/remote.php/dav/calendars/{}/{}",
            server_url.trim_end_matches('/'),
            username,
            calendar_name
        );
        Self::new(url, username.to_string(), password.to_string())
    }
}

impl Protocol for CalDavProtocol {
    fn fetch_events(&self, _calendar_id: &str) -> ProtocolResult<Vec<CalendarEvent>> {
        // Return cached events - use sync() to refresh from server
        Ok(self.cached_events.clone())
    }

    fn add_event(&mut self, _calendar_id: &str, event: &CalendarEvent) -> ProtocolResult<()> {
        // Send to remote server
        self.client.create_event(event)?;
        // Update local cache
        self.cached_events.push(event.clone());
        Ok(())
    }

    fn update_event(&mut self, _calendar_id: &str, event: &CalendarEvent) -> ProtocolResult<()> {
        // Update on remote server
        self.client.update_event(event)?;
        // Update local cache
        if let Some(pos) = self.cached_events.iter().position(|e| e.uid == event.uid) {
            self.cached_events[pos] = event.clone();
        }
        Ok(())
    }

    fn delete_event(&mut self, _calendar_id: &str, uid: &str) -> ProtocolResult<bool> {
        // Delete from remote server
        self.client.delete_event(uid)?;
        // Remove from local cache
        if let Some(pos) = self.cached_events.iter().position(|e| e.uid == uid) {
            self.cached_events.remove(pos);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn sync(&mut self, _calendar_id: &str) -> ProtocolResult<()> {
        // Fetch fresh data from server
        self.cached_events = self.client.fetch_events()?;
        Ok(())
    }

    fn requires_network(&self) -> bool {
        true
    }

    fn protocol_type(&self) -> &'static str {
        "caldav"
    }
}
