//! Dialog state and management.
//!
//! This module provides centralized dialog management for the application.
//! All dialogs, popups, and modals are managed through the `DialogManager`,
//! ensuring consistent behavior:
//!
//! - Only one dialog can be open at a time
//! - Opening a new dialog closes any existing one
//! - Escape key closes the current dialog
//! - Focus loss behavior is handled uniformly
//!
//! # Usage
//!
//! ```rust
//! use crate::dialogs::{ActiveDialog, DialogManager, DialogAction};
//!
//! // Open a dialog
//! DialogManager::open(&mut app.active_dialog, ActiveDialog::ColorPicker { calendar_id });
//!
//! // Close via Escape
//! DialogManager::handle_escape(&mut app.active_dialog);
//!
//! // Check if dialog is open
//! if app.active_dialog.is_open() { ... }
//! ```

mod manager;
mod event_dialog;
mod calendar_dialog;
mod import_dialog;
mod import_progress_dialog;
mod import_result_dialog;

pub use manager::{
    ActiveDialog,
    DialogAction,
    DialogManager,
    QuickEventResult,
};
pub use import_dialog::render_import_dialog;
pub use import_progress_dialog::render_import_progress_dialog;
pub use import_result_dialog::render_import_result_dialog;

// Keep old exports for backwards compatibility during migration
// Note: EventDialogData is managed through legacy fields because text_editor::Content
// doesn't implement Clone
#[allow(unused_imports)]
pub use event_dialog::EventDialogUiState;
#[allow(unused_imports)]
pub use calendar_dialog::{CalendarDialogData, CalendarDialogMode};
