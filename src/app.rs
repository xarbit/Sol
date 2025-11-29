use crate::cache::CalendarCache;
use crate::calendars::CalendarManager;
use crate::components;
use crate::fl;
use crate::locale::LocalePreferences;
use crate::menu_action::MenuAction;
use crate::message::Message;
use crate::models::{CalendarState, WeekState, DayState, YearState};
use crate::settings::AppSettings;
use crate::views::{self, CalendarView};
use chrono::{Datelike, NaiveDate};
use cosmic::app::Core;
use cosmic::iced::keyboard;
use cosmic::widget::{about, calendar::CalendarModel, menu, text_editor};
use cosmic::widget::menu::Action as _; // Import trait for .message() method
use cosmic::{Application, Element};
use std::collections::HashMap;

const APP_ID: &str = "io.github.xarbit.SolCalendar";

/// Mode for the calendar dialog (Create or Edit)
#[derive(Debug, Clone, PartialEq)]
pub enum CalendarDialogMode {
    /// Creating a new calendar
    Create,
    /// Editing an existing calendar
    Edit {
        /// ID of the calendar being edited
        calendar_id: String,
    },
}

/// State for the calendar dialog (used for both Create and Edit)
#[derive(Debug, Clone)]
pub struct CalendarDialogState {
    /// Dialog mode - Create or Edit
    pub mode: CalendarDialogMode,
    /// Calendar name being entered/edited
    pub name: String,
    /// Selected color for the calendar
    pub color: String,
}

/// State for the delete calendar confirmation dialog
#[derive(Debug, Clone)]
pub struct DeleteCalendarDialogState {
    /// ID of the calendar to delete
    pub calendar_id: String,
    /// Name of the calendar (for display in confirmation)
    pub calendar_name: String,
}

/// Enum for which field is being edited in the event dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventDialogField {
    Title,
    Location,
    Url,
}

/// State for the event dialog (Create or Edit)
pub struct EventDialogState {
    /// Event UID (None for new events, Some for editing)
    pub editing_uid: Option<String>,
    /// Event title/summary
    pub title: String,
    /// Event location
    pub location: String,
    /// Whether this is an all-day event
    pub all_day: bool,
    /// Start date
    pub start_date: chrono::NaiveDate,
    /// Start date input buffer (for editing)
    pub start_date_input: String,
    /// Start time (None for all-day events)
    pub start_time: Option<chrono::NaiveTime>,
    /// Start time input buffer (for editing)
    pub start_time_input: String,
    /// End date
    pub end_date: chrono::NaiveDate,
    /// End date input buffer (for editing)
    pub end_date_input: String,
    /// End time (None for all-day events)
    pub end_time: Option<chrono::NaiveTime>,
    /// End time input buffer (for editing)
    pub end_time_input: String,
    /// Travel time before the event
    pub travel_time: crate::caldav::TravelTime,
    /// Repeat/recurrence settings
    pub repeat: crate::caldav::RepeatFrequency,
    /// Selected calendar ID for the event
    pub calendar_id: String,
    /// Invitees (email addresses)
    pub invitees: Vec<String>,
    /// New invitee being typed (input buffer)
    pub invitee_input: String,
    /// Alert/reminder settings
    pub alert: crate::caldav::AlertTime,
    /// Second alert (optional)
    pub alert_second: Option<crate::caldav::AlertTime>,
    /// File attachments (paths or URLs)
    pub attachments: Vec<String>,
    /// URL associated with the event
    pub url: String,
    /// Notes/description content (for text_editor widget)
    pub notes_content: text_editor::Content,
    /// Which field is currently being edited (None = no field in edit mode)
    pub editing_field: Option<EventDialogField>,
    /// Whether the start date calendar picker is open
    pub start_date_picker_open: bool,
    /// Calendar model for start date picker
    pub start_date_calendar: CalendarModel,
    /// Whether the end date calendar picker is open
    pub end_date_picker_open: bool,
    /// Calendar model for end date picker
    pub end_date_calendar: CalendarModel,
    /// Whether the start time picker is open
    pub start_time_picker_open: bool,
    /// Whether the end time picker is open
    pub end_time_picker_open: bool,
}

