use crate::app::CosmicCalendar;
use crate::message::Message;
use crate::views::CalendarView;

/// Handle all application messages and update state
pub fn handle_message(app: &mut CosmicCalendar, message: Message) {
    match message {
        Message::ChangeView(view) => {
            app.current_view = view;
        }
        Message::PreviousPeriod => {
            handle_previous_period(app);
        }
        Message::NextPeriod => {
            handle_next_period(app);
        }
        Message::Today => {
            app.navigate_to_today();
        }
        Message::SelectDay(day) => {
            app.selected_day = Some(day);
        }
        Message::ToggleSidebar => {
            app.show_sidebar = !app.show_sidebar;
        }
        Message::ToggleSearch => {
            app.show_search = !app.show_search;
        }
        Message::ToggleCalendar(id) => {
            handle_toggle_calendar(app, id);
        }
        Message::OpenColorPicker(id) => {
            app.color_picker_open = Some(id);
        }
        Message::ChangeCalendarColor(id, color) => {
            handle_change_calendar_color(app, id, color);
        }
        Message::CloseColorPicker => {
            app.color_picker_open = None;
        }
        Message::MiniCalendarPrevMonth => {
            app.navigate_to_previous_month();
        }
        Message::MiniCalendarNextMonth => {
            app.navigate_to_next_month();
        }
        Message::NewEvent => {
            // TODO: Open new event dialog
            println!("New Event requested");
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
    }
}

/// Handle previous period navigation based on current view
fn handle_previous_period(app: &mut CosmicCalendar) {
    match app.current_view {
        CalendarView::Month => app.navigate_to_previous_month(),
        CalendarView::Week => {
            // Week navigation logic
        }
        CalendarView::Day => {
            // Day navigation logic
        }
    }
}

/// Handle next period navigation based on current view
fn handle_next_period(app: &mut CosmicCalendar) {
    match app.current_view {
        CalendarView::Month => app.navigate_to_next_month(),
        CalendarView::Week => {
            // Week navigation logic
        }
        CalendarView::Day => {
            // Day navigation logic
        }
    }
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
}
