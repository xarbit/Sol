use chrono::{Datelike, NaiveDate, NaiveTime, TimeZone, Utc};
use crate::app::{CosmicCalendar, CalendarDialogState, CalendarDialogMode, DeleteCalendarDialogState};
use crate::caldav::CalendarEvent;
use crate::components::color_picker::CALENDAR_COLORS;
use crate::message::Message;
use crate::views::CalendarView;
use cosmic::app::Task;
use uuid::Uuid;

/// Handle all application messages and update state
pub fn handle_message(app: &mut CosmicCalendar, message: Message) -> Task<Message> {
    // Sync sidebar with condensed state on every update
    let is_condensed = app.core.is_condensed();
    if is_condensed != app.last_condensed {
        app.last_condensed = is_condensed;
        // Auto-collapse sidebar when entering condensed mode, show when leaving
        app.show_sidebar = !is_condensed;
    }

    match message {
        Message::ChangeView(view) => {
            // When changing views, sync views to the selected_date so the new view
            // shows the period containing the anchor date
            app.current_view = view;
            app.sync_views_to_selected_date();
        }
        Message::PreviousPeriod => {
            handle_previous_period(app);
        }
        Message::NextPeriod => {
            handle_next_period(app);
        }
        Message::Today => {
            // Today button navigates to today in all views
            app.navigate_to_today();
        }
        Message::SelectDay(year, month, day) => {
            // Set the selected date - this syncs all views automatically
            if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                app.set_selected_date(date);
            }
        }
        Message::ToggleSidebar => {
            app.show_sidebar = !app.show_sidebar;
        }
        Message::WindowResized => {
            // Sync is handled at start of update(), nothing else needed
        }
        Message::ToggleSearch => {
            app.show_search = !app.show_search;
        }
        Message::ToggleWeekNumbers => {
            app.settings.show_week_numbers = !app.settings.show_week_numbers;
            // Save settings to persist the change
            app.settings.save().ok();
        }
        Message::ToggleCalendar(id) => {
            // Close color picker when interacting with other elements
            app.color_picker_open = None;
            handle_toggle_calendar(app, id);
        }
        Message::SelectCalendar(id) => {
            // Close color picker when selecting a different calendar
            app.color_picker_open = None;
            app.selected_calendar_id = Some(id);
            app.update_selected_calendar_color();
        }
        Message::ToggleColorPicker(id) => {
            // Toggle: if already open for this calendar, close it; otherwise open it
            if app.color_picker_open.as_ref() == Some(&id) {
                app.color_picker_open = None;
            } else {
                app.color_picker_open = Some(id);
            }
        }
        Message::CloseColorPicker => {
            app.color_picker_open = None;
        }
        Message::ChangeCalendarColor(id, color) => {
            handle_change_calendar_color(app, id, color);
        }
        Message::StartQuickEvent(date) => {
            // Start editing a quick event on the specified date
            app.quick_event_editing = Some((date, String::new()));
        }
        Message::QuickEventTextChanged(text) => {
            // Update the text of the quick event being edited
            if let Some((date, _)) = app.quick_event_editing.take() {
                app.quick_event_editing = Some((date, text));
            }
        }
        Message::CommitQuickEvent => {
            handle_commit_quick_event(app);
        }
        Message::CancelQuickEvent => {
            app.quick_event_editing = None;
        }
        Message::DeleteEvent(uid) => {
            handle_delete_event(app, uid);
        }
        Message::MiniCalendarPrevMonth => {
            app.navigate_mini_calendar_previous();
        }
        Message::MiniCalendarNextMonth => {
            app.navigate_mini_calendar_next();
        }
        Message::NewEvent => {
            // TODO: Open new event dialog
            println!("New Event requested");
        }
        Message::ImportICal => {
            // TODO: Open file picker for iCal import
            println!("Import iCal requested");
        }
        Message::ExportICal => {
            // TODO: Open file picker for iCal export
            println!("Export iCal requested");
        }
        Message::Settings => {
            // TODO: Open settings dialog
            println!("Settings requested");
        }
        Message::About => {
            app.core.window.show_context = !app.core.window.show_context;
        }
        Message::LaunchUrl(url) => {
            // Open URL in default browser
            let _ = open::that(&url);
        }
        Message::ToggleContextDrawer => {
            app.core.window.show_context = !app.core.window.show_context;
        }
        Message::Surface(action) => {
            return cosmic::task::message(cosmic::Action::Cosmic(
                cosmic::app::Action::Surface(action),
            ));
        }
        // Calendar management messages
        Message::OpenNewCalendarDialog => {
            app.color_picker_open = None;
            handle_open_calendar_dialog_create(app);
        }
        Message::OpenEditCalendarDialog(id) => {
            app.color_picker_open = None;
            handle_open_calendar_dialog_edit(app, id);
        }
        Message::EditCalendarByIndex(index) => {
            app.color_picker_open = None;
            if let Some(calendar) = app.calendar_manager.sources().get(index) {
                let id = calendar.info().id.clone();
                handle_open_calendar_dialog_edit(app, id);
            }
        }
        Message::CalendarDialogNameChanged(name) => {
            if let Some(ref mut dialog) = app.calendar_dialog {
                dialog.name = name;
            }
        }
        Message::CalendarDialogColorChanged(color) => {
            if let Some(ref mut dialog) = app.calendar_dialog {
                dialog.color = color;
            }
        }
        Message::ConfirmCalendarDialog => {
            handle_confirm_calendar_dialog(app);
        }
        Message::CancelCalendarDialog => {
            app.calendar_dialog = None;
        }
        Message::DeleteSelectedCalendar => {
            app.color_picker_open = None;
            handle_delete_selected_calendar(app);
        }
        Message::RequestDeleteCalendar(id) => {
            app.color_picker_open = None;
            handle_request_delete_calendar(app, id);
        }
        Message::SelectCalendarByIndex(index) => {
            app.color_picker_open = None;
            if let Some(calendar) = app.calendar_manager.sources().get(index) {
                let id = calendar.info().id.clone();
                app.selected_calendar_id = Some(id);
                app.update_selected_calendar_color();
            }
        }
        Message::DeleteCalendarByIndex(index) => {
            app.color_picker_open = None;
            if let Some(calendar) = app.calendar_manager.sources().get(index) {
                let id = calendar.info().id.clone();
                handle_request_delete_calendar(app, id);
            }
        }
        Message::ConfirmDeleteCalendar => {
            handle_confirm_delete_calendar(app);
        }
        Message::CancelDeleteCalendar => {
            app.delete_calendar_dialog = None;
        }
    }

    Task::none()
}

