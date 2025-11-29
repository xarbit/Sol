//! Calendar management handlers (create, edit, delete, toggle, color)

use crate::app::{CalendarDialogMode, CalendarDialogState, CosmicCalendar, DeleteCalendarDialogState};
use crate::components::color_picker::CALENDAR_COLORS;

/// Toggle a calendar's enabled state and save configuration
pub fn handle_toggle_calendar(app: &mut CosmicCalendar, id: String) {
    if let Some(calendar) = app
        .calendar_manager
        .sources_mut()
        .iter_mut()
        .find(|c| c.info().id == id)
    {
        calendar.set_enabled(!calendar.is_enabled());
    }
    // Save configuration after toggle
    app.calendar_manager.save_config().ok();
    // Refresh events to show/hide events from toggled calendar
    app.refresh_cached_events();
}

/// Change a calendar's color and save configuration
pub fn handle_change_calendar_color(app: &mut CosmicCalendar, id: String, color: String) {
    if let Some(calendar) = app
        .calendar_manager
        .sources_mut()
        .iter_mut()
        .find(|c| c.info().id == id)
    {
        calendar.info_mut().color = color;
    }
    // Save configuration after color change
    app.calendar_manager.save_config().ok();
    // Close the color picker after selection
    app.color_picker_open = None;
    // Refresh events to update event colors in views
    app.refresh_cached_events();
    // Also update selected calendar color if this was the selected calendar
    app.update_selected_calendar_color();
}

/// Open the calendar dialog in Create mode
pub fn handle_open_calendar_dialog_create(app: &mut CosmicCalendar) {
    // Default to the first color in the palette
    let default_color = CALENDAR_COLORS
        .first()
        .map(|(hex, _)| hex.to_string())
        .unwrap_or_else(|| "#3B82F6".to_string());

    app.calendar_dialog = Some(CalendarDialogState {
        mode: CalendarDialogMode::Create,
        name: String::new(),
        color: default_color,
    });
}

/// Open the calendar dialog in Edit mode for a specific calendar
pub fn handle_open_calendar_dialog_edit(app: &mut CosmicCalendar, calendar_id: String) {
    // Find the calendar to get its current values
    let Some(calendar) = app
        .calendar_manager
        .sources()
        .iter()
        .find(|c| c.info().id == calendar_id)
    else {
        return;
    };

    let info = calendar.info();
    app.calendar_dialog = Some(CalendarDialogState {
        mode: CalendarDialogMode::Edit {
            calendar_id: info.id.clone(),
        },
        name: info.name.clone(),
        color: info.color.clone(),
    });
}

/// Confirm the calendar dialog (Create or Edit)
pub fn handle_confirm_calendar_dialog(app: &mut CosmicCalendar) {
    let Some(dialog) = app.calendar_dialog.take() else {
        return;
    };

    let name = dialog.name.trim();
    if name.is_empty() {
        // Don't allow empty name
        return;
    }

    match dialog.mode {
        CalendarDialogMode::Create => {
            // Generate a unique ID based on the name
            let id = name
                .to_lowercase()
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == ' ')
                .map(|c| if c == ' ' { '-' } else { c })
                .collect::<String>();

            // Make sure ID is unique by appending a number if needed
            let mut unique_id = id.clone();
            let mut counter = 1;
            while app
                .calendar_manager
                .sources()
                .iter()
                .any(|c| c.info().id == unique_id)
            {
                unique_id = format!("{}-{}", id, counter);
                counter += 1;
            }

            // Add the calendar
            app.calendar_manager.add_local_calendar(
                unique_id.clone(),
                name.to_string(),
                dialog.color,
            );

            // Select the new calendar
            app.selected_calendar_id = Some(unique_id);
            app.update_selected_calendar_color();
        }
        CalendarDialogMode::Edit { calendar_id } => {
            // Update the existing calendar
            if let Some(calendar) = app
                .calendar_manager
                .sources_mut()
                .iter_mut()
                .find(|c| c.info().id == calendar_id)
            {
                calendar.info_mut().name = name.to_string();
                calendar.info_mut().color = dialog.color;
            }
            // Save configuration
            app.calendar_manager.save_config().ok();
            // Refresh events to update colors
            app.refresh_cached_events();
            // Update selected calendar color if this was the selected calendar
            app.update_selected_calendar_color();
        }
    }
}

/// Open the delete calendar confirmation dialog for the selected calendar
pub fn handle_delete_selected_calendar(app: &mut CosmicCalendar) {
    let Some(ref calendar_id) = app.selected_calendar_id else {
        return;
    };
    handle_request_delete_calendar(app, calendar_id.clone());
}

/// Open the delete calendar confirmation dialog for a specific calendar
pub fn handle_request_delete_calendar(app: &mut CosmicCalendar, calendar_id: String) {
    // Find the calendar to get its name
    let calendar_name = app
        .calendar_manager
        .sources()
        .iter()
        .find(|c| c.info().id == calendar_id)
        .map(|c| c.info().name.clone())
        .unwrap_or_else(|| calendar_id.clone());

    app.delete_calendar_dialog = Some(DeleteCalendarDialogState {
        calendar_id,
        calendar_name,
    });
}

/// Confirm and delete the calendar
pub fn handle_confirm_delete_calendar(app: &mut CosmicCalendar) {
    let Some(dialog) = app.delete_calendar_dialog.take() else {
        return;
    };

    // Delete the calendar
    app.calendar_manager.delete_calendar(&dialog.calendar_id);

    // If we deleted the selected calendar, select another one
    if app.selected_calendar_id.as_ref() == Some(&dialog.calendar_id) {
        app.selected_calendar_id = app
            .calendar_manager
            .sources()
            .first()
            .map(|c| c.info().id.clone());
        app.update_selected_calendar_color();
    }

    // Refresh events in case any events from the deleted calendar were displayed
    app.refresh_cached_events();
}
