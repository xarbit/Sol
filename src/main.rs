mod app;
mod cache;
mod caldav;
mod calendars;
mod color_constants;
mod components;
mod keyboard;
mod layout;
mod layout_constants;
mod locale;
mod localize;
mod localized_names;
mod menu_action;
mod message;
mod models;
mod settings;
mod storage;
mod styles;
mod ui_constants;
mod update;
mod views;

use app::CosmicCalendar;
use cosmic::app::Settings;

/// Application entry point
pub fn main() -> cosmic::iced::Result {
    // Initialize localization system
    localize::init();

    cosmic::app::run::<CosmicCalendar>(Settings::default(), ())
}
