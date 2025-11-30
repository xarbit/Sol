//! Message handling and state updates
//!
//! This module contains the main message handler and delegates to specialized
//! submodules for different categories of messages:
//!
//! - `navigation`: View navigation (previous/next period, view changes)
//! - `calendar`: Calendar management (create, edit, delete, toggle, color)
//! - `event`: Event management (quick events, create, delete)
//! - `selection`: Drag selection for multi-day event creation

mod calendar;
mod event;
mod navigation;
mod selection;

use chrono::{NaiveDate, Timelike};
use cosmic::app::Task;
use cosmic::iced::widget::scrollable;
use log::{debug, info};

use crate::app::CosmicCalendar;
use crate::components::quick_event_input_id;
use crate::dialogs::{ActiveDialog, DialogManager};
use crate::message::Message;
use crate::services::SettingsHandler;
use crate::views::{week_time_grid_id, CalendarView};
use crate::ui_constants::HOUR_ROW_HEIGHT;
use cosmic::iced_widget::text_input;

/// Helper to dismiss empty quick events on focus-loss actions (navigation, day selection)
/// This centralizes the pattern of clearing transient UI state when the user navigates away
#[inline]
fn dismiss_on_focus_loss(app: &mut CosmicCalendar) {
    DialogManager::dismiss_empty_quick_event(&mut app.active_dialog);
}

/// Focus the quick event input field
/// Returns a Task that focuses the text input for immediate typing
#[inline]
fn focus_quick_event_input() -> Task<Message> {
    text_input::focus(quick_event_input_id())
}

/// Scroll the week view time grid to the current time
/// Returns a Task that scrolls to show the current hour (offset by 1-2 hours to show some past)
#[inline]
fn scroll_week_to_current_time() -> Task<Message> {
    let now = chrono::Local::now();
    let current_hour = now.hour();

    // Scroll to show current time with some context (show 1-2 hours before current time)
    // Each hour row is HOUR_ROW_HEIGHT pixels tall
    let scroll_to_hour = current_hour.saturating_sub(1) as f32;
    let scroll_offset = scroll_to_hour * HOUR_ROW_HEIGHT;

    // Use scroll_to with AbsoluteOffset for vertical scrolling
    scrollable::scroll_to(
        week_time_grid_id(),
        scrollable::AbsoluteOffset { x: 0.0, y: scroll_offset },
    )
}

/// Close the legacy event dialog field
/// This helper is kept because text_editor::Content doesn't implement Clone
#[allow(deprecated)]
#[inline]
fn close_legacy_event_dialog(app: &mut CosmicCalendar) {
    app.event_dialog = None;
}

