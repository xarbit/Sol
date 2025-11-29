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

use chrono::NaiveDate;
use cosmic::app::Task;

use crate::app::CosmicCalendar;
use crate::message::Message;

// Re-export handlers for use in this module
use calendar::{
    handle_change_calendar_color, handle_confirm_calendar_dialog, handle_confirm_delete_calendar,
    handle_delete_selected_calendar, handle_open_calendar_dialog_create,
    handle_open_calendar_dialog_edit, handle_request_delete_calendar, handle_toggle_calendar,
};
use event::{
    handle_cancel_quick_event, handle_commit_quick_event, handle_delete_event,
    handle_quick_event_text_changed, handle_start_quick_event,
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
        // === View Navigation ===
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
            app.settings.show_week_numbers = !app.settings.show_week_numbers;
            // Save settings to persist the change
            app.settings.save().ok();
        }

        // === Calendar Management ===
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

        // === Event Management ===
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

        // === Mini Calendar ===
        Message::MiniCalendarPrevMonth => {
            app.navigate_mini_calendar_previous();
        }
        Message::MiniCalendarNextMonth => {
            app.navigate_mini_calendar_next();
        }

        // === Menu Actions ===
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
    }

    Task::none()
}
