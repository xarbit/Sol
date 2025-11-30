//! Event dialog state structures
//!
//! Separates business data from UI state for cleaner architecture.
//!
//! NOTE: These new structures are prepared for migrating EventDialogState
//! from app.rs to the centralized ActiveDialog pattern. The migration is
//! blocked by text_editor::Content not implementing Clone.

#![allow(dead_code)] // Prepared for dialog architecture migration

use chrono::{NaiveDate, NaiveTime};
use cosmic::widget::{calendar::CalendarModel, text_editor};

use crate::caldav::{AlertTime, RepeatFrequency, TravelTime};

/// Enum for which field is being edited in the event dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventDialogField {
    Title,
    Location,
    Url,
}

/// Business data for an event being created/edited
///
/// This contains the actual event data that will be saved,
/// separate from UI-specific state like picker visibility.
#[derive(Debug, Clone)]
pub struct EventDialogData {
    /// Event UID (None for new events, Some for editing)
    pub editing_uid: Option<String>,
    /// Event title/summary
    pub title: String,
    /// Event location
    pub location: String,
    /// Whether this is an all-day event
    pub all_day: bool,
    /// Start date
    pub start_date: NaiveDate,
    /// Start time (None for all-day events)
    pub start_time: Option<NaiveTime>,
    /// End date
    pub end_date: NaiveDate,
    /// End time (None for all-day events)
    pub end_time: Option<NaiveTime>,
    /// Travel time before the event
    pub travel_time: TravelTime,
    /// Repeat/recurrence settings
    pub repeat: RepeatFrequency,
    /// Selected calendar ID for the event
    pub calendar_id: String,
    /// Invitees (email addresses)
    pub invitees: Vec<String>,
    /// Alert/reminder settings
    pub alert: AlertTime,
    /// Second alert (optional)
    pub alert_second: Option<AlertTime>,
    /// File attachments (paths or URLs)
    pub attachments: Vec<String>,
    /// URL associated with the event
    pub url: String,
    /// Notes/description
    pub notes: String,
}

impl EventDialogData {
    /// Create new event dialog data for a new event
    pub fn new(date: NaiveDate, calendar_id: String) -> Self {
        let default_start_time = NaiveTime::from_hms_opt(9, 0, 0);
        let default_end_time = NaiveTime::from_hms_opt(10, 0, 0);

        Self {
            editing_uid: None,
            title: String::new(),
            location: String::new(),
            all_day: false,
            start_date: date,
            start_time: default_start_time,
            end_date: date,
            end_time: default_end_time,
            travel_time: TravelTime::None,
            repeat: RepeatFrequency::Never,
            calendar_id,
            invitees: vec![],
            alert: AlertTime::None,
            alert_second: None,
            attachments: vec![],
            url: String::new(),
            notes: String::new(),
        }
    }

    /// Check if this is an edit of an existing event
    pub fn is_edit_mode(&self) -> bool {
        self.editing_uid.is_some()
    }
}

/// UI-specific state for the event dialog
///
/// This contains transient UI state that doesn't need to be persisted
/// or passed to business logic.
pub struct EventDialogUiState {
    /// Input buffer for start date text
    pub start_date_input: String,
    /// Input buffer for start time text
    pub start_time_input: String,
    /// Input buffer for end date text
    pub end_date_input: String,
    /// Input buffer for end time text
    pub end_time_input: String,
    /// Input buffer for new invitee
    pub invitee_input: String,
    /// Which field is currently being edited
    pub editing_field: Option<EventDialogField>,
    /// Whether the start date calendar picker is open
    pub start_date_picker_open: bool,
    /// Calendar model for start date picker
    pub start_date_calendar: CalendarModel,
    /// Whether the end date calendar picker is open
    pub end_date_picker_open: bool,
    /// Calendar model for end date picker
    pub end_date_calendar: CalendarModel,
    /// Whether the start time picker is open
    pub start_time_picker_open: bool,
    /// Whether the end time picker is open
    pub end_time_picker_open: bool,
    /// Notes editor content (widget state)
    pub notes_content: text_editor::Content,
}

impl EventDialogUiState {
    /// Create UI state from event data
    pub fn from_data(data: &EventDialogData) -> Self {
        Self {
            start_date_input: data.start_date.format("%Y-%m-%d").to_string(),
            start_time_input: data
                .start_time
                .map(|t| t.format("%H:%M").to_string())
                .unwrap_or_else(|| "09:00".to_string()),
            end_date_input: data.end_date.format("%Y-%m-%d").to_string(),
            end_time_input: data
                .end_time
                .map(|t| t.format("%H:%M").to_string())
                .unwrap_or_else(|| "10:00".to_string()),
            invitee_input: String::new(),
            editing_field: None,
            start_date_picker_open: false,
            start_date_calendar: CalendarModel::new(data.start_date, data.start_date),
            end_date_picker_open: false,
            end_date_calendar: CalendarModel::new(data.end_date, data.end_date),
            start_time_picker_open: false,
            end_time_picker_open: false,
            notes_content: text_editor::Content::with_text(&data.notes),
        }
    }

    /// Close all pickers
    pub fn close_all_pickers(&mut self) {
        self.start_date_picker_open = false;
        self.end_date_picker_open = false;
        self.start_time_picker_open = false;
        self.end_time_picker_open = false;
    }
}