// Re-export handlers for use in this module
use calendar::{
    handle_change_calendar_color, handle_confirm_calendar_dialog, handle_confirm_delete_calendar,
    handle_delete_selected_calendar, handle_open_calendar_dialog_create,
    handle_open_calendar_dialog_edit, handle_request_delete_calendar, handle_toggle_calendar,
};
use event::{
    handle_cancel_event_dialog, handle_cancel_quick_event, handle_commit_quick_event,
    handle_confirm_event_dialog, handle_delete_event, handle_drag_event_cancel,
    handle_drag_event_end, handle_drag_event_start, handle_drag_event_update,
    handle_open_edit_event_dialog, handle_open_new_event_dialog, handle_quick_event_text_changed,
    handle_select_event, handle_start_quick_event, handle_start_quick_timed_event,
};
use navigation::{handle_next_period, handle_previous_period};
use selection::{
    handle_selection_cancel, handle_selection_end, handle_selection_start, handle_selection_update,
    handle_time_selection_start, handle_time_selection_update, handle_time_selection_end,
};

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
            // Close legacy event dialog
            close_legacy_event_dialog(app);
            // For quick events: only dismiss if empty (focus loss behavior)
            // For other dialogs: close unconditionally
            if app.active_dialog.is_quick_event() {
                dismiss_on_focus_loss(app);
            } else {
                DialogManager::close(&mut app.active_dialog);
            }
        }

        // === View Navigation ===
        // All navigation actions dismiss empty quick events (focus loss behavior)
        Message::ChangeView(view) => {
            dismiss_on_focus_loss(app);
            app.current_view = view;
            app.sync_views_to_selected_date();
            // Auto-scroll to current time when entering week view
            if view == CalendarView::Week {
                return scroll_week_to_current_time();
            }
        }
        Message::PreviousPeriod => {
            dismiss_on_focus_loss(app);
            handle_previous_period(app);
        }
        Message::NextPeriod => {
            dismiss_on_focus_loss(app);
            handle_next_period(app);
        }
        Message::Today => {
            dismiss_on_focus_loss(app);
            app.navigate_to_today();
        }
        Message::SelectDay(year, month, day) => {
            dismiss_on_focus_loss(app);
            if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                app.set_selected_date(date);
            }
        }
        Message::SelectDayNoNavigate(date) => {
            dismiss_on_focus_loss(app);
            app.selected_date = date;
        }

        // === UI State ===
        Message::TimeTick => {
            // Timer tick to update the current time indicator
            // The view will re-render with the new time automatically
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
            debug!("Message::ToggleWeekNumbers");
            if let Err(e) = SettingsHandler::toggle_week_numbers(&mut app.settings) {
                log::error!("Failed to toggle week numbers: {}", e);
            }
        }

        // === Calendar Management ===
        Message::ToggleCalendar(id) => {
            // Close dialogs when interacting with other elements
            DialogManager::close(&mut app.active_dialog);
            handle_toggle_calendar(app, id);
        }
        Message::SelectCalendar(id) => {
            // Close dialogs when selecting a different calendar
            DialogManager::close(&mut app.active_dialog);
            app.selected_calendar_id = Some(id);
            app.update_selected_calendar_color();
        }
        Message::ToggleColorPicker(id) => {
            // Toggle: if already open for this calendar, close it; otherwise open it
            if app.active_dialog.color_picker_calendar_id() == Some(&id) {
                DialogManager::close(&mut app.active_dialog);
            } else {
                DialogManager::open(&mut app.active_dialog, ActiveDialog::ColorPicker { calendar_id: id });
            }
        }
        Message::CloseColorPicker => {
            DialogManager::close(&mut app.active_dialog);
        }
        Message::ChangeCalendarColor(id, color) => {
            handle_change_calendar_color(app, id, color);
        }
        Message::OpenNewCalendarDialog => {
            DialogManager::close(&mut app.active_dialog);
            handle_open_calendar_dialog_create(app);
        }
        Message::OpenEditCalendarDialog(id) => {
            DialogManager::close(&mut app.active_dialog);
            handle_open_calendar_dialog_edit(app, id);
        }
        Message::EditCalendarByIndex(index) => {
            DialogManager::close(&mut app.active_dialog);
            if let Some(calendar) = app.calendar_manager.sources().get(index) {
                let id = calendar.info().id.clone();
                handle_open_calendar_dialog_edit(app, id);
            }
        }
        Message::CalendarDialogNameChanged(name) => {
            // Update calendar dialog name via active_dialog
            match &mut app.active_dialog {
                ActiveDialog::CalendarCreate { name: n, .. }
                | ActiveDialog::CalendarEdit { name: n, .. } => {
                    *n = name;
                }
                _ => {}
            }
        }
        Message::CalendarDialogColorChanged(color) => {
            // Update calendar dialog color via active_dialog
            match &mut app.active_dialog {
                ActiveDialog::CalendarCreate { color: c, .. }
                | ActiveDialog::CalendarEdit { color: c, .. } => {
                    *c = color;
                }
                _ => {}
            }
        }
        Message::ConfirmCalendarDialog => {
            handle_confirm_calendar_dialog(app);
        }
        Message::CancelCalendarDialog => {
            DialogManager::close(&mut app.active_dialog);
        }
        Message::DeleteSelectedCalendar => {
            DialogManager::close(&mut app.active_dialog);
            handle_delete_selected_calendar(app);
        }
        Message::RequestDeleteCalendar(id) => {
            DialogManager::close(&mut app.active_dialog);
            handle_request_delete_calendar(app, id);
        }
        Message::SelectCalendarByIndex(index) => {
            DialogManager::close(&mut app.active_dialog);
            if let Some(calendar) = app.calendar_manager.sources().get(index) {
                let id = calendar.info().id.clone();
                app.selected_calendar_id = Some(id);
                app.update_selected_calendar_color();
            }
        }
        Message::DeleteCalendarByIndex(index) => {
            DialogManager::close(&mut app.active_dialog);
            if let Some(calendar) = app.calendar_manager.sources().get(index) {
                let id = calendar.info().id.clone();
                handle_request_delete_calendar(app, id);
            }
        }
        Message::ConfirmDeleteCalendar => {
            handle_confirm_delete_calendar(app);
        }
        Message::CancelDeleteCalendar => {
            DialogManager::close(&mut app.active_dialog);
        }

        // === Selection - Drag Selection for Multi-Day Events ===
        Message::SelectionStart(date) => {
            // Cancel any empty quick event when starting a selection
            DialogManager::dismiss_empty_quick_event(&mut app.active_dialog);
            handle_selection_start(app, date);
        }
        Message::SelectionUpdate(date) => {
            handle_selection_update(app, date);
        }
        Message::SelectionEnd => {
            handle_selection_end(app);
            // Focus the quick event input if a quick event was started
            if app.active_dialog.is_quick_event() {
                return focus_quick_event_input();
            }
        }
        Message::SelectionCancel => {
            handle_selection_cancel(app);
        }

        // === Time-Based Selection - Drag Selection for Timed Events ===
        Message::TimeSelectionStart(date, time) => {
            // Cancel any empty quick event when starting a selection
            DialogManager::dismiss_empty_quick_event(&mut app.active_dialog);
            handle_time_selection_start(app, date, time);
        }
        Message::TimeSelectionUpdate(date, time) => {
            handle_time_selection_update(app, date, time);
        }
        Message::TimeSelectionEnd => {
            handle_time_selection_end(app);
            // Focus the quick event input if a quick event was started
            if app.active_dialog.is_quick_event() {
                return focus_quick_event_input();
            }
        }

        // === Event Management - Quick Events ===
        Message::StartQuickEvent(date) => {
            handle_start_quick_event(app, date);
            return focus_quick_event_input();
        }
        Message::StartQuickTimedEvent(date, start_time, end_time) => {
            handle_start_quick_timed_event(app, date, start_time, end_time);
            return focus_quick_event_input();
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
        Message::SelectEvent(uid) => {
            handle_select_event(app, uid);
        }

        // === Event Drag-and-Drop ===
        Message::DragEventStart(uid, date, summary, color) => {
            handle_drag_event_start(app, uid, date, summary, color);
        }
        Message::DragEventUpdate(date) => {
            handle_drag_event_update(app, date);
        }
        Message::DragEventCursorMove(x, y) => {
            app.event_drag_state.update_cursor(x, y);
        }
        Message::DragEventEnd => {
            handle_drag_event_end(app);
        }
        Message::DragEventCancel => {
            handle_drag_event_cancel(app);
        }

        // === Event Management - Event Dialog ===
        Message::OpenNewEventDialog => {
            handle_open_new_event_dialog(app);
        }
        Message::OpenEditEventDialog(uid) => {
            // Cancel any drag operation that may have started from the first click of double-click
            app.event_drag_state.cancel();
            // Clear selection since we're opening the edit dialog
            app.selected_event_uid = None;
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
