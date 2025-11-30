//! Week view selection handler
//!
//! In week view, drag selection on time slots creates timed events:
//! - Drag on time slots: Open event dialog with specific start/end times
//! - Drag on day headers: Create all-day event (future)
//!
//! TODO: Implement time-based selection when week view supports it

use chrono::NaiveDate;
use log::debug;

use crate::app::CosmicCalendar;

/// Handle selection end in week view
/// Currently just selects the day - time-based selection to be implemented
pub fn handle_selection_end(app: &mut CosmicCalendar, start: NaiveDate, _end: NaiveDate) {
    debug!("week::handle_selection_end: Week view selection not yet implemented");
    // For now, just select the start day
    app.set_selected_date(start);

    // Future implementation:
    // - If selection was on time slots, open event dialog with times
    // - If selection was on day headers, create all-day event
}
