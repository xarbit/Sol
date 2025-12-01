//! Calendar Handler - Centralized calendar management.
//!
//! This handler manages calendar CRUD operations (not events, but the calendars themselves).
//! It handles creating, editing, deleting calendars, toggling visibility, and color changes.

use crate::calendars::CalendarManager;
use crate::components::color_picker::CALENDAR_COLORS;
use log::{debug, error, info, warn};
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

        debug!("CalendarHandler: Generated unique ID '{}' from name '{}'", unique_id, name);
        unique_id
    }

    /// Validate calendar data before creating/updating
    pub fn validate(data: &NewCalendarData) -> CalendarResult<()> {
        if data.name.trim().is_empty() {
            warn!("CalendarHandler: Validation failed - empty name");
            return Err(CalendarError::ValidationError(
                "Calendar name is required".to_string(),
            ));
        }

        if data.color.is_empty() {
            warn!("CalendarHandler: Validation failed - empty color");
            return Err(CalendarError::ValidationError(
                "Calendar color is required".to_string(),
            ));
        }

        Ok(())
    }

    /// Create a new calendar
    pub fn create(manager: &mut CalendarManager, data: NewCalendarData) -> CalendarResult<String> {
        info!("CalendarHandler: Creating calendar '{}'", data.name);

        // Validate
        Self::validate(&data)?;

        // Generate unique ID
        let id = Self::generate_id(&data.name, manager);

        // Add the calendar
        debug!("CalendarHandler: Adding calendar id='{}' name='{}' color='{}'",
               id, data.name, data.color);
        manager.add_local_calendar(id.clone(), data.name.clone(), data.color);

        info!("CalendarHandler: Successfully created calendar '{}' (id={})", data.name, id);
        Ok(id)
    }

    /// Update an existing calendar
    pub fn update(
        manager: &mut CalendarManager,
        calendar_id: &str,
        data: UpdateCalendarData,
    ) -> CalendarResult<()> {
        info!("CalendarHandler: Updating calendar '{}'", calendar_id);

        let calendar = manager
            .sources_mut()
            .iter_mut()
            .find(|c| c.info().id == calendar_id)
            .ok_or_else(|| {
                error!("CalendarHandler: Calendar '{}' not found for update", calendar_id);
                CalendarError::NotFound(calendar_id.to_string())
            })?;

        // Apply updates
        if let Some(name) = data.name {
            if name.trim().is_empty() {
                warn!("CalendarHandler: Update rejected - empty name for '{}'", calendar_id);
                return Err(CalendarError::ValidationError(
                    "Calendar name cannot be empty".to_string(),
                ));
            }
            debug!("CalendarHandler: Updating name to '{}'", name);
            calendar.info_mut().name = name;
        }

        if let Some(color) = data.color {
            debug!("CalendarHandler: Updating color to '{}'", color);
            calendar.info_mut().color = color;
        }

        if let Some(enabled) = data.enabled {
            debug!("CalendarHandler: Updating enabled to {}", enabled);
            calendar.set_enabled(enabled);
        }

        // Save configuration
        manager
            .save_config()
            .map_err(|e| {
                error!("CalendarHandler: Failed to save config: {}", e);
                CalendarError::ConfigError(e.to_string())
            })?;

        info!("CalendarHandler: Successfully updated calendar '{}'", calendar_id);
        Ok(())
    }

    /// Toggle a calendar's enabled state
    pub fn toggle_enabled(manager: &mut CalendarManager, calendar_id: &str) -> CalendarResult<bool> {
        debug!("CalendarHandler: Toggling enabled state for '{}'", calendar_id);

        let calendar = manager
            .sources_mut()
            .iter_mut()
            .find(|c| c.info().id == calendar_id)
            .ok_or_else(|| {
                error!("CalendarHandler: Calendar '{}' not found for toggle", calendar_id);
                CalendarError::NotFound(calendar_id.to_string())
            })?;

        let new_state = !calendar.is_enabled();
        calendar.set_enabled(new_state);

        info!("CalendarHandler: Calendar '{}' enabled={}", calendar_id, new_state);

        // Save configuration
        manager
            .save_config()
            .map_err(|e| {
                error!("CalendarHandler: Failed to save config after toggle: {}", e);
                CalendarError::ConfigError(e.to_string())
            })?;

        Ok(new_state)
    }

    /// Change a calendar's color
    pub fn change_color(
        manager: &mut CalendarManager,
        calendar_id: &str,
        color: String,
    ) -> CalendarResult<()> {
        info!("CalendarHandler: Changing color for '{}' to '{}'", calendar_id, color);
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
        info!("CalendarHandler: Deleting calendar '{}'", calendar_id);

        if !manager.delete_calendar(calendar_id) {
            error!("CalendarHandler: Calendar '{}' not found for deletion", calendar_id);
            return Err(CalendarError::NotFound(calendar_id.to_string()));
        }

        info!("CalendarHandler: Successfully deleted calendar '{}'", calendar_id);
        Ok(())
    }

    /// Get calendar info by ID
    pub fn get_info(
        manager: &CalendarManager,
        calendar_id: &str,
    ) -> CalendarResult<(String, String, bool)> {
        debug!("CalendarHandler: Getting info for calendar '{}'", calendar_id);

        let calendar = manager
            .sources()
            .iter()
            .find(|c| c.info().id == calendar_id)
            .ok_or_else(|| {
                debug!("CalendarHandler: Calendar '{}' not found", calendar_id);
                CalendarError::NotFound(calendar_id.to_string())
            })?;

        let info = calendar.info();
        Ok((info.name.clone(), info.color.clone(), info.enabled))
    }

    /// Get the first available calendar ID (for selecting a default)
    pub fn get_first_calendar_id(manager: &CalendarManager) -> Option<String> {
        let id = manager.sources().first().map(|c| c.info().id.clone());
        debug!("CalendarHandler: First calendar ID: {:?}", id);
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Validation Tests ====================

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
    fn test_validate_whitespace_only_name() {
        let data = NewCalendarData {
            name: "   ".to_string(),
            color: "#FF0000".to_string(),
        };
        let result = CalendarHandler::validate(&data);
        assert!(matches!(result, Err(CalendarError::ValidationError(_))));
    }

    #[test]
    fn test_validate_empty_color() {
        let data = NewCalendarData {
            name: "Work".to_string(),
            color: "".to_string(),
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
    fn test_validate_unicode_name() {
        let data = NewCalendarData {
            name: "工作日历 📅".to_string(),
            color: "#3B82F6".to_string(),
        };
        let result = CalendarHandler::validate(&data);
        assert!(result.is_ok());
    }

    // ==================== Default Color Tests ====================

    #[test]
    fn test_default_color() {
        let color = CalendarHandler::default_color();
        assert!(!color.is_empty());
        assert!(color.starts_with('#'));
    }

    #[test]
    fn test_default_color_is_valid_hex() {
        let color = CalendarHandler::default_color();
        assert!(color.len() == 7); // #RRGGBB format
        assert!(color.chars().skip(1).all(|c| c.is_ascii_hexdigit()));
    }

    // ==================== CalendarError Display Tests ====================

    #[test]
    fn test_calendar_error_display_not_found() {
        let error = CalendarError::NotFound("cal-123".to_string());
        assert_eq!(error.to_string(), "Calendar not found: cal-123");
    }

    #[test]
    fn test_calendar_error_display_validation() {
        let error = CalendarError::ValidationError("Name required".to_string());
        assert_eq!(error.to_string(), "Invalid calendar: Name required");
    }

    #[test]
    fn test_calendar_error_display_config() {
        let error = CalendarError::ConfigError("Write failed".to_string());
        assert_eq!(error.to_string(), "Config error: Write failed");
    }

    #[test]
    fn test_calendar_error_display_duplicate_id() {
        let error = CalendarError::DuplicateId("work".to_string());
        assert_eq!(error.to_string(), "Calendar ID already exists: work");
    }

    // ==================== CalendarError Debug Tests ====================

    #[test]
    fn test_calendar_error_debug() {
        let error = CalendarError::NotFound("test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("NotFound"));
    }
}