/// Main application state
pub struct CosmicCalendar {
    pub core: Core,
    pub current_view: CalendarView,
    /// The anchor date shared by all views - when you select a date or switch views,
    /// all views sync to show the period containing this date
    pub selected_date: NaiveDate,
    pub calendar_manager: CalendarManager,
    pub show_sidebar: bool,
    /// Track previous condensed state to detect changes and sync sidebar
    pub last_condensed: bool,
    pub show_search: bool,
    pub color_picker_open: Option<String>,
    pub cache: CalendarCache,
    pub week_state: WeekState,
    pub day_state: DayState,
    pub year_state: YearState,
    /// Mini calendar state - independent from main view for browsing
    pub mini_calendar_state: CalendarState,
    pub locale: LocalePreferences,
    pub settings: AppSettings,
    pub about: about::About,
    pub key_binds: HashMap<menu::KeyBind, MenuAction>,
    /// The currently selected calendar for new events (calendar id)
    pub selected_calendar_id: Option<String>,
    /// Quick event being edited: (date, event_text) - None when not editing
    pub quick_event_editing: Option<(NaiveDate, String)>,
    /// Cached events for current month view, grouped by date (supports adjacent months)
    pub cached_month_events: std::collections::HashMap<chrono::NaiveDate, Vec<crate::components::DisplayEvent>>,
    /// Color of the selected calendar (cached for quick event input)
    pub selected_calendar_color: String,
    /// Calendar dialog state (for Create/Edit) - None when dialog is closed
    pub calendar_dialog: Option<CalendarDialogState>,
    /// Delete calendar confirmation dialog state - None when dialog is closed
    pub delete_calendar_dialog: Option<DeleteCalendarDialogState>,
    /// Event dialog state (for Create/Edit) - None when dialog is closed
    pub event_dialog: Option<EventDialogState>,
}

impl CosmicCalendar {
    /// Initialize the application with default calendars
    fn initialize_app(core: Core) -> Self {
        let today = chrono::Local::now().date_naive();

        let year = today.year();
        let month = today.month();

        // Create cache and pre-cache surrounding months
        let mut cache = CalendarCache::new(year, month);
        cache.precache_surrounding(1, 2);

        // Initialize calendar manager with default calendars
        let calendar_manager = CalendarManager::with_defaults();

        // Select the first calendar by default for new events
        let selected_calendar_id = calendar_manager
            .sources()
            .first()
            .map(|c| c.info().id.clone());

        // Load application settings
        let settings = AppSettings::load().unwrap_or_default();

        // Create About dialog
        let about = about::About::default()
            .name(fl!("app-title"))
            .version(env!("CARGO_PKG_VERSION"))
            .license(fl!("about-license"))
            .links([(fl!("about-repository"), "https://github.com/xarbit/sol")]);

        // Detect system locale preferences
        let locale = LocalePreferences::detect_from_system();

        // Initialize keyboard shortcuts from centralized module
        let key_binds = crate::keyboard::init_key_binds();

        // Mini calendar starts showing the current month
        let mini_calendar_state = CalendarState::new(year, month);

        // Get selected calendar color (default to first calendar's color)
        let selected_calendar_color = calendar_manager
            .sources()
            .first()
            .map(|c| c.info().color.clone())
            .unwrap_or_else(|| "#3B82F6".to_string());

        // Cache events for current month
        let cached_month_events = calendar_manager.get_display_events_for_month(year, month);

        CosmicCalendar {
            core,
            current_view: CalendarView::Month,
            selected_date: today,
            calendar_manager,
            show_sidebar: true,
            last_condensed: false, // Will be synced on first render
            show_search: false,
            color_picker_open: None,
            cache,
            week_state: WeekState::current_with_first_day(locale.first_day_of_week, &locale),
            day_state: DayState::current(&locale),
            year_state: YearState::current(),
            mini_calendar_state,
            locale,
            settings,
            about,
            key_binds,
            selected_calendar_id,
            quick_event_editing: None,
            cached_month_events,
            selected_calendar_color,
            calendar_dialog: None,
            delete_calendar_dialog: None,
            event_dialog: None,
        }
    }

    /// Sync all views to show the period containing the selected_date
    pub fn sync_views_to_selected_date(&mut self) {
        let date = self.selected_date;
        let year = date.year();
        let month = date.month();

        // Update month view cache
        self.cache.set_current(year, month);
        self.cache.precache_surrounding(1, 2);

        // Update week view
        self.week_state = WeekState::new(date, self.locale.first_day_of_week, &self.locale);

        // Update day view
        self.day_state = DayState::new(date, &self.locale);

        // Update year view
        self.year_state = YearState::new(year);

        // Sync mini calendar to show the month containing selected_date
        self.mini_calendar_state = CalendarState::new(year, month);

        // Refresh cached events for the new month
        self.refresh_cached_events();
    }

    /// Refresh the cached events for the current month view
    pub fn refresh_cached_events(&mut self) {
        let cache_state = self.cache.current_state();
        self.cached_month_events = self.calendar_manager
            .get_display_events_for_month(cache_state.year, cache_state.month);
    }

    /// Update the selected calendar color cache
    pub fn update_selected_calendar_color(&mut self) {
        if let Some(ref cal_id) = self.selected_calendar_id {
            if let Some(calendar) = self.calendar_manager.sources().iter().find(|c| &c.info().id == cal_id) {
                self.selected_calendar_color = calendar.info().color.clone();
            }
        }
    }

