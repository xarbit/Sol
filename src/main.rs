mod app;
mod cache;
mod caldav;
mod calendars;
mod color_constants;
mod components;
mod database;
mod dialogs;
mod keyboard;
mod layout;
mod layout_constants;
mod locale;
mod localize;
mod localized_names;
mod logging;
mod menu_action;
mod message;
mod models;
mod protocols;
mod selection;
mod services;
mod settings;
mod storage;
mod styles;
mod ui_constants;
mod update;
mod validation;
mod views;

use app::CosmicCalendar;
use cosmic::app::Settings;
use database::Database;
use log::info;
use std::env;

/// Application entry point
pub fn main() -> cosmic::iced::Result {
    // Initialize centralized logging
    logging::init();

    info!("Sol Calendar starting...");

    // Check for --clear-events flag
    let args: Vec<String> = env::args().collect();
    if args.contains(&"--clear-events".to_string()) {
        info!("--clear-events flag detected, clearing all events from database");
        match Database::open() {
            Ok(db) => {
                match db.clear_all_events() {
                    Ok(count) => {
                        info!("Cleared {} events from database", count);
                        println!("Cleared {} events from database", count);
                    }
                    Err(e) => {
                        log::error!("Failed to clear events: {}", e);
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to open database: {}", e);
            }
        }
    }

    // Initialize localization system
    localize::init();

    info!("Localization initialized, launching application");
    cosmic::app::run::<CosmicCalendar>(Settings::default(), ())
}
