//! Month view selection handler
//!
//! In month view, drag selection creates all-day events:
//! - Single day: Select the day (standard click behavior)
//! - Multi-day: Open quick event input for naming the event, then create all-day event

use chrono::NaiveDate;
use log::debug;

use crate::app::CosmicCalendar;
use crate::dialogs::{DialogAction, DialogManager};

/// Handle selection end in month view
pub fn handle_selection_end(app: &mut CosmicCalendar, start: NaiveDate, end: NaiveDate) {
    if start == end {
        // Single day selection - just select the day
        debug!("month::handle_selection_end: Single day selection at {}", start);
        app.set_selected_date(start);
    } else {
        // Multi-day selection - open quick event input for naming
        debug!(
            "month::handle_selection_end: Multi-day selection from {} to {}, opening quick event input",
            start, end
        );
        // Start quick event with the date range - user can type the name, then press Enter to save
        DialogManager::handle_action(
            &mut app.active_dialog,
            DialogAction::StartQuickEventRange { start, end },
        );
        // Select the start date to show where the input appears
        app.set_selected_date(start);
    }
}
