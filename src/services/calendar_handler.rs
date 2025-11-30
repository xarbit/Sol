//! Calendar Handler - Centralized calendar management.
//!
//! This handler manages calendar CRUD operations (not events, but the calendars themselves).
//! It handles creating, editing, deleting calendars, toggling visibility, and color changes.

use crate::calendars::CalendarManager;
use crate::components::color_picker::CALENDAR_COLORS;
use std::error::Error;

/// Result type for calendar operations
pub type CalendarResult<T> = Result<T, CalendarError>;

/// Error types for calendar operations
#[derive(Debug)]
pub enum CalendarError {
    /// Calendar not found
    NotFound(String),
    /// Invalid calendar data
    ValidationError(String),
    /// Failed to save configuration
    ConfigError(String),
    /// Calendar ID already exists
    DuplicateId(String),
}

impl std::fmt::Display for CalendarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CalendarError::NotFound(id) => write!(f, "Calendar not found: {}", id),
            CalendarError::ValidationError(msg) => write!(f, "Invalid calendar: {}", msg),
            CalendarError::ConfigError(msg) => write!(f, "Config error: {}", msg),
            CalendarError::DuplicateId(id) => write!(f, "Calendar ID already exists: {}", id),
        }
    }
}

impl Error for CalendarError {}

/// Data for creating a new calendar
pub struct NewCalendarData {
    pub name: String,
    pub color: String,
}

/// Data for updating a calendar
pub struct UpdateCalendarData {
    pub name: Option<String>,
    pub color: Option<String>,
    pub enabled: Option<bool>,
}

/// Calendar Handler - centralized calendar management.
pub struct CalendarHandler;

impl CalendarHandler {
    /// Get the default color for a new calendar
    pub fn default_color() -> String {
        CALENDAR_COLORS
            .first()
            .map(|(hex, _)| hex.to_string())
            .unwrap_or_else(|| "#3B82F6".to_string())
    }

    /// Generate a unique calendar ID from a name
    pub fn generate_id(name: &str, manager: &CalendarManager) -> String {
        let base_id: String = name
            .trim()
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == ' ')
            .map(|c| if c == ' ' { '-' } else { c })
            .collect();

        // Make sure ID is unique
        let mut unique_id = base_id.clone();
        let mut counter = 1;

        while manager.sources().iter().any(|c| c.info().id == unique_id) {
            unique_id = format!("{}-{}", base_id, counter);
            counter += 1;
        }

        unique_id
    }

    /// Validate calendar data before creating/updating
    pub fn validate(data: &NewCalendarData) -> CalendarResult<()> {
        if data.name.trim().is_empty() {
            return Err(CalendarError::ValidationError(
                "Calendar name is required".to_string(),
            ));
        }

        if data.color.is_empty() {
            return Err(CalendarError::ValidationError(
                "Calendar color is required".to_string(),
            ));
        }

        Ok(())
    }

    /// Create a new calendar
    pub fn create(manager: &mut CalendarManager, data: NewCalendarData) -> CalendarResult<String> {
        // Validate
        Self::validate(&data)?;

        // Generate unique ID
        let id = Self::generate_id(&data.name, manager);

        // Add the calendar
        manager.add_local_calendar(id.clone(), data.name, data.color);

        Ok(id)
    }

    /// Update an existing calendar
    pub fn update(
        manager: &mut CalendarManager,
        calendar_id: &str,
        data: UpdateCalendarData,
    ) -> CalendarResult<()> {
        let calendar = manager
            .sources_mut()
            .iter_mut()
            .find(|c| c.info().id == calendar_id)
            .ok_or_else(|| CalendarError::NotFound(calendar_id.to_string()))?;

        // Apply updates
        if let Some(name) = data.name {
            if name.trim().is_empty() {
                return Err(CalendarError::ValidationError(
                    "Calendar name cannot be empty".to_string(),
                ));
            }
            calendar.info_mut().name = name;
        }

        if let Some(color) = data.color {
            calendar.info_mut().color = color;
        }

        if let Some(enabled) = data.enabled {
            calendar.set_enabled(enabled);
        }

        // Save configuration
        manager
            .save_config()
            .map_err(|e| CalendarError::ConfigError(e.to_string()))?;

        Ok(())
    }

    /// Toggle a calendar's enabled state
    pub fn toggle_enabled(manager: &mut CalendarManager, calendar_id: &str) -> CalendarResult<bool> {
        let calendar = manager
            .sources_mut()
            .iter_mut()
            .find(|c| c.info().id == calendar_id)
            .ok_or_else(|| CalendarError::NotFound(calendar_id.to_string()))?;

        let new_state = !calendar.is_enabled();
        calendar.set_enabled(new_state);

        // Save configuration
        manager
            .save_config()
            .map_err(|e| CalendarError::ConfigError(e.to_string()))?;

        Ok(new_state)
    }

    /// Change a calendar's color
    pub fn change_color(
        manager: &mut CalendarManager,
        calendar_id: &str,
        color: String,
    ) -> CalendarResult<()> {
        Self::update(
            manager,
            calendar_id,
            UpdateCalendarData {
                name: None,
                color: Some(color),
                enabled: None,
            },
        )
    }

    /// Delete a calendar and all its events
    pub fn delete(manager: &mut CalendarManager, calendar_id: &str) -> CalendarResult<()> {
        if !manager.delete_calendar(calendar_id) {
            return Err(CalendarError::NotFound(calendar_id.to_string()));
        }
        Ok(())
    }

    /// Get calendar info by ID
    pub fn get_info(
        manager: &CalendarManager,
        calendar_id: &str,
    ) -> CalendarResult<(String, String, bool)> {
        let calendar = manager
            .sources()
            .iter()
            .find(|c| c.info().id == calendar_id)
            .ok_or_else(|| CalendarError::NotFound(calendar_id.to_string()))?;

        let info = calendar.info();
        Ok((info.name.clone(), info.color.clone(), info.enabled))
    }

    /// Get the first available calendar ID (for selecting a default)
    pub fn get_first_calendar_id(manager: &CalendarManager) -> Option<String> {
        manager.sources().first().map(|c| c.info().id.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_empty_name() {
        let data = NewCalendarData {
            name: "".to_string(),
            color: "#FF0000".to_string(),
        };
        let result = CalendarHandler::validate(&data);
        assert!(matches!(result, Err(CalendarError::ValidationError(_))));
    }

    #[test]
    fn test_validate_valid_data() {
        let data = NewCalendarData {
            name: "Work".to_string(),
            color: "#FF0000".to_string(),
        };
        let result = CalendarHandler::validate(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_default_color() {
        let color = CalendarHandler::default_color();
        assert!(!color.is_empty());
        assert!(color.starts_with('#'));
    }
}
