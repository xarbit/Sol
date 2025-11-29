use chrono::Datelike;
use crate::localized_names;

/// Represents a day in the calendar grid with full date info
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CalendarDay {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    /// Whether this day is from the currently displayed month
    pub is_current_month: bool,
}

/// Cached calendar state to avoid recalculating on every render
#[derive(Debug, Clone, PartialEq)]
pub struct CalendarState {
    pub year: i32,
    pub month: u32,
    pub weeks: Vec<Vec<Option<u32>>>,
    /// Full weeks including adjacent month days for display
    pub weeks_full: Vec<Vec<CalendarDay>>,
    pub today: (i32, u32, u32), // (year, month, day)
    pub month_year_text: String, // Pre-formatted "Month Year" text
}

impl CalendarState {
    pub fn new(year: i32, month: u32) -> Self {
        let first_day = chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let first_weekday = first_day.weekday().num_days_from_monday();

        let days_in_month = if month == 12 {
            chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1)
                .unwrap()
                .signed_duration_since(first_day)
                .num_days()
        } else {
            chrono::NaiveDate::from_ymd_opt(year, month + 1, 1)
                .unwrap()
                .signed_duration_since(first_day)
                .num_days()
        };

        // Calculate previous month info
        let (prev_year, prev_month) = if month == 1 {
            (year - 1, 12)
        } else {
            (year, month - 1)
        };
        let prev_month_days = if month == 1 {
            chrono::NaiveDate::from_ymd_opt(year - 1, 12, 31).unwrap().day()
        } else {
            chrono::NaiveDate::from_ymd_opt(year, month, 1)
                .unwrap()
                .pred_opt()
                .unwrap()
                .day()
        };

        // Calculate next month info
        let (next_year, next_month) = if month == 12 {
            (year + 1, 1)
        } else {
            (year, month + 1)
        };

        let mut weeks = vec![];
        let mut weeks_full = vec![];
        let mut current_week = vec![];
        let mut current_week_full = vec![];

        // Fill in previous month days
        for i in 0..first_weekday {
            current_week.push(None);
            let prev_day = prev_month_days - (first_weekday - 1 - i);
            current_week_full.push(CalendarDay {
                year: prev_year,
                month: prev_month,
                day: prev_day,
                is_current_month: false,
            });
        }

        // Fill in current month days
        for day in 1..=days_in_month {
            current_week.push(Some(day as u32));
            current_week_full.push(CalendarDay {
                year,
                month,
                day: day as u32,
                is_current_month: true,
            });
            if current_week.len() == 7 {
                weeks.push(current_week.clone());
                weeks_full.push(current_week_full.clone());
                current_week.clear();
                current_week_full.clear();
            }
        }

        // Fill in next month days
        if !current_week.is_empty() {
            let mut next_day = 1u32;
            while current_week.len() < 7 {
                current_week.push(None);
                current_week_full.push(CalendarDay {
                    year: next_year,
                    month: next_month,
                    day: next_day,
                    is_current_month: false,
                });
                next_day += 1;
            }
            weeks.push(current_week);
            weeks_full.push(current_week_full);
        }

        let today = chrono::Local::now();
        let month_name = localized_names::get_month_name(month);
        let month_year_text = format!("{} {}", month_name, year);

        CalendarState {
            year,
            month,
            weeks,
            weeks_full,
            today: (today.year(), today.month(), today.day()),
            month_year_text,
        }
    }

    pub fn is_today(&self, day: u32) -> bool {
        self.today == (self.year, self.month, day)
    }

    #[allow(dead_code)] // Helper method for future use
    pub fn is_current_month(&self) -> bool {
        self.today.0 == self.year && self.today.1 == self.month
    }

    /// Get ISO 8601 week numbers for each week in the month
    /// Returns a vector of week numbers corresponding to each week in self.weeks
    pub fn week_numbers(&self) -> Vec<u32> {
        let mut week_numbers = Vec::new();

        for week in &self.weeks {
            // Find the first valid day in this week to determine the week number
            if let Some(Some(day)) = week.iter().find(|d| d.is_some()) {
                let date = chrono::NaiveDate::from_ymd_opt(self.year, self.month, *day).unwrap();
                week_numbers.push(date.iso_week().week());
            } else {
                // Empty week (shouldn't happen, but handle gracefully)
                week_numbers.push(0);
            }
        }

        week_numbers
    }

    /// Get the weekday for a specific day in the month
    pub fn get_weekday(&self, day: u32) -> chrono::Weekday {
        let date = chrono::NaiveDate::from_ymd_opt(self.year, self.month, day)
            .expect("Invalid date");
        date.weekday()
    }
}
