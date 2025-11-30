//! Calendar management handlers (create, edit, delete, toggle, color)

use crate::app::CosmicCalendar;
use crate::dialogs::{ActiveDialog, DialogManager};
use crate::services::{CalendarHandler, NewCalendarData, UpdateCalendarData};
use log::{debug, error, info, warn};

/// Toggle a calendar's enabled state and save configuration
pub fn handle_toggle_calendar(app: &mut CosmicCalendar, id: String) {
    debug!("handle_toggle_calendar: Toggling calendar '{}'", id);

    match CalendarHandler::toggle_enabled(&mut app.calendar_manager, &id) {
        Ok(new_state) => {
            info!("Calendar '{}' toggled to enabled={}", id, new_state);
            // Refresh events to show/hide events from toggled calendar
            app.refresh_cached_events();
        }
        Err(e) => {
            error!("Failed to toggle calendar '{}': {}", id, e);
        }
    }
}

/// Change a calendar's color and save configuration
pub fn handle_change_calendar_color(app: &mut CosmicCalendar, id: String, color: String) {
    debug!("handle_change_calendar_color: Changing color for '{}' to '{}'", id, color);

    match CalendarHandler::change_color(&mut app.calendar_manager, &id, color.clone()) {
        Ok(()) => {
            info!("Calendar '{}' color changed to '{}'", id, color);
            // Close the color picker after selection
            DialogManager::close(&mut app.active_dialog);
            // Refresh events to update event colors in views
            app.refresh_cached_events();
            // Also update selected calendar color if this was the selected calendar
            app.update_selected_calendar_color();
        }
        Err(e) => {
            error!("Failed to change calendar color: {}", e);
        }
    }
}

/// Open the calendar dialog in Create mode
pub fn handle_open_calendar_dialog_create(app: &mut CosmicCalendar) {
    debug!("handle_open_calendar_dialog_create: Opening create dialog");

    let default_color = CalendarHandler::default_color();

    DialogManager::open(
        &mut app.active_dialog,
        ActiveDialog::CalendarCreate {
            name: String::new(),
            color: default_color,
        },
    );
}

/// Open the calendar dialog in Edit mode for a specific calendar
pub fn handle_open_calendar_dialog_edit(app: &mut CosmicCalendar, calendar_id: String) {
    debug!("handle_open_calendar_dialog_edit: Opening edit dialog for '{}'", calendar_id);

    match CalendarHandler::get_info(&app.calendar_manager, &calendar_id) {
        Ok((name, color, _enabled)) => {
            DialogManager::open(
                &mut app.active_dialog,
                ActiveDialog::CalendarEdit {
                    calendar_id,
                    name,
                    color,
                },
            );
        }
        Err(e) => {
            warn!("Cannot edit calendar '{}': {}", calendar_id, e);
        }
    }
}

/// Confirm the calendar dialog (Create or Edit)
pub fn handle_confirm_calendar_dialog(app: &mut CosmicCalendar) {
    // Extract data from active_dialog before closing
    let dialog_data = match &app.active_dialog {
        ActiveDialog::CalendarCreate { name, color } => {
            Some((None, name.clone(), color.clone()))
        }
        ActiveDialog::CalendarEdit { calendar_id, name, color } => {
            Some((Some(calendar_id.clone()), name.clone(), color.clone()))
        }
        _ => None,
    };

    let Some((calendar_id_opt, name, color)) = dialog_data else {
        return;
    };

    // Close dialog first
    DialogManager::close(&mut app.active_dialog);

    let name = name.trim();
    if name.is_empty() {
        warn!("handle_confirm_calendar_dialog: Empty name, ignoring");
        return;
    }

    match calendar_id_opt {
        None => {
            // Create mode
            debug!("handle_confirm_calendar_dialog: Creating calendar '{}'", name);

            match CalendarHandler::create(
                &mut app.calendar_manager,
                NewCalendarData {
                    name: name.to_string(),
                    color,
                },
            ) {
                Ok(id) => {
                    info!("Calendar '{}' created with id '{}'", name, id);
                    // Select the new calendar
                    app.selected_calendar_id = Some(id);
                    app.update_selected_calendar_color();
                }
                Err(e) => {
                    error!("Failed to create calendar: {}", e);
                }
            }
        }
        Some(calendar_id) => {
            // Edit mode
            debug!("handle_confirm_calendar_dialog: Updating calendar '{}'", calendar_id);

            match CalendarHandler::update(
                &mut app.calendar_manager,
                &calendar_id,
                UpdateCalendarData {
                    name: Some(name.to_string()),
                    color: Some(color),
                    enabled: None,
                },
            ) {
                Ok(()) => {
                    info!("Calendar '{}' updated", calendar_id);
                    // Refresh events to update colors
                    app.refresh_cached_events();
                    // Update selected calendar color if this was the selected calendar
                    app.update_selected_calendar_color();
                }
                Err(e) => {
                    error!("Failed to update calendar '{}': {}", calendar_id, e);
                }
            }
        }
    }
}

/// Open the delete calendar confirmation dialog for the selected calendar
pub fn handle_delete_selected_calendar(app: &mut CosmicCalendar) {
    let Some(ref calendar_id) = app.selected_calendar_id else {
        debug!("handle_delete_selected_calendar: No calendar selected");
        return;
    };
    handle_request_delete_calendar(app, calendar_id.clone());
}

/// Open the delete calendar confirmation dialog for a specific calendar
pub fn handle_request_delete_calendar(app: &mut CosmicCalendar, calendar_id: String) {
    debug!("handle_request_delete_calendar: Requesting delete for '{}'", calendar_id);

    // Get calendar info using the handler
    let calendar_name = match CalendarHandler::get_info(&app.calendar_manager, &calendar_id) {
        Ok((name, _, _)) => name,
        Err(_) => calendar_id.clone(),
    };

    DialogManager::open(
        &mut app.active_dialog,
        ActiveDialog::CalendarDelete {
            calendar_id,
            calendar_name,
        },
    );
}

/// Confirm and delete the calendar
pub fn handle_confirm_delete_calendar(app: &mut CosmicCalendar) {
    // Extract data from active_dialog before closing
    let calendar_id = match &app.active_dialog {
        ActiveDialog::CalendarDelete { calendar_id, .. } => calendar_id.clone(),
        _ => return,
    };

    // Close dialog first
    DialogManager::close(&mut app.active_dialog);

    debug!("handle_confirm_delete_calendar: Deleting '{}'", calendar_id);

    match CalendarHandler::delete(&mut app.calendar_manager, &calendar_id) {
        Ok(()) => {
            info!("Calendar '{}' deleted", calendar_id);

            // If we deleted the selected calendar, select another one
            if app.selected_calendar_id.as_ref() == Some(&calendar_id) {
                app.selected_calendar_id = CalendarHandler::get_first_calendar_id(&app.calendar_manager);
                app.update_selected_calendar_color();
            }

            // Refresh events in case any events from the deleted calendar were displayed
            app.refresh_cached_events();
        }
        Err(e) => {
            error!("Failed to delete calendar '{}': {}", calendar_id, e);
        }
    }
}