    /// Set the selected date and sync all views
    pub fn set_selected_date(&mut self, date: NaiveDate) {
        self.selected_date = date;
        self.sync_views_to_selected_date();
    }

    /// Navigate to today in the current view
    pub fn navigate_to_today(&mut self) {
        let today = chrono::Local::now().date_naive();
        self.set_selected_date(today);
    }

    /// Navigate to the previous period based on current view
    pub fn navigate_mini_calendar_previous(&mut self) {
        let state = &self.mini_calendar_state;
        let (year, month) = if state.month == 1 {
            (state.year - 1, 12)
        } else {
            (state.year, state.month - 1)
        };
        self.mini_calendar_state = CalendarState::new(year, month);
    }

    /// Navigate to the next period based on current view
    pub fn navigate_mini_calendar_next(&mut self) {
        let state = &self.mini_calendar_state;
        let (year, month) = if state.month == 12 {
            (state.year + 1, 1)
        } else {
            (state.year, state.month + 1)
        };
        self.mini_calendar_state = CalendarState::new(year, month);
    }

    /// Render the sidebar
    pub fn render_sidebar(&self) -> Element<'_, Message> {
        let selected_day = if self.mini_calendar_state.year == self.selected_date.year()
            && self.mini_calendar_state.month == self.selected_date.month()
        {
            Some(self.selected_date.day())
        } else {
            None
        };

        views::render_sidebar(
            &self.mini_calendar_state,
            self.calendar_manager.sources(),
            selected_day,
            self.color_picker_open.as_ref(),
            self.selected_calendar_id.as_ref(),
        )
    }

    /// Render the main content area (toolbar + calendar view)
    pub fn render_main_content(&self) -> Element<'_, Message> {
        // Build month events with quick event state if editing
        let quick_event_data: Option<(chrono::NaiveDate, &str, &str)> = self.quick_event_editing
            .as_ref()
            .map(|(date, text)| (*date, text.as_str(), self.selected_calendar_color.as_str()));

        let month_events = views::MonthViewEvents {
            events_by_date: &self.cached_month_events,
            quick_event: quick_event_data,
        };

        views::render_main_content(
            &self.cache,
            &self.week_state,
            &self.day_state,
            &self.year_state,
            &self.locale,
            self.current_view,
            Some(self.selected_date),
            self.settings.show_week_numbers,
            Some(month_events),
        )
    }
}

impl Default for CosmicCalendar {
    fn default() -> Self {
        Self::initialize_app(Core::default())
    }
}

impl Application for CosmicCalendar {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, cosmic::app::Task<Self::Message>) {
        (Self::initialize_app(core), cosmic::app::Task::none())
    }

    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        components::render_header_start(&self.core, &self.key_binds, self.show_sidebar, self.settings.show_week_numbers)
    }

    fn header_end(&self) -> Vec<Element<'_, Self::Message>> {
        components::render_header_end()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        crate::layout::render_layout(self)
    }

    fn update(&mut self, message: Self::Message) -> cosmic::app::Task<Self::Message> {
        crate::update::handle_message(self, message)
    }

    fn context_drawer(&self) -> Option<cosmic::app::context_drawer::ContextDrawer<'_, Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(cosmic::app::context_drawer::about(
            &self.about,
            |url| Message::LaunchUrl(url.to_string()),
            Message::ToggleContextDrawer,
        ))
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Self::Message> {
        cosmic::iced::event::listen_with(|event, _status, _window_id| {
            match event {
                // Handle keyboard shortcuts
                cosmic::iced::Event::Keyboard(keyboard::Event::KeyPressed {
                    key,
                    modifiers,
                    ..
                }) => {
                    // Convert modifiers to menu modifiers
                    let mut menu_modifiers = Vec::new();
                    if modifiers.control() {
                        menu_modifiers.push(menu::key_bind::Modifier::Ctrl);
                    }
                    if modifiers.shift() {
                        menu_modifiers.push(menu::key_bind::Modifier::Shift);
                    }
                    if modifiers.alt() {
                        menu_modifiers.push(menu::key_bind::Modifier::Alt);
                    }
                    if modifiers.logo() {
                        menu_modifiers.push(menu::key_bind::Modifier::Super);
                    }

                    let key_bind = menu::KeyBind {
                        modifiers: menu_modifiers,
                        key: key.clone(),
                    };

                    // Look up the action in the global keyboard shortcuts
                    if let Some(action) = crate::keyboard::get_key_binds().get(&key_bind) {
                        return Some(action.message());
                    }
                    None
                }
                // Handle window resize to sync sidebar with condensed state
                // The actual condensed state is checked in update handler
                cosmic::iced::Event::Window(cosmic::iced::window::Event::Resized { .. }) => {
                    Some(Message::WindowResized)
                }
                _ => None,
            }
        })
    }
}
