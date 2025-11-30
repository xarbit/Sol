//! Calendar dialog state structures
//!
//! Separates business data from UI concerns.
//!
//! NOTE: These structures are prepared for migrating CalendarDialogState
//! to the centralized ActiveDialog pattern.

#![allow(dead_code)] // Prepared for dialog architecture migration

/// Mode for the calendar dialog (Create or Edit)
#[derive(Debug, Clone, PartialEq)]
pub enum CalendarDialogMode {
    /// Creating a new calendar
    Create,
    /// Editing an existing calendar
    Edit {
        /// ID of the calendar being edited
        calendar_id: String,
    },
}

/// Data for the calendar dialog
#[derive(Debug, Clone)]
pub struct CalendarDialogData {
    /// Dialog mode - Create or Edit
    pub mode: CalendarDialogMode,
    /// Calendar name
    pub name: String,
    /// Selected color (hex)
    pub color: String,
}

impl CalendarDialogData {
    /// Create a new calendar dialog in create mode
    pub fn new_create(default_color: &str) -> Self {
        Self {
            mode: CalendarDialogMode::Create,
            name: String::new(),
            color: default_color.to_string(),
        }
    }

    /// Create a new calendar dialog in edit mode
    pub fn new_edit(calendar_id: String, name: String, color: String) -> Self {
        Self {
            mode: CalendarDialogMode::Edit { calendar_id },
            name,
            color,
        }
    }

    /// Check if this is an edit dialog
    pub fn is_edit_mode(&self) -> bool {
        matches!(self.mode, CalendarDialogMode::Edit { .. })
    }
}
