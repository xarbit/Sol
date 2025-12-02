//! Dialog Manager - Centralized dialog/popup state management.
//!
//! This module provides a unified system for managing all dialogs, popups,
//! and modal overlays in the application. It ensures:
//!
//! - Only one dialog can be open at a time
//! - Opening a new dialog automatically closes any existing one
//! - Escape key closes the current dialog
//! - Consistent focus management across the app
//!
//! # Architecture
//!
//! ```text
//! User Action (click, keyboard)
//!         │
//!         ▼
//! Message::Dialog(DialogAction)
//!         │
//!         ▼
//! DialogManager::handle_action()
//!         │
//!         ├── Opens new dialog (closes existing)
//!         ├── Closes current dialog
//!         └── Updates dialog state
//! ```
//!
//! # Note on Event Dialogs
//!
//! Event dialogs are currently managed through the legacy `event_dialog` field
//! in `CosmicCalendar` because `text_editor::Content` doesn't implement `Clone`.
//! The centralized `Message::CloseDialog` handler closes all legacy dialog fields.

use crate::caldav::CalendarEvent;
use chrono::{NaiveDate, NaiveTime};
use log::{debug, info};

/// Identifies which dialog or transient UI element is currently active.
/// Only one can be open at a time - opening a new one closes the existing.
///
/// This includes:
/// - Modal dialogs (calendar create/edit, delete confirmation)
/// - Popovers (color picker)
/// - Inline inputs (quick event)
/// - Full dialogs (event dialog - state in legacy field due to text_editor::Content)
#[derive(Debug, Clone, PartialEq)]
pub enum ActiveDialog {
    /// No dialog is open
    None,
    /// Quick event input for single or multi-day events
    /// - Single day (start == end): double-click on a day
    /// - Multi-day (start != end): drag selection across days
    /// - Timed event: drag selection on hour cells in week/day view
    /// The input appears on the start date
    QuickEvent {
        /// Start date of the event
        start_date: NaiveDate,
        /// End date of the event (same as start for single-day)
        end_date: NaiveDate,
        /// Start time (None for all-day events)
        start_time: Option<NaiveTime>,
        /// End time (None for all-day events)
        end_time: Option<NaiveTime>,
        /// Event title being typed
        text: String,
    },
    /// Color picker for a specific calendar
    ColorPicker {
        calendar_id: String,
    },
    /// Create new calendar dialog
    CalendarCreate {
        name: String,
        color: String,
    },
    /// Edit existing calendar dialog
    CalendarEdit {
        calendar_id: String,
        name: String,
        color: String,
    },
    /// Delete calendar confirmation dialog
    CalendarDelete {
        calendar_id: String,
        calendar_name: String,
    },
    /// Delete event confirmation dialog
    EventDelete {
        /// Event UID to delete (may be occurrence UID like "master-uid_YYYYMMDD")
        event_uid: String,
        /// Event name for display in confirmation
        event_name: String,
        /// Whether this is a recurring event
        is_recurring: bool,
        /// The date of the occurrence being deleted (for adding to exception_dates)
        /// This is extracted from occurrence UIDs (format: master-uid_YYYYMMDD)
        occurrence_date: Option<chrono::NaiveDate>,
    },
    /// Event dialog is open (state managed by legacy field)
    /// This variant exists to track that an event dialog is open,
    /// but the actual data is in `app.event_dialog`
    EventDialogOpen,
    /// Import events dialog
    Import {
        /// Events to import
        events: Vec<CalendarEvent>,
        /// Source file name for display
        source_file_name: String,
        /// Selected target calendar ID
        selected_calendar_id: Option<String>,
    },
    /// Import progress dialog for large imports
    ImportProgress {
        /// Current event being imported (1-indexed)
        current: usize,
        /// Total number of events to import
        total: usize,
        /// Currently importing event summary
        current_event: String,
        /// Log of imported events (for display)
        import_log: Vec<String>,
        /// List of imported event UIDs (for rollback)
        imported_uids: Vec<String>,
        /// Target calendar ID
        calendar_id: String,
    },
    /// Import result dialog shown after import completes
    ImportResult {
        /// Whether the import was successful
        success: bool,
        /// Number of events successfully imported
        imported_count: usize,
        /// Number of events skipped (duplicates)
        skipped_count: usize,
        /// Number of events that failed to import
        failed_count: usize,
        /// Source file name for display
        source_file_name: String,
        /// Target calendar name for display
        calendar_name: String,
        /// List of imported event UIDs (for potential revert)
        imported_uids: Vec<String>,
        /// Target calendar ID (for revert)
        calendar_id: String,
        /// Optional error message if import failed
        error_message: Option<String>,
    },
}