/// Direction for period navigation
enum NavigationDirection {
    Previous,
    Next,
}

/// Handle period navigation (previous or next) based on current view
fn handle_period_navigation(app: &mut CosmicCalendar, direction: NavigationDirection) {
    let multiplier: i32 = match direction {
        NavigationDirection::Previous => -1,
        NavigationDirection::Next => 1,
    };

    let new_date = match app.current_view {
        CalendarView::Year => {
            // Move by one year
            navigate_by_year(app.selected_date, multiplier)
        }
        CalendarView::Month => {
            // Move by one month
            navigate_by_month(app.selected_date, multiplier)
        }
        CalendarView::Week => {
            // Move by one week
            Some(app.selected_date + chrono::Duration::days(7 * multiplier as i64))
        }
        CalendarView::Day => {
            // Move by one day
            Some(app.selected_date + chrono::Duration::days(multiplier as i64))
        }
    };

    if let Some(date) = new_date {
        app.set_selected_date(date);
    }
}

/// Navigate a date by the given number of years, handling edge cases like Feb 29
fn navigate_by_year(date: NaiveDate, years: i32) -> Option<NaiveDate> {
    let new_year = date.year() + years;
    // Try the same day first, then fall back to day 28 for edge cases
    NaiveDate::from_ymd_opt(new_year, date.month(), date.day().min(28))
        .or_else(|| NaiveDate::from_ymd_opt(new_year, date.month(), 28))
}

