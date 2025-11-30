//! Day view selection handler
//!
//! In day view, drag selection on time slots creates timed events:
//! - Drag on time slots: Open event dialog with specific start/end times
//! - Drag on date header: Create all-day event (future)
//!
//! TODO: Implement time-based selection when day view supports it

use chrono::NaiveDate;
use log::debug;

use crate::app::CosmicCalendar;

/// Handle selection end in day view
/// Currently just selects the day - time-based selection to be implemented
pub fn handle_selection_end(app: &mut CosmicCalendar, start: NaiveDate, _end: NaiveDate) {
    debug!("day::handle_selection_end: Day view selection not yet implemented");
    // For now, just select the start day
    app.set_selected_date(start);

    // Future implementation:
    // - If selection was on time slots, open event dialog with times
    // - If selection was on date header, create all-day event
}