impl Default for ActiveDialog {
    fn default() -> Self {
        Self::None
    }
}

impl ActiveDialog {
    /// Check if any dialog is currently open
    pub fn is_open(&self) -> bool {
        !matches!(self, ActiveDialog::None)
    }

    /// Check if a quick event input is open
    pub fn is_quick_event(&self) -> bool {
        matches!(self, ActiveDialog::QuickEvent { .. })
    }

    /// Check if a specific dialog type is open
    #[allow(dead_code)] // Reserved for future dialog type checking
    pub fn is_color_picker(&self) -> bool {
        matches!(self, ActiveDialog::ColorPicker { .. })
    }

    #[allow(dead_code)] // Reserved for future dialog type checking
    pub fn is_calendar_dialog(&self) -> bool {
        matches!(
            self,
            ActiveDialog::CalendarCreate { .. }
                | ActiveDialog::CalendarEdit { .. }
                | ActiveDialog::CalendarDelete { .. }
        )
    }

    #[allow(dead_code)] // Reserved for future dialog type checking
    pub fn is_event_dialog(&self) -> bool {
        matches!(self, ActiveDialog::EventDialogOpen)
    }

    /// Check if this is an event delete confirmation dialog
    #[allow(dead_code)] // Reserved for future dialog type checking
    pub fn is_event_delete(&self) -> bool {
        matches!(self, ActiveDialog::EventDelete { .. })
    }

    /// Check if this is an import dialog
    #[allow(dead_code)] // Reserved for import dialog handling
    pub fn is_import(&self) -> bool {
        matches!(self, ActiveDialog::Import { .. })
    }

    /// Get import dialog data if open (events, source_file_name, selected_calendar_id)
    #[allow(dead_code)] // Reserved for import dialog rendering
    pub fn import_data(&self) -> Option<(&[CalendarEvent], &str, Option<&str>)> {
        match self {
            ActiveDialog::Import {
                events,
                source_file_name,
                selected_calendar_id,
            } => Some((
                events,
                source_file_name,
                selected_calendar_id.as_deref(),
            )),
            _ => None,
        }
    }

    /// Get event delete data if this is an event delete dialog
    /// Returns (event_uid, event_name, is_recurring, occurrence_date)
    pub fn event_delete_data(&self) -> Option<(&str, &str, bool, Option<chrono::NaiveDate>)> {
        match self {
            ActiveDialog::EventDelete { event_uid, event_name, is_recurring, occurrence_date } => {
                Some((event_uid, event_name, *is_recurring, *occurrence_date))
            }
            _ => None,
        }
    }

    /// Get the color picker calendar ID if open
    pub fn color_picker_calendar_id(&self) -> Option<&str> {
        match self {
            ActiveDialog::ColorPicker { calendar_id } => Some(calendar_id),
            _ => None,
        }
    }

    /// Get quick event data if editing (start_date, text)
    /// Returns the start date for display purposes (input appears on start date)
    pub fn quick_event_data(&self) -> Option<(NaiveDate, &str)> {
        match self {
            ActiveDialog::QuickEvent { start_date, text, .. } => Some((*start_date, text)),
            _ => None,
        }
    }

    /// Get full quick event range if editing (start_date, end_date, text)
    pub fn quick_event_range(&self) -> Option<(NaiveDate, NaiveDate, &str)> {
        match self {
            ActiveDialog::QuickEvent { start_date, end_date, text, .. } => {
                Some((*start_date, *end_date, text))
            }
            _ => None,
        }
    }

    /// Get quick event times if this is a timed event
    pub fn quick_event_times(&self) -> Option<(NaiveTime, NaiveTime)> {
        match self {
            ActiveDialog::QuickEvent { start_time: Some(start), end_time: Some(end), .. } => {
                Some((*start, *end))
            }
            _ => None,
        }
    }

    /// Check if the quick event is a timed event (has times)
    #[allow(dead_code)] // Reserved for future timed quick event handling
    pub fn is_timed_quick_event(&self) -> bool {
        matches!(self, ActiveDialog::QuickEvent { start_time: Some(_), .. })
    }

    /// Check if the quick event spans multiple days
    pub fn is_multi_day_quick_event(&self) -> bool {
        match self {
            ActiveDialog::QuickEvent { start_date, end_date, .. } => start_date != end_date,
            _ => false,
        }
    }