/// Navigate a date by the given number of months, handling edge cases
fn navigate_by_month(date: NaiveDate, months: i32) -> Option<NaiveDate> {
    let total_months = date.year() * 12 + date.month() as i32 - 1 + months;
    let new_year = total_months / 12;
    let new_month = (total_months % 12 + 1) as u32;

    // Try the same day first, then fall back to day 28 for edge cases
    NaiveDate::from_ymd_opt(new_year, new_month, date.day().min(28))
        .or_else(|| NaiveDate::from_ymd_opt(new_year, new_month, 28))
}

/// Handle previous period navigation
fn handle_previous_period(app: &mut CosmicCalendar) {
    handle_period_navigation(app, NavigationDirection::Previous);
}

/// Handle next period navigation
fn handle_next_period(app: &mut CosmicCalendar) {
    handle_period_navigation(app, NavigationDirection::Next);
}

/// Toggle a calendar's enabled state and save configuration
fn handle_toggle_calendar(app: &mut CosmicCalendar, id: String) {
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
fn handle_change_calendar_color(app: &mut CosmicCalendar, id: String, color: String) {
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

/// Commit the quick event being edited - create a new event in the selected calendar
fn handle_commit_quick_event(app: &mut CosmicCalendar) {
    // Get the event data and clear the editing state
    let Some((date, text)) = app.quick_event_editing.take() else {
        return;
    };

    // Don't create empty events
    let text = text.trim();
    if text.is_empty() {
        return;
    }

    // Get the selected calendar ID
    let Some(calendar_id) = app.selected_calendar_id.clone() else {
        eprintln!("No calendar selected for new event");
        return;
    };

    // Create an all-day event for the selected date
    // Use midnight UTC for start, end of day for end
    let start_time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
    let end_time = NaiveTime::from_hms_opt(23, 59, 59).unwrap();

    let start = Utc.from_utc_datetime(&date.and_time(start_time));
    let end = Utc.from_utc_datetime(&date.and_time(end_time));

    let event = CalendarEvent {
        uid: Uuid::new_v4().to_string(),
        summary: text.to_string(),
        description: None,
        start,
        end,
        location: None,
    };

    // Find the calendar and add the event
    if let Some(calendar) = app
        .calendar_manager
        .sources_mut()
        .iter_mut()
        .find(|c| c.info().id == calendar_id)
    {
        if let Err(e) = calendar.add_event(event) {
            eprintln!("Failed to add event: {}", e);
            return;
        }
        // Sync to persist the event
        if let Err(e) = calendar.sync() {
            eprintln!("Failed to sync calendar: {}", e);
        }
    }

    // Refresh the cached events to show the new event
    app.refresh_cached_events();
}

/// Delete an event by its UID from all calendars
fn handle_delete_event(app: &mut CosmicCalendar, uid: String) {
    for calendar in app.calendar_manager.sources_mut().iter_mut() {
        if calendar.delete_event(&uid).is_ok() {
            // Sync to persist the deletion
            let _ = calendar.sync();
            break;
        }
    }
    // Refresh cached events to reflect deletion
    app.refresh_cached_events();
}

/// Open the calendar dialog in Create mode
fn handle_open_calendar_dialog_create(app: &mut CosmicCalendar) {
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
fn handle_open_calendar_dialog_edit(app: &mut CosmicCalendar, calendar_id: String) {
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
fn handle_confirm_calendar_dialog(app: &mut CosmicCalendar) {
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
            while app.calendar_manager.sources().iter().any(|c| c.info().id == unique_id) {
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
fn handle_delete_selected_calendar(app: &mut CosmicCalendar) {
    let Some(ref calendar_id) = app.selected_calendar_id else {
        return;
    };
    handle_request_delete_calendar(app, calendar_id.clone());
}

/// Open the delete calendar confirmation dialog for a specific calendar
fn handle_request_delete_calendar(app: &mut CosmicCalendar, calendar_id: String) {
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
fn handle_confirm_delete_calendar(app: &mut CosmicCalendar) {
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
