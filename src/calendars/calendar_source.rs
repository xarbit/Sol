use crate::caldav::CalendarEvent;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::Debug;

/// Type of calendar source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CalendarType {
    Local,
    CalDav,
    Google,
    Outlook,
    ICloud,
    Other,
}

impl CalendarType {
    pub fn as_str(&self) -> &str {
        match self {
            CalendarType::Local => "Local",
            CalendarType::CalDav => "CalDAV",
            CalendarType::Google => "Google Calendar",
            CalendarType::Outlook => "Outlook",
            CalendarType::ICloud => "iCloud",
            CalendarType::Other => "Other",
        }
    }
}

/// Metadata about a calendar source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarInfo {
    /// Unique identifier for this calendar
    pub id: String,
    /// Display name of the calendar
    pub name: String,
    /// Type of calendar source
    pub calendar_type: CalendarType,
    /// Color for displaying events (hex format: "#RRGGBB")
    pub color: String,
    /// Description of the calendar
    pub description: Option<String>,
    /// Whether the calendar is currently enabled/visible
    pub enabled: bool,
}

impl CalendarInfo {
    pub fn new(id: String, name: String, calendar_type: CalendarType) -> Self {
        CalendarInfo {
            id,
            name,
            calendar_type,
            color: Self::default_color_for_type(calendar_type),
            description: None,
            enabled: true,
        }
    }

    fn default_color_for_type(calendar_type: CalendarType) -> String {
        match calendar_type {
            CalendarType::Local => "#3B82F6".to_string(),      // blue
            CalendarType::CalDav => "#8B5CF6".to_string(),     // purple
            CalendarType::Google => "#EA4335".to_string(),     // google red
            CalendarType::Outlook => "#0078D4".to_string(),    // outlook blue
            CalendarType::ICloud => "#007AFF".to_string(),     //
            CalendarType::Other => "#6B7280".to_string(),      // gray
        }
    }
}

/// Trait that all calendar sources must implement
pub trait CalendarSource: Debug + Send {
    /// Get metadata about this calendar
    fn info(&self) -> &CalendarInfo;

    /// Get mutable reference to calendar info
    fn info_mut(&mut self) -> &mut CalendarInfo;

    /// Check if this calendar is enabled
    fn is_enabled(&self) -> bool {
        self.info().enabled
    }

    /// Enable or disable this calendar
    fn set_enabled(&mut self, enabled: bool) {
        self.info_mut().enabled = enabled;
    }

    /// Fetch all events from this calendar source
    fn fetch_events(&self) -> Result<Vec<CalendarEvent>, Box<dyn Error>>;

    /// Add a new event to this calendar
    fn add_event(&mut self, event: CalendarEvent) -> Result<(), Box<dyn Error>>;

    /// Update an existing event
    fn update_event(&mut self, event: CalendarEvent) -> Result<(), Box<dyn Error>>;

    /// Delete an event by UID
    fn delete_event(&mut self, uid: &str) -> Result<(), Box<dyn Error>>;

    /// Sync with the remote source (for remote calendars)
    /// For local calendars, this might just save to disk
    fn sync(&mut self) -> Result<(), Box<dyn Error>>;

    /// Check if this calendar supports read operations
    fn supports_read(&self) -> bool {
        true
    }

    /// Check if this calendar supports write operations
    fn supports_write(&self) -> bool {
        true
    }
}
