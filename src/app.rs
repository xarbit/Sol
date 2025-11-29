use crate::cache::CalendarCache;
use crate::calendars::CalendarManager;
use crate::components;
use crate::fl;
use crate::locale::LocalePreferences;
use crate::menu_action::MenuAction;
use crate::message::Message;
use crate::models::{WeekState, DayState, YearState};
use crate::settings::AppSettings;
use crate::storage::LocalStorage;
use crate::views::{self, CalendarView};
use chrono::Datelike;
use cosmic::app::Core;
use cosmic::iced::keyboard;
use cosmic::widget::{about, menu};
use cosmic::widget::menu::Action as _; // Import trait for .message() method
use cosmic::{Application, Element};
use std::collections::HashMap;

const APP_ID: &str = "io.github.xarbit.SolCalendar";

/// Main application state
pub struct CosmicCalendar {
    pub core: Core,
    pub current_view: CalendarView,
    pub current_year: i32,
    pub current_month: u32,
    pub selected_day: Option<u32>,
    #[allow(dead_code)] // Will be used for event storage in future
    pub storage: LocalStorage,
    pub calendar_manager: CalendarManager,
    pub show_sidebar: bool,
    pub show_search: bool,
    pub color_picker_open: Option<String>,
    pub cache: CalendarCache,
    pub week_state: WeekState,
    pub day_state: DayState,
    pub year_state: YearState,
    pub locale: LocalePreferences,
    pub settings: AppSettings,
    pub about: about::About,
    pub key_binds: HashMap<menu::KeyBind, MenuAction>,
}

impl CosmicCalendar {
    /// Initialize the application with default calendars
    fn initialize_app(core: Core) -> Self {
        let now = chrono::Local::now();
        let storage_path = LocalStorage::get_storage_path();
        let storage = LocalStorage::load_from_file(&storage_path).unwrap_or_default();

        let year = now.year();
        let month = now.month();

        // Create cache and pre-cache surrounding months
        let mut cache = CalendarCache::new(year, month);
        cache.precache_surrounding(1, 2);

        // Initialize calendar manager with default calendars
        let calendar_manager = CalendarManager::with_defaults();

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

        CosmicCalendar {
            core,
            current_view: CalendarView::Month,
            current_year: year,
            current_month: month,
            selected_day: Some(now.day()),
            storage,
            calendar_manager,
            show_sidebar: true,
            show_search: false,
            color_picker_open: None,
            cache,
            week_state: WeekState::current_with_first_day(locale.first_day_of_week, &locale),
            day_state: DayState::current(&locale),
            year_state: YearState::current(),
            locale,
            settings,
            about,
            key_binds,
        }
    }

    /// Navigate to the previous month
    pub fn navigate_to_previous_month(&mut self) {
        if self.current_month == 1 {
            self.current_month = 12;
            self.current_year -= 1;
        } else {
            self.current_month -= 1;
        }
        self.cache.set_current(self.current_year, self.current_month);
        self.cache.precache_surrounding(1, 2);
    }

    /// Navigate to the next month
    pub fn navigate_to_next_month(&mut self) {
        if self.current_month == 12 {
            self.current_month = 1;
            self.current_year += 1;
        } else {
            self.current_month += 1;
        }
        self.cache.set_current(self.current_year, self.current_month);
        self.cache.precache_surrounding(1, 2);
    }

    /// Navigate to today
    pub fn navigate_to_today(&mut self) {
        let now = chrono::Local::now();
        self.current_year = now.year();
        self.current_month = now.month();
        self.selected_day = Some(now.day());
        self.cache.set_current(self.current_year, self.current_month);
        self.cache.precache_surrounding(1, 2);
    }

    /// Render the sidebar
    pub fn render_sidebar(&self) -> Element<'_, Message> {
        views::render_sidebar(
            self.cache.current_state(),
            self.calendar_manager.sources(),
            self.selected_day,
            self.color_picker_open.as_ref(),
        )
    }

    /// Render the main content area (toolbar + calendar view)
    pub fn render_main_content(&self) -> Element<'_, Message> {
        views::render_main_content(&self.cache, &self.week_state, &self.day_state, &self.year_state, &self.locale, self.current_view, self.selected_day, self.settings.show_week_numbers)
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
            if let cosmic::iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                modifiers,
                ..
            }) = event
            {
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
            }
            None
        })
    }
}
