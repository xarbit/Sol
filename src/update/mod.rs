//! Message handling and state updates
//!
//! This module contains the main message handler and delegates to specialized
//! submodules for different categories of messages:
//!
//! - `navigation`: View navigation (previous/next period, view changes)
//! - `calendar`: Calendar management (create, edit, delete, toggle, color)
//! - `event`: Event management (quick events, create, delete)

mod calendar;
mod event;
mod navigation;

use chrono::{NaiveDate, Timelike};
use cosmic::app::Task;
use log::{debug, info, warn};

use crate::app::CosmicCalendar;
use crate::dialogs::DialogManager;
use crate::message::Message;
use crate::services::SettingsHandler;

// Re-export handlers for use in this module
use calendar::{
    handle_change_calendar_color, handle_confirm_calendar_dialog, handle_confirm_delete_calendar,
    handle_delete_selected_calendar, handle_open_calendar_dialog_create,
    handle_open_calendar_dialog_edit, handle_request_delete_calendar, handle_toggle_calendar,
};
use event::{
    handle_cancel_event_dialog, handle_cancel_quick_event, handle_commit_quick_event,
    handle_confirm_event_dialog, handle_delete_event, handle_open_edit_event_dialog,
    handle_open_new_event_dialog, handle_quick_event_text_changed, handle_start_quick_event,
};
use navigation::{handle_next_period, handle_previous_period};

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
        // === Dialog Management (Centralized) ===
        Message::Dialog(action) => {
            debug!("Message::Dialog: {:?}", action);
            // Handle dialog actions through DialogManager
            DialogManager::handle_action(&mut app.active_dialog, action);
        }
        Message::CloseDialog => {
            debug!("Message::CloseDialog: Closing dialogs");
            // Close all legacy dialog fields
            #[allow(deprecated)]
            {
                app.event_dialog = None;
                app.calendar_dialog = None;
                app.delete_calendar_dialog = None;
                app.color_picker_open = None;
            }
            // For quick events: only dismiss if empty (focus loss behavior)
            // For other dialogs: close unconditionally
            if app.active_dialog.is_quick_event() {
                DialogManager::dismiss_empty_quick_event(&mut app.active_dialog);
            } else {
                DialogManager::close(&mut app.active_dialog);
            }
        }

        // === View Navigation ===
        Message::ChangeView(view) => {
            // Dismiss empty quick event when changing views (focus loss)
            DialogManager::dismiss_empty_quick_event(&mut app.active_dialog);
            // When changing views, sync views to the selected_date so the new view
            // shows the period containing the anchor date
            app.current_view = view;
            app.sync_views_to_selected_date();
        }
        Message::PreviousPeriod => {
            // Dismiss empty quick event when navigating (focus loss)
            DialogManager::dismiss_empty_quick_event(&mut app.active_dialog);
            handle_previous_period(app);
        }
        Message::NextPeriod => {
            // Dismiss empty quick event when navigating (focus loss)
            DialogManager::dismiss_empty_quick_event(&mut app.active_dialog);
            handle_next_period(app);
        }
        Message::Today => {
            // Dismiss empty quick event when navigating to today (focus loss)
            DialogManager::dismiss_empty_quick_event(&mut app.active_dialog);
            // Today button navigates to today in all views
            app.navigate_to_today();
        }
        Message::SelectDay(year, month, day) => {
            // Dismiss empty quick event when clicking on a day (focus loss)
            DialogManager::dismiss_empty_quick_event(&mut app.active_dialog);
            // Set the selected date - this syncs all views automatically
            if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                app.set_selected_date(date);
            }
        }
        Message::SelectDayNoNavigate(date) => {
            // Dismiss empty quick event when clicking on a day (focus loss)
            DialogManager::dismiss_empty_quick_event(&mut app.active_dialog);
            // Set selected date without navigating (for adjacent month days)
            // Only updates the selected_date, doesn't sync views to that date's month
            app.selected_date = date;
        }

        // === UI State ===
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
            debug!("Message::ToggleWeekNumbers");
            if let Err(e) = SettingsHandler::toggle_week_numbers(&mut app.settings) {
                log::error!("Failed to toggle week numbers: {}", e);
            }
        }

        // === Calendar Management ===
        Message::ToggleCalendar(id) => {
            // Close color picker when interacting with other elements
            #[allow(deprecated)]
            { app.color_picker_open = None; }
            handle_toggle_calendar(app, id);
        }
        Message::SelectCalendar(id) => {
            // Close color picker when selecting a different calendar
            #[allow(deprecated)]
            { app.color_picker_open = None; }
            app.selected_calendar_id = Some(id);
            app.update_selected_calendar_color();
        }
        Message::ToggleColorPicker(id) => {
            // Toggle: if already open for this calendar, close it; otherwise open it
            #[allow(deprecated)]
            if app.color_picker_open.as_ref() == Some(&id) {
                app.color_picker_open = None;
            } else {
                app.color_picker_open = Some(id);
            }
        }
        Message::CloseColorPicker => {
            #[allow(deprecated)]
            { app.color_picker_open = None; }
        }
        Message::ChangeCalendarColor(id, color) => {
            handle_change_calendar_color(app, id, color);
        }
        Message::OpenNewCalendarDialog => {
            #[allow(deprecated)]
            { app.color_picker_open = None; }
            handle_open_calendar_dialog_create(app);
        }
        Message::OpenEditCalendarDialog(id) => {
            #[allow(deprecated)]
            { app.color_picker_open = None; }
            handle_open_calendar_dialog_edit(app, id);
        }
        Message::EditCalendarByIndex(index) => {
            #[allow(deprecated)]
            { app.color_picker_open = None; }
            if let Some(calendar) = app.calendar_manager.sources().get(index) {
                let id = calendar.info().id.clone();
                handle_open_calendar_dialog_edit(app, id);
            }
        }
        Message::CalendarDialogNameChanged(name) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.calendar_dialog {
                dialog.name = name;
            }
        }
        Message::CalendarDialogColorChanged(color) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.calendar_dialog {
                dialog.color = color;
            }
        }
        Message::ConfirmCalendarDialog => {
            handle_confirm_calendar_dialog(app);
        }
        Message::CancelCalendarDialog => {
            #[allow(deprecated)]
            { app.calendar_dialog = None; }
        }
        Message::DeleteSelectedCalendar => {
            #[allow(deprecated)]
            { app.color_picker_open = None; }
            handle_delete_selected_calendar(app);
        }
        Message::RequestDeleteCalendar(id) => {
            #[allow(deprecated)]
            { app.color_picker_open = None; }
            handle_request_delete_calendar(app, id);
        }
        Message::SelectCalendarByIndex(index) => {
            #[allow(deprecated)]
            { app.color_picker_open = None; }
            if let Some(calendar) = app.calendar_manager.sources().get(index) {
                let id = calendar.info().id.clone();
                app.selected_calendar_id = Some(id);
                app.update_selected_calendar_color();
            }
        }
        Message::DeleteCalendarByIndex(index) => {
            #[allow(deprecated)]
            { app.color_picker_open = None; }
            if let Some(calendar) = app.calendar_manager.sources().get(index) {
                let id = calendar.info().id.clone();
                handle_request_delete_calendar(app, id);
            }
        }
        Message::ConfirmDeleteCalendar => {
            handle_confirm_delete_calendar(app);
        }
        Message::CancelDeleteCalendar => {
            #[allow(deprecated)]
            { app.delete_calendar_dialog = None; }
        }

        // === Event Management - Quick Events ===
        Message::StartQuickEvent(date) => {
            handle_start_quick_event(app, date);
        }
        Message::QuickEventTextChanged(text) => {
            handle_quick_event_text_changed(app, text);
        }
        Message::CommitQuickEvent => {
            handle_commit_quick_event(app);
        }
        Message::CancelQuickEvent => {
            handle_cancel_quick_event(app);
        }
        Message::DeleteEvent(uid) => {
            handle_delete_event(app, uid);
        }

        // === Event Management - Event Dialog ===
        Message::OpenNewEventDialog => {
            handle_open_new_event_dialog(app);
        }
        Message::OpenEditEventDialog(uid) => {
            handle_open_edit_event_dialog(app, uid);
        }
        Message::EventDialogToggleEdit(field, editing) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.editing_field = if editing { Some(field) } else { None };
            }
        }
        Message::EventDialogTitleChanged(title) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.title = title;
            }
        }
        Message::EventDialogLocationChanged(location) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.location = location;
            }
        }
        Message::EventDialogAllDayToggled(all_day) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.all_day = all_day;
            }
        }
        Message::EventDialogStartDateInputChanged(input) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.start_date_input = input.clone();
                // Try to parse the date
                if let Ok(date) = chrono::NaiveDate::parse_from_str(&input, "%Y-%m-%d") {
                    dialog.start_date = date;
                    // If end date is before start, adjust it
                    if dialog.end_date < date {
                        dialog.end_date = date;
                        dialog.end_date_input = date.format("%Y-%m-%d").to_string();
                    }
                }
            }
        }
        Message::EventDialogStartDateChanged(date) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.start_date = date;
                dialog.start_date_input = date.format("%Y-%m-%d").to_string();
                dialog.start_date_calendar.set_selected_visible(date);
                dialog.start_date_picker_open = false; // Close picker after selection
                // If end date is before start, adjust it
                if dialog.end_date < date {
                    dialog.end_date = date;
                    dialog.end_date_input = date.format("%Y-%m-%d").to_string();
                    dialog.end_date_calendar.set_selected_visible(date);
                }
            }
        }
        Message::EventDialogToggleStartDatePicker => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.start_date_picker_open = !dialog.start_date_picker_open;
                dialog.end_date_picker_open = false; // Close the other picker
            }
        }
        Message::EventDialogStartDateCalendarPrev => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.start_date_calendar.show_prev_month();
            }
        }
        Message::EventDialogStartDateCalendarNext => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.start_date_calendar.show_next_month();
            }
        }
        Message::EventDialogToggleStartTimePicker => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.start_time_picker_open = !dialog.start_time_picker_open;
                dialog.end_time_picker_open = false;
                dialog.start_date_picker_open = false;
                dialog.end_date_picker_open = false;
            }
        }
        Message::EventDialogStartTimeHourChanged(hour) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                let current = dialog.start_time.unwrap_or_else(|| chrono::NaiveTime::from_hms_opt(9, 0, 0).unwrap());
                if let Some(new_time) = chrono::NaiveTime::from_hms_opt(hour, current.minute(), 0) {
                    dialog.start_time = Some(new_time);
                    dialog.start_time_input = new_time.format("%H:%M").to_string();
                }
            }
        }
        Message::EventDialogStartTimeMinuteChanged(minute) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                let current = dialog.start_time.unwrap_or_else(|| chrono::NaiveTime::from_hms_opt(9, 0, 0).unwrap());
                if let Some(new_time) = chrono::NaiveTime::from_hms_opt(current.hour(), minute, 0) {
                    dialog.start_time = Some(new_time);
                    dialog.start_time_input = new_time.format("%H:%M").to_string();
                }
            }
        }
        Message::EventDialogEndDateInputChanged(input) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.end_date_input = input.clone();
                // Try to parse the date
                if let Ok(date) = chrono::NaiveDate::parse_from_str(&input, "%Y-%m-%d") {
                    dialog.end_date = date;
                }
            }
        }
        Message::EventDialogEndDateChanged(date) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.end_date = date;
                dialog.end_date_input = date.format("%Y-%m-%d").to_string();
                dialog.end_date_calendar.set_selected_visible(date);
                dialog.end_date_picker_open = false; // Close picker after selection
            }
        }
        Message::EventDialogToggleEndDatePicker => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.end_date_picker_open = !dialog.end_date_picker_open;
                dialog.start_date_picker_open = false; // Close the other picker
            }
        }
        Message::EventDialogEndDateCalendarPrev => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.end_date_calendar.show_prev_month();
            }
        }
        Message::EventDialogEndDateCalendarNext => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.end_date_calendar.show_next_month();
            }
        }
        Message::EventDialogToggleEndTimePicker => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.end_time_picker_open = !dialog.end_time_picker_open;
                dialog.start_time_picker_open = false;
                dialog.start_date_picker_open = false;
                dialog.end_date_picker_open = false;
            }
        }
        Message::EventDialogEndTimeHourChanged(hour) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                let current = dialog.end_time.unwrap_or_else(|| chrono::NaiveTime::from_hms_opt(10, 0, 0).unwrap());
                if let Some(new_time) = chrono::NaiveTime::from_hms_opt(hour, current.minute(), 0) {
                    dialog.end_time = Some(new_time);
                    dialog.end_time_input = new_time.format("%H:%M").to_string();
                }
            }
        }
        Message::EventDialogEndTimeMinuteChanged(minute) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                let current = dialog.end_time.unwrap_or_else(|| chrono::NaiveTime::from_hms_opt(10, 0, 0).unwrap());
                if let Some(new_time) = chrono::NaiveTime::from_hms_opt(current.hour(), minute, 0) {
                    dialog.end_time = Some(new_time);
                    dialog.end_time_input = new_time.format("%H:%M").to_string();
                }
            }
        }
        Message::EventDialogTravelTimeChanged(travel_time) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.travel_time = travel_time;
            }
        }
        Message::EventDialogRepeatChanged(repeat) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.repeat = repeat;
            }
        }
        Message::EventDialogCalendarChanged(calendar_id) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.calendar_id = calendar_id;
            }
        }
        Message::EventDialogInviteeInputChanged(input) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.invitee_input = input;
            }
        }
        Message::EventDialogAddInvitee => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                let email = dialog.invitee_input.trim().to_string();
                if !email.is_empty() && !dialog.invitees.contains(&email) {
                    dialog.invitees.push(email);
                    dialog.invitee_input.clear();
                }
            }
        }
        Message::EventDialogRemoveInvitee(index) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                if index < dialog.invitees.len() {
                    dialog.invitees.remove(index);
                }
            }
        }
        Message::EventDialogAlertChanged(alert) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.alert = alert;
            }
        }
        Message::EventDialogAlertSecondChanged(alert) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.alert_second = alert;
            }
        }
        Message::EventDialogAddAttachment(path) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                if !dialog.attachments.contains(&path) {
                    dialog.attachments.push(path);
                }
            }
        }
        Message::EventDialogRemoveAttachment(index) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                if index < dialog.attachments.len() {
                    dialog.attachments.remove(index);
                }
            }
        }
        Message::EventDialogUrlChanged(url) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.url = url;
            }
        }
        Message::EventDialogNotesAction(action) => {
            #[allow(deprecated)]
            if let Some(ref mut dialog) = app.event_dialog {
                dialog.notes_content.perform(action);
            }
        }
        Message::ConfirmEventDialog => {
            handle_confirm_event_dialog(app);
        }
        Message::CancelEventDialog => {
            handle_cancel_event_dialog(app);
        }

        // === Mini Calendar ===
        Message::MiniCalendarPrevMonth => {
            app.navigate_mini_calendar_previous();
        }
        Message::MiniCalendarNextMonth => {
            app.navigate_mini_calendar_next();
        }

        // === Menu Actions ===
        Message::NewEvent => {
            handle_open_new_event_dialog(app);
        }
        Message::ImportICal => {
            // TODO: Open file picker for iCal import
            info!("Message::ImportICal: Import iCal requested (not yet implemented)");
        }
        Message::ExportICal => {
            // TODO: Open file picker for iCal export
            info!("Message::ExportICal: Export iCal requested (not yet implemented)");
        }
        Message::Settings => {
            // TODO: Open settings dialog
            info!("Message::Settings: Settings requested (not yet implemented)");
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
    }

    Task::none()
}
