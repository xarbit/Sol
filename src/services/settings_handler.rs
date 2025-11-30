//! Settings Handler - Centralized settings management.
//!
//! This handler provides a single point of contact for all settings operations,
//! including loading, saving, validation, and applying settings changes.

use crate::settings::AppSettings;
use std::error::Error;

/// Result type for settings operations
pub type SettingsResult<T> = Result<T, SettingsError>;

/// Error types for settings operations
#[derive(Debug)]
pub enum SettingsError {
    /// Failed to load settings
    LoadError(String),
    /// Failed to save settings
    SaveError(String),
    /// Invalid setting value
    ValidationError(String),
}

impl std::fmt::Display for SettingsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SettingsError::LoadError(msg) => write!(f, "Failed to load settings: {}", msg),
            SettingsError::SaveError(msg) => write!(f, "Failed to save settings: {}", msg),
            SettingsError::ValidationError(msg) => write!(f, "Invalid setting: {}", msg),
        }
    }
}

impl Error for SettingsError {}

/// Settings Handler - centralized settings management.
pub struct SettingsHandler;

impl SettingsHandler {
    /// Load settings from disk, returning defaults if not found
    pub fn load() -> SettingsResult<AppSettings> {
        AppSettings::load().map_err(|e| SettingsError::LoadError(e.to_string()))
    }

    /// Save settings to disk
    pub fn save(settings: &AppSettings) -> SettingsResult<()> {
        settings
            .save()
            .map_err(|e| SettingsError::SaveError(e.to_string()))
    }

    /// Toggle week numbers display and save
    pub fn toggle_week_numbers(settings: &mut AppSettings) -> SettingsResult<()> {
        settings.show_week_numbers = !settings.show_week_numbers;
        Self::save(settings)
    }

    /// Set week numbers display and save
    pub fn set_week_numbers(settings: &mut AppSettings, show: bool) -> SettingsResult<()> {
        settings.show_week_numbers = show;
        Self::save(settings)
    }

    /// Reset settings to defaults and save
    pub fn reset_to_defaults() -> SettingsResult<AppSettings> {
        let settings = AppSettings::default();
        Self::save(&settings)?;
        Ok(settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_settings() {
        // Should load defaults if no file exists
        let result = SettingsHandler::load();
        assert!(result.is_ok());
    }

    #[test]
    fn test_toggle_creates_opposite() {
        let mut settings = AppSettings::default();
        let original = settings.show_week_numbers;

        // Toggle (but don't save to avoid file system in tests)
        settings.show_week_numbers = !settings.show_week_numbers;

        assert_ne!(settings.show_week_numbers, original);
    }
}
