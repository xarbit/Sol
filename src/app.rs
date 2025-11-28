use crate::cache::CalendarCache;
use crate::calendars::{CalendarManager, LocalCalendar};
use crate::components;
use crate::fl;
use crate::message::Message;
use crate::storage::LocalStorage;
use crate::views::{self, CalendarView};
use chrono::Datelike;
use cosmic::app::Core;
use cosmic::iced::window;
use cosmic::widget::{self, about};
use cosmic::{Application, Element};

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
    pub about: about::About,
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
        let calendar_manager = Self::create_default_calendars();

        // Create About dialog
        let about = about::About::default()
            .name(fl!("app-title"))
            .version(env!("CARGO_PKG_VERSION"))
            .license(fl!("about-license"))
            .links([(fl!("about-repository"), "https://github.com/xarbit/sol")]);

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
            about,
        }
    }

    /// Create default calendar sources
    fn create_default_calendars() -> CalendarManager {
        let mut calendar_manager = CalendarManager::new();

        // Add default local calendars
        calendar_manager.add_source(Box::new(LocalCalendar::with_color(
            "personal".to_string(),
            "Personal".to_string(),
            "#3B82F6".to_string(),
        )));

        calendar_manager.add_source(Box::new(LocalCalendar::with_color(
            "work".to_string(),
            "Work".to_string(),
            "#8B5CF6".to_string(),
        )));

        calendar_manager
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
        views::render_main_content(&self.cache, self.current_view, self.selected_day)
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
        components::render_header_start()
    }

    fn header_end(&self) -> Vec<Element<'_, Self::Message>> {
        components::render_header_end()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        crate::layout::render_layout(self)
    }

    fn update(&mut self, message: Self::Message) -> cosmic::app::Task<Self::Message> {
        crate::update::handle_message(self, message);
        cosmic::app::Task::none()
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
}
