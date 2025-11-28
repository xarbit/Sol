/// Locale-aware formatting and settings based on system configuration
use std::env;

/// Locale preferences for calendar display
#[derive(Debug, Clone, PartialEq)]
pub struct LocalePreferences {
    pub use_24_hour: bool,
    pub first_day_of_week: chrono::Weekday,
    pub locale_string: String,
}

impl LocalePreferences {
    /// Detect locale preferences from environment variables
    pub fn detect_from_system() -> Self {
        let locale_string = env::var("LC_TIME")
            .or_else(|_| env::var("LC_ALL"))
            .or_else(|_| env::var("LANG"))
            .unwrap_or_else(|_| "en_US.UTF-8".to_string());

        let use_24_hour = detect_24_hour_format(&locale_string);
        let first_day_of_week = detect_first_day_of_week(&locale_string);

        LocalePreferences {
            use_24_hour,
            first_day_of_week,
            locale_string,
        }
    }

    /// Format hour for display (12h or 24h format)
    pub fn format_hour(&self, hour: u32) -> String {
        if self.use_24_hour {
            format!("{:02}:00", hour)
        } else {
            // 12-hour format with AM/PM
            if hour == 0 {
                "12 AM".to_string()
            } else if hour < 12 {
                format!("{} AM", hour)
            } else if hour == 12 {
                "12 PM".to_string()
            } else {
                format!("{} PM", hour - 12)
            }
        }
    }

    /// Get the number of days to subtract from Monday to get first day of week
    /// Monday = 0, Tuesday = 1, ..., Sunday = 6
    pub fn days_from_monday(&self) -> i64 {
        use chrono::Weekday;
        match self.first_day_of_week {
            Weekday::Mon => 0,
            Weekday::Tue => -1,
            Weekday::Wed => -2,
            Weekday::Thu => -3,
            Weekday::Fri => -4,
            Weekday::Sat => -5,
            Weekday::Sun => -6,
        }
    }
}

impl Default for LocalePreferences {
    fn default() -> Self {
        Self::detect_from_system()
    }
}

/// Detect if locale uses 24-hour format
fn detect_24_hour_format(locale: &str) -> bool {
    // Most locales use 24-hour format except:
    // - US (en_US)
    // - UK traditionally uses 12h but transitioning to 24h
    // - Canada (en_CA) uses 12h
    // - Australia (en_AU) traditionally 12h
    // - Philippines (fil_PH, en_PH)

    let locale_lower = locale.to_lowercase();

    // Explicit 12-hour format locales
    let twelve_hour_locales = [
        "en_us", "en_ca", "en_au", "en_nz", "en_ph",
        "fil_ph", "tl_ph"
    ];

    // Check if it's a known 12-hour locale
    for twelve_hour in &twelve_hour_locales {
        if locale_lower.starts_with(twelve_hour) {
            return false;
        }
    }

    // Default to 24-hour for all other locales
    true
}

/// Detect first day of week from locale
fn detect_first_day_of_week(locale: &str) -> chrono::Weekday {
    use chrono::Weekday;

    let locale_lower = locale.to_lowercase();

    // Locales that start week on Sunday
    let sunday_locales = [
        "en_us", "en_ca", "en_au", "en_nz", "en_ph",
        "ja_jp", "ko_kr", "zh_cn", "zh_tw", "zh_hk",
        "he_il", "ar_sa", "ar_ae", "ar_eg",
        "fil_ph", "tl_ph", "pt_br"
    ];

    // Locales that start week on Saturday
    let saturday_locales = [
        "ar_iq", "ar_ly", "ar_om", "ar_qa", "ar_sd",
        "ar_sy", "ar_ye"
    ];

    // Check for Sunday-starting locales
    for sunday_locale in &sunday_locales {
        if locale_lower.starts_with(sunday_locale) {
            return Weekday::Sun;
        }
    }

    // Check for Saturday-starting locales
    for saturday_locale in &saturday_locales {
        if locale_lower.starts_with(saturday_locale) {
            return Weekday::Sat;
        }
    }

    // Default to Monday (ISO 8601 standard) for most European and other locales
    Weekday::Mon
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Weekday;

    #[test]
    fn test_24_hour_detection() {
        assert_eq!(detect_24_hour_format("de_DE.UTF-8"), true);
        assert_eq!(detect_24_hour_format("en_GB.UTF-8"), true);
        assert_eq!(detect_24_hour_format("fr_FR.UTF-8"), true);
        assert_eq!(detect_24_hour_format("en_US.UTF-8"), false);
        assert_eq!(detect_24_hour_format("en_CA.UTF-8"), false);
    }

    #[test]
    fn test_first_day_detection() {
        assert_eq!(detect_first_day_of_week("de_DE.UTF-8"), Weekday::Mon);
        assert_eq!(detect_first_day_of_week("en_GB.UTF-8"), Weekday::Mon);
        assert_eq!(detect_first_day_of_week("en_US.UTF-8"), Weekday::Sun);
        assert_eq!(detect_first_day_of_week("ja_JP.UTF-8"), Weekday::Sun);
        assert_eq!(detect_first_day_of_week("ar_SA.UTF-8"), Weekday::Sun);
    }

    #[test]
    fn test_hour_formatting() {
        let locale_24h = LocalePreferences {
            use_24_hour: true,
            first_day_of_week: Weekday::Mon,
            locale_string: "de_DE.UTF-8".to_string(),
        };

        assert_eq!(locale_24h.format_hour(0), "00:00");
        assert_eq!(locale_24h.format_hour(13), "13:00");
        assert_eq!(locale_24h.format_hour(23), "23:00");

        let locale_12h = LocalePreferences {
            use_24_hour: false,
            first_day_of_week: Weekday::Sun,
            locale_string: "en_US.UTF-8".to_string(),
        };

        assert_eq!(locale_12h.format_hour(0), "12 AM");
        assert_eq!(locale_12h.format_hour(1), "1 AM");
        assert_eq!(locale_12h.format_hour(12), "12 PM");
        assert_eq!(locale_12h.format_hour(13), "1 PM");
    }
}