    /// Check if quick event has empty text (for dismissal on focus loss)
    pub fn is_quick_event_empty(&self) -> bool {
        match self {
            ActiveDialog::QuickEvent { text, .. } => text.trim().is_empty(),
            _ => false,
        }
    }

    /// Check if a date is within the quick event date range
    /// Used for showing selection highlight while quick event input is open
    pub fn is_date_in_quick_event_range(&self, date: NaiveDate) -> bool {
        match self {
            ActiveDialog::QuickEvent { start_date, end_date, .. } => {
                date >= *start_date && date <= *end_date
            }
            _ => false,
        }
    }
}

/// Actions that can be performed on dialogs
#[allow(dead_code)] // Part of dialog action API
#[derive(Debug, Clone)]
pub enum DialogAction {
    /// Close any open dialog (dismisses empty quick events, cancels others)
    Close,
    /// Close dialog and perform cleanup/confirm action
    CloseAndConfirm,
    /// Start a quick event on a single date (double-click)
    StartQuickEvent(NaiveDate),
    /// Start a quick event spanning a date range (drag selection)
    StartQuickEventRange { start: NaiveDate, end: NaiveDate },
    /// Start a quick timed event with specific times (time slot selection in week/day view)
    StartQuickTimedEvent { date: NaiveDate, start_time: NaiveTime, end_time: NaiveTime },
    /// Update quick event text while typing
    QuickEventTextChanged(String),
    /// Commit the quick event (create the event)
    CommitQuickEvent,
    /// Open color picker for a calendar
    OpenColorPicker(String),
    /// Open create calendar dialog
    OpenCalendarCreate { default_color: String },
    /// Open edit calendar dialog
    OpenCalendarEdit {
        calendar_id: String,
        name: String,
        color: String,
    },
    /// Open delete calendar confirmation
    OpenCalendarDelete {
        calendar_id: String,
        calendar_name: String,
    },
    /// Mark event dialog as open (actual state is in legacy field)
    MarkEventDialogOpen,
    /// Update calendar dialog name
    CalendarNameChanged(String),
    /// Update calendar dialog color
    CalendarColorChanged(String),
    /// Open import dialog with events and source file name
    OpenImport {
        events: Vec<CalendarEvent>,
        source_file_name: String,
    },
    /// Select target calendar for import
    SelectImportCalendar(String),
}

/// Dialog Manager handles all dialog state transitions
pub struct DialogManager;

impl DialogManager {
    /// Open a new dialog, closing any existing one
    pub fn open(current: &mut ActiveDialog, new_dialog: ActiveDialog) {
        if current.is_open() {
            debug!("DialogManager: Closing existing dialog before opening new one");
        }
        info!("DialogManager: Opening {:?}", std::mem::discriminant(&new_dialog));
        *current = new_dialog;
    }

    /// Close the current dialog
    pub fn close(current: &mut ActiveDialog) {
        if current.is_open() {
            info!("DialogManager: Closing dialog");
            *current = ActiveDialog::None;
        }
    }

    /// Close dialog if it's an empty quick event, otherwise keep it open
    /// This is called when focus is lost (clicking elsewhere)
    /// Returns true if a quick event was dismissed
    pub fn dismiss_empty_quick_event(current: &mut ActiveDialog) -> bool {
        if current.is_quick_event_empty() {
            debug!("DialogManager: Dismissing empty quick event");
            *current = ActiveDialog::None;
            true
        } else {
            false
        }
    }

    /// Close dialog if Escape was pressed
    /// Returns true if a dialog was closed
    #[allow(dead_code)] // Reserved for keyboard handling
    pub fn handle_escape(current: &mut ActiveDialog) -> bool {
        if current.is_open() {
            info!("DialogManager: Escape pressed, closing dialog");
            *current = ActiveDialog::None;
            true
        } else {
            false
        }
    }

