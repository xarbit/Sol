use crate::models::CalendarState;
use std::collections::HashMap;

/// Manages all calendar caching including state, formatted text, and pre-cached months
pub struct CalendarCache {
    /// Cache of calendar states by (year, month)
    states: HashMap<(i32, u32), CalendarState>,
    /// Cache of formatted period text by (year, month)
    period_texts: HashMap<(i32, u32), String>,
    /// Current active month
    current: (i32, u32),
}

impl CalendarCache {
    /// Create a new cache for the given year and month
    pub fn new(year: i32, month: u32) -> Self {
        let mut cache = CalendarCache {
            states: HashMap::new(),
            period_texts: HashMap::new(),
            current: (year, month),
        };

        // Cache the initial month
        cache.ensure_cached(year, month);

        cache
    }

    /// Update the current month and ensure it's cached
    pub fn set_current(&mut self, year: i32, month: u32) {
        let changed = self.current != (year, month);
        self.current = (year, month);

        if changed {
            self.ensure_cached(year, month);
        }
    }

    /// Get the current calendar state
    pub fn current_state(&self) -> &CalendarState {
        self.states.get(&self.current)
            .expect("Current month should always be cached")
    }

    /// Get the current formatted period text
    pub fn current_period_text(&self) -> &str {
        self.period_texts.get(&self.current)
            .map(|s| s.as_str())
            .expect("Current month period text should always be cached")
    }

    /// Ensure a specific month is cached
    fn ensure_cached(&mut self, year: i32, month: u32) {
        let key = (year, month);

        // Cache calendar state if not already present
        if !self.states.contains_key(&key) {
            self.states.insert(key, CalendarState::new(year, month));
        }

        // Cache period text if not already present
        if !self.period_texts.contains_key(&key) {
            let date = chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap();
            let period_text = format!("{}", date.format("%B %Y"));
            self.period_texts.insert(key, period_text);
        }
    }

    /// Pre-cache surrounding months (previous and next N months)
    pub fn precache_surrounding(&mut self, months_before: u32, months_after: u32) {
        let (year, month) = self.current;

        // Pre-cache previous months
        for i in 1..=months_before {
            let (prev_year, prev_month) = self.subtract_months(year, month, i);
            self.ensure_cached(prev_year, prev_month);
        }

        // Pre-cache next months
        for i in 1..=months_after {
            let (next_year, next_month) = self.add_months(year, month, i);
            self.ensure_cached(next_year, next_month);
        }
    }

    /// Add N months to a given year/month
    fn add_months(&self, year: i32, month: u32, n: u32) -> (i32, u32) {
        let total_months = (year * 12 + month as i32 - 1) + n as i32;
        let new_year = total_months / 12;
        let new_month = (total_months % 12) + 1;
        (new_year, new_month as u32)
    }

    /// Subtract N months from a given year/month
    fn subtract_months(&self, year: i32, month: u32, n: u32) -> (i32, u32) {
        let total_months = (year * 12 + month as i32 - 1) - n as i32;
        let new_year = total_months / 12;
        let new_month = (total_months % 12) + 1;
        (new_year, new_month as u32)
    }

    /// Clear old cached entries to prevent unbounded growth
    /// Keeps only the current month and surrounding months
    #[allow(dead_code)] // For future use when implementing cache cleanup strategies
    pub fn cleanup(&mut self, keep_radius: u32) {
        let (year, month) = self.current;

        // Collect keys to keep
        let mut keys_to_keep = vec![(year, month)];

        for i in 1..=keep_radius {
            keys_to_keep.push(self.subtract_months(year, month, i));
            keys_to_keep.push(self.add_months(year, month, i));
        }

        // Remove entries not in the keep list
        self.states.retain(|k, _| keys_to_keep.contains(k));
        self.period_texts.retain(|k, _| keys_to_keep.contains(k));
    }

    /// Get cache statistics for debugging
    #[allow(dead_code)]
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            states_cached: self.states.len(),
            period_texts_cached: self.period_texts.len(),
            current_month: self.current,
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct CacheStats {
    pub states_cached: usize,
    pub period_texts_cached: usize,
    pub current_month: (i32, u32),
}
