//! Navigation-related message handlers (Previous/Next period, view changes)

use chrono::{Datelike, NaiveDate};
use crate::app::CosmicCalendar;
use crate::views::CalendarView;

/// Direction for period navigation
pub enum NavigationDirection {
    Previous,
    Next,
}

/// Handle period navigation (previous or next) based on current view
pub fn handle_period_navigation(app: &mut CosmicCalendar, direction: NavigationDirection) {
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
pub fn handle_previous_period(app: &mut CosmicCalendar) {
    handle_period_navigation(app, NavigationDirection::Previous);
}

/// Handle next period navigation
pub fn handle_next_period(app: &mut CosmicCalendar) {
    handle_period_navigation(app, NavigationDirection::Next);
}