    /// Handle a dialog action, returns true if the action requires further processing
    /// (e.g., CommitQuickEvent needs the caller to actually create the event)
    pub fn handle_action(current: &mut ActiveDialog, action: DialogAction) -> Option<QuickEventResult> {
        match action {
            DialogAction::Close => {
                Self::close(current);
                None
            }
            DialogAction::CloseAndConfirm => {
                // The confirm logic should be handled by the caller
                // This just signals the dialog should close after confirmation
                Self::close(current);
                None
            }
            DialogAction::StartQuickEvent(date) => {
                Self::open(
                    current,
                    ActiveDialog::QuickEvent {
                        start_date: date,
                        end_date: date,
                        start_time: None,
                        end_time: None,
                        text: String::new(),
                    },
                );
                None
            }
            DialogAction::StartQuickEventRange { start, end } => {
                Self::open(
                    current,
                    ActiveDialog::QuickEvent {
                        start_date: start,
                        end_date: end,
                        start_time: None,
                        end_time: None,
                        text: String::new(),
                    },
                );
                None
            }
            DialogAction::StartQuickTimedEvent { date, start_time, end_time } => {
                Self::open(
                    current,
                    ActiveDialog::QuickEvent {
                        start_date: date,
                        end_date: date,
                        start_time: Some(start_time),
                        end_time: Some(end_time),
                        text: String::new(),
                    },
                );
                None
            }
            DialogAction::QuickEventTextChanged(text) => {
                if let ActiveDialog::QuickEvent { text: t, .. } = current {
                    *t = text;
                }
                None
            }
            DialogAction::CommitQuickEvent => {
                // Extract the data before closing, return it for the caller to process
                if let ActiveDialog::QuickEvent { start_date, end_date, start_time, end_time, text } = current {
                    let result = QuickEventResult {
                        start_date: *start_date,
                        end_date: *end_date,
                        start_time: *start_time,
                        end_time: *end_time,
                        text: text.clone(),
                    };
                    *current = ActiveDialog::None;
                    Some(result)
                } else {
                    None
                }
            }
            DialogAction::OpenColorPicker(calendar_id) => {
                Self::open(current, ActiveDialog::ColorPicker { calendar_id });
                None
            }
            DialogAction::OpenCalendarCreate { default_color } => {
                Self::open(
                    current,
                    ActiveDialog::CalendarCreate {
                        name: String::new(),
                        color: default_color,
                    },
                );
                None
            }
            DialogAction::OpenCalendarEdit {
                calendar_id,
                name,
                color,
            } => {
                Self::open(
                    current,
                    ActiveDialog::CalendarEdit {
                        calendar_id,
                        name,
                        color,
                    },
                );
                None
            }
            DialogAction::OpenCalendarDelete {
                calendar_id,
                calendar_name,
            } => {
                Self::open(
                    current,
                    ActiveDialog::CalendarDelete {
                        calendar_id,
                        calendar_name,
                    },
                );
                None
            }
            DialogAction::MarkEventDialogOpen => {
                Self::open(current, ActiveDialog::EventDialogOpen);
                None
            }
            DialogAction::CalendarNameChanged(name) => {
                match current {
                    ActiveDialog::CalendarCreate { name: n, .. }
                    | ActiveDialog::CalendarEdit { name: n, .. } => {
                        *n = name;
                    }
                    _ => {}
                }
                None
            }
            DialogAction::CalendarColorChanged(color) => {
                match current {
                    ActiveDialog::CalendarCreate { color: c, .. }
                    | ActiveDialog::CalendarEdit { color: c, .. } => {
                        *c = color;
                    }
                    _ => {}
                }
                None
            }
            DialogAction::OpenImport {
                events,
                source_file_name,
            } => {
                Self::open(
                    current,
                    ActiveDialog::Import {
                        events,
                        source_file_name,
                        selected_calendar_id: None,
                    },
                );
                None
            }
            DialogAction::SelectImportCalendar(calendar_id) => {
                if let ActiveDialog::Import {
                    selected_calendar_id,
                    ..
                } = current
                {
                    *selected_calendar_id = Some(calendar_id);
                }
                None
            }
        }
    }
}

/// Result returned when committing a quick event
/// Contains the data needed to create the actual event
#[derive(Debug, Clone)]
pub struct QuickEventResult {
    /// Start date of the event
    pub start_date: NaiveDate,
    /// End date of the event (same as start for single-day)
    pub end_date: NaiveDate,
    /// Start time (None for all-day events)
    pub start_time: Option<NaiveTime>,
    /// End time (None for all-day events)
    pub end_time: Option<NaiveTime>,
    /// Event title
    pub text: String,
}

impl QuickEventResult {
    /// Check if this is a multi-day event
    #[allow(dead_code)] // Reserved for event creation logic
    pub fn is_multi_day(&self) -> bool {
        self.start_date != self.end_date
    }

