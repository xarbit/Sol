//! Selection handlers for multi-day event creation
//!
//! This module provides view-specific selection handling. Each view has its own
//! submodule with tailored behavior:
//!
//! - **month**: Quick create all-day events (no dialog)
//! - **week**: Open event dialog with specific times (future)
//! - **day**: Open event dialog with specific times (future)
//!
//! The core selection logic (start, update, cancel) is shared across all views.

mod day;
mod month;
mod week;

use chrono::NaiveDate;
use log::debug;

use crate::app::CosmicCalendar;
use crate::views::CalendarView;

/// Start a drag selection at the given date (mouse press on day cell)
/// This is view-agnostic - all views start selection the same way
pub fn handle_selection_start(app: &mut CosmicCalendar, date: NaiveDate) {
    debug!("handle_selection_start: Starting selection at {}", date);
    app.selection_state.start(date);
}

/// Update the selection end point (mouse move while dragging)
/// This is view-agnostic - all views update selection the same way
pub fn handle_selection_update(app: &mut CosmicCalendar, date: NaiveDate) {
    if app.selection_state.is_active {
        debug!("handle_selection_update: Updating selection to {}", date);
        app.selection_state.update(date);
    }
}

/// End the selection (mouse release)
/// Dispatches to view-specific handlers based on current view
pub fn handle_selection_end(app: &mut CosmicCalendar) {
    debug!("handle_selection_end: Ending selection in {:?} view", app.current_view);

    let Some(range) = app.selection_state.end() else {
        return;
    };

    match app.current_view {
        CalendarView::Month => {
            month::handle_selection_end(app, range.start.date, range.end.date);
        }
        CalendarView::Week => {
            week::handle_selection_end(app, range.start.date, range.end.date);
        }
        CalendarView::Day => {
            day::handle_selection_end(app, range.start.date, range.end.date);
        }
        CalendarView::Year => {
            // Year view: just select the day (no special selection behavior)
            app.set_selected_date(range.start.date);
        }
    }
}

/// Cancel the current selection
pub fn handle_selection_cancel(app: &mut CosmicCalendar) {
    debug!("handle_selection_cancel: Cancelling selection");
    app.selection_state.cancel();
}
