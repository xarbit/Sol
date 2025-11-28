use chrono::Datelike;

/// Cached calendar state to avoid recalculating on every render
#[derive(Debug, Clone, PartialEq)]
pub struct CalendarState {
    pub year: i32,
    pub month: u32,
    pub weeks: Vec<Vec<Option<u32>>>,
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

        let mut weeks = vec![];
        let mut current_week = vec![];

        for _ in 0..first_weekday {
            current_week.push(None);
        }

        for day in 1..=days_in_month {
            current_week.push(Some(day as u32));
            if current_week.len() == 7 {
                weeks.push(current_week.clone());
                current_week.clear();
            }
        }

        if !current_week.is_empty() {
            while current_week.len() < 7 {
                current_week.push(None);
            }
            weeks.push(current_week);
        }

        let today = chrono::Local::now();
        let month_year_text = format!("{}", first_day.format("%B %Y"));

        CalendarState {
            year,
            month,
            weeks,
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
}