    /// Check if this is a timed event (has times)
    #[allow(dead_code)] // Reserved for event creation logic
    pub fn is_timed(&self) -> bool {
        self.start_time.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialog_none_by_default() {
        let dialog = ActiveDialog::default();
        assert!(!dialog.is_open());
    }

    #[test]
    fn test_open_closes_existing() {
        let mut dialog = ActiveDialog::ColorPicker {
            calendar_id: "test".to_string(),
        };
        assert!(dialog.is_open());

        DialogManager::open(
            &mut dialog,
            ActiveDialog::CalendarCreate {
                name: String::new(),
                color: "#FF0000".to_string(),
            },
        );

        assert!(dialog.is_calendar_dialog());
        assert!(!dialog.is_color_picker());
    }

    #[test]
    fn test_escape_closes_dialog() {
        let mut dialog = ActiveDialog::ColorPicker {
            calendar_id: "test".to_string(),
        };

        let closed = DialogManager::handle_escape(&mut dialog);

        assert!(closed);
        assert!(!dialog.is_open());
    }

    #[test]
    fn test_escape_does_nothing_when_no_dialog() {
        let mut dialog = ActiveDialog::None;

        let closed = DialogManager::handle_escape(&mut dialog);

        assert!(!closed);
        assert!(!dialog.is_open());
    }

    #[test]
    fn test_quick_event_start() {
        let mut dialog = ActiveDialog::None;
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        DialogManager::handle_action(&mut dialog, DialogAction::StartQuickEvent(date));

        assert!(dialog.is_quick_event());
        assert!(dialog.is_quick_event_empty());
        assert_eq!(dialog.quick_event_data(), Some((date, "")));
    }

    #[test]
    fn test_quick_event_text_change() {
        let mut dialog = ActiveDialog::None;
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        DialogManager::handle_action(&mut dialog, DialogAction::StartQuickEvent(date));
        DialogManager::handle_action(&mut dialog, DialogAction::QuickEventTextChanged("Meeting".to_string()));

        assert!(!dialog.is_quick_event_empty());
        assert_eq!(dialog.quick_event_data(), Some((date, "Meeting")));
    }

    #[test]
    fn test_quick_event_commit() {
        let mut dialog = ActiveDialog::None;
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        DialogManager::handle_action(&mut dialog, DialogAction::StartQuickEvent(date));
        DialogManager::handle_action(&mut dialog, DialogAction::QuickEventTextChanged("Meeting".to_string()));
        let result = DialogManager::handle_action(&mut dialog, DialogAction::CommitQuickEvent);

        assert!(!dialog.is_open());
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.start_date, date);
        assert_eq!(result.end_date, date);
        assert!(!result.is_multi_day());
        assert_eq!(result.text, "Meeting");
    }

    #[test]
    fn test_multi_day_quick_event() {
        let mut dialog = ActiveDialog::None;
        let start = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 18).unwrap();

        DialogManager::handle_action(&mut dialog, DialogAction::StartQuickEventRange { start, end });
        DialogManager::handle_action(&mut dialog, DialogAction::QuickEventTextChanged("Vacation".to_string()));

        assert!(dialog.is_quick_event());
        assert!(dialog.is_multi_day_quick_event());
        assert_eq!(dialog.quick_event_range(), Some((start, end, "Vacation")));

        let result = DialogManager::handle_action(&mut dialog, DialogAction::CommitQuickEvent);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.start_date, start);
        assert_eq!(result.end_date, end);
        assert!(result.is_multi_day());
        assert_eq!(result.text, "Vacation");
    }

    #[test]
    fn test_dismiss_empty_quick_event() {
        let mut dialog = ActiveDialog::None;
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        DialogManager::handle_action(&mut dialog, DialogAction::StartQuickEvent(date));

        // Should dismiss because text is empty
        let dismissed = DialogManager::dismiss_empty_quick_event(&mut dialog);

        assert!(dismissed);
        assert!(!dialog.is_open());
    }

    #[test]
    fn test_dismiss_does_not_close_non_empty_quick_event() {
        let mut dialog = ActiveDialog::None;
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        DialogManager::handle_action(&mut dialog, DialogAction::StartQuickEvent(date));
        DialogManager::handle_action(&mut dialog, DialogAction::QuickEventTextChanged("Meeting".to_string()));

        // Should NOT dismiss because text is not empty
        let dismissed = DialogManager::dismiss_empty_quick_event(&mut dialog);

        assert!(!dismissed);
        assert!(dialog.is_open());
    }

    #[test]
    fn test_opening_dialog_closes_quick_event() {
        let mut dialog = ActiveDialog::None;
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        DialogManager::handle_action(&mut dialog, DialogAction::StartQuickEvent(date));
        assert!(dialog.is_quick_event());

        // Opening a calendar dialog should close the quick event
        DialogManager::handle_action(&mut dialog, DialogAction::OpenCalendarCreate {
            default_color: "#FF0000".to_string(),
        });

        assert!(!dialog.is_quick_event());
        assert!(dialog.is_calendar_dialog());
    }
}
