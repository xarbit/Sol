//! Demo data generator for testing and demonstration purposes.
//!
//! This module generates realistic calendar events across a full year
//! to showcase the calendar application's capabilities.

use crate::caldav::{AlertTime, CalendarEvent, RepeatFrequency, TravelTime};
use crate::database::Database;
use chrono::{Datelike, Duration, NaiveDate, NaiveTime, TimeZone, Utc, Weekday};
use log::info;
use std::error::Error;
use uuid::Uuid;

/// Event templates for generating realistic demo data
struct EventTemplate {
    summary: &'static str,
    location: Option<&'static str>,
    duration_hours: i64,
    all_day: bool,
    travel_time: TravelTime,
    alert: AlertTime,
    notes: Option<&'static str>,
}

/// Generate demo data for the calendar
pub fn populate_demo_data(db: &Database) -> Result<usize, Box<dyn Error>> {
    info!("Generating demo data for a full year...");

    let today = chrono::Local::now().date_naive();
    let year_start = NaiveDate::from_ymd_opt(today.year(), 1, 1).unwrap();
    let year_end = NaiveDate::from_ymd_opt(today.year(), 12, 31).unwrap();

    let mut event_count = 0;

    // === WORK EVENTS (work calendar) ===
    event_count += generate_recurring_meetings(db, year_start, year_end)?;
    event_count += generate_project_deadlines(db, year_start, year_end)?;
    event_count += generate_work_events(db, year_start, year_end)?;

    // === PERSONAL EVENTS (personal calendar) ===
    event_count += generate_personal_events(db, year_start, year_end)?;
    event_count += generate_social_events(db, year_start, year_end)?;
    event_count += generate_health_fitness(db, year_start, year_end)?;

    // === HOLIDAYS AND SPECIAL DAYS ===
    event_count += generate_holidays(db, today.year())?;

    // === RANDOM VARIED EVENTS ===
    event_count += generate_varied_events(db, year_start, year_end)?;

    info!("Generated {} demo events", event_count);
    Ok(event_count)
}

/// Generate weekly recurring meetings using recurring events
fn generate_recurring_meetings(db: &Database, start: NaiveDate, end: NaiveDate) -> Result<usize, Box<dyn Error>> {
    let mut count = 0;

    // Find first Monday in range
    let mut first_monday = start;
    while first_monday.weekday() != Weekday::Mon && first_monday <= end {
        first_monday = first_monday.succ_opt().unwrap_or(first_monday);
    }

    if first_monday <= end {
        // Weekly Monday standup
        let event = CalendarEvent {
            uid: Uuid::new_v4().to_string(),
            summary: "Team Standup".to_string(),
            location: Some("Conference Room A".to_string()),
            all_day: false,
            start: Utc.from_utc_datetime(&first_monday.and_hms_opt(9, 0, 0).unwrap()),
            end: Utc.from_utc_datetime(&first_monday.and_hms_opt(10, 0, 0).unwrap()),
            travel_time: TravelTime::None,
            repeat: RepeatFrequency::Weekly,
            repeat_until: Some(end),
            exception_dates: vec![],
            invitees: vec![],
            alert: AlertTime::FifteenMinutes,
            alert_second: None,
            attachments: vec![],
            url: None,
            notes: Some("Daily sync with the team. Discuss blockers and progress.".to_string()),
        };
        db.insert_event("work", &event)?;
        count += 1;
    }

    // Find first Wednesday in range
    let mut first_wednesday = start;
    while first_wednesday.weekday() != Weekday::Wed && first_wednesday <= end {
        first_wednesday = first_wednesday.succ_opt().unwrap_or(first_wednesday);
    }

    if first_wednesday <= end {
        // Weekly Wednesday 1:1
        let event = CalendarEvent {
            uid: Uuid::new_v4().to_string(),
            summary: "1:1 with Manager".to_string(),
            location: Some("Manager's Office".to_string()),
            all_day: false,
            start: Utc.from_utc_datetime(&first_wednesday.and_hms_opt(14, 0, 0).unwrap()),
            end: Utc.from_utc_datetime(&first_wednesday.and_hms_opt(15, 0, 0).unwrap()),
            travel_time: TravelTime::None,
            repeat: RepeatFrequency::Weekly,
            repeat_until: Some(end),
            exception_dates: vec![],
            invitees: vec![],
            alert: AlertTime::ThirtyMinutes,
            alert_second: None,
            attachments: vec![],
            url: None,
            notes: Some("Weekly check-in. Bring status updates and questions.".to_string()),
        };
        db.insert_event("work", &event)?;
        count += 1;
    }

    // Find first Friday in range
    let mut first_friday = start;
    while first_friday.weekday() != Weekday::Fri && first_friday <= end {
        first_friday = first_friday.succ_opt().unwrap_or(first_friday);
    }

    if first_friday <= end {
        // Biweekly Friday sprint review
        let event = CalendarEvent {
            uid: Uuid::new_v4().to_string(),
            summary: "Sprint Review".to_string(),
            location: Some("Main Conference Room".to_string()),
            all_day: false,
            start: Utc.from_utc_datetime(&first_friday.and_hms_opt(15, 0, 0).unwrap()),
            end: Utc.from_utc_datetime(&first_friday.and_hms_opt(17, 0, 0).unwrap()),
            travel_time: TravelTime::None,
            repeat: RepeatFrequency::Biweekly,
            repeat_until: Some(end),
            exception_dates: vec![],
            invitees: vec![],
            alert: AlertTime::OneHour,
            alert_second: None,
            attachments: vec![],
            url: None,
            notes: Some("Demo completed work to stakeholders.".to_string()),
        };
        db.insert_event("work", &event)?;
        count += 1;
    }

    Ok(count)
}

/// Generate project deadlines throughout the year
fn generate_project_deadlines(db: &Database, start: NaiveDate, end: NaiveDate) -> Result<usize, Box<dyn Error>> {
    let deadlines = [
        ("Q1 Report Due", 3, 15),
        ("Product Launch v2.0", 4, 1),
        ("Mid-Year Review", 6, 30),
        ("Q2 Report Due", 6, 15),
        ("Conference Presentation", 9, 20),
        ("Q3 Report Due", 9, 15),
        ("Year-End Planning", 11, 15),
        ("Q4 Report Due", 12, 15),
    ];

    let mut count = 0;
    let year = start.year();

    for (summary, month, day) in deadlines {
        if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
            if date >= start && date <= end {
                insert_event(db, "work", EventTemplate {
                    summary,
                    location: None,
                    duration_hours: 0,
                    all_day: true,
                    travel_time: TravelTime::None,
                    alert: AlertTime::OneDay,
                    notes: Some("Important deadline - ensure all deliverables are ready."),
                }, date, NaiveTime::from_hms_opt(0, 0, 0).unwrap())?;
                count += 1;
            }
        }
    }

    Ok(count)
}

/// Generate various work events
fn generate_work_events(db: &Database, start: NaiveDate, end: NaiveDate) -> Result<usize, Box<dyn Error>> {
    let mut count = 0;
    let year = start.year();

    // Quarterly all-hands meetings
    for month in [1, 4, 7, 10] {
        if let Some(date) = NaiveDate::from_ymd_opt(year, month, 15) {
            if date >= start && date <= end {
                insert_event(db, "work", EventTemplate {
                    summary: "All-Hands Meeting",
                    location: Some("Auditorium"),
                    duration_hours: 2,
                    all_day: false,
                        travel_time: TravelTime::FifteenMinutes,
                    alert: AlertTime::OneHour,
                    notes: Some("Company-wide update from leadership."),
                }, date, NaiveTime::from_hms_opt(10, 0, 0).unwrap())?;
                count += 1;
            }
        }
    }

    // Training sessions
    let trainings = [
        ("Security Training", 2, 10),
        ("New Tools Workshop", 5, 5),
        ("Leadership Training", 8, 20),
        ("Compliance Training", 11, 10),
    ];

    for (summary, month, day) in trainings {
        if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
            if date >= start && date <= end {
                insert_event(db, "work", EventTemplate {
                    summary,
                    location: Some("Training Room B"),
                    duration_hours: 4,
                    all_day: false,
                        travel_time: TravelTime::None,
                    alert: AlertTime::OneDay,
                    notes: Some("Mandatory training session."),
                }, date, NaiveTime::from_hms_opt(9, 0, 0).unwrap())?;
                count += 1;
            }
        }
    }

    Ok(count)
}

/// Generate personal events
fn generate_personal_events(db: &Database, start: NaiveDate, end: NaiveDate) -> Result<usize, Box<dyn Error>> {
    let mut count = 0;
    let year = start.year();

    // Birthdays
    let birthdays = [
        ("Mom's Birthday", 3, 8),
        ("Dad's Birthday", 7, 22),
        ("Best Friend's Birthday", 5, 14),
        ("Partner's Birthday", 9, 3),
        ("My Birthday", 11, 18),
    ];

    for (summary, month, day) in birthdays {
        if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
            if date >= start && date <= end {
                insert_event(db, "personal", EventTemplate {
                    summary,
                    location: None,
                    duration_hours: 0,
                    all_day: true,
                    travel_time: TravelTime::None,
                    alert: AlertTime::OneWeek,
                    notes: Some("Don't forget to get a gift!"),
                }, date, NaiveTime::from_hms_opt(0, 0, 0).unwrap())?;
                count += 1;
            }
        }
    }

    // Appointments
    let appointments = [
        ("Dentist Appointment", 2, 20, 10, 30),
        ("Car Service", 4, 8, 8, 0),
        ("Eye Exam", 6, 12, 14, 0),
        ("Annual Physical", 8, 5, 9, 0),
        ("Haircut", 1, 15, 11, 0),
        ("Haircut", 3, 15, 11, 0),
        ("Haircut", 5, 15, 11, 0),
        ("Haircut", 7, 15, 11, 0),
        ("Haircut", 9, 15, 11, 0),
        ("Haircut", 11, 15, 11, 0),
    ];

    for (summary, month, day, hour, minute) in appointments {
        if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
            if date >= start && date <= end {
                insert_event(db, "personal", EventTemplate {
                    summary,
                    location: Some("Downtown"),
                    duration_hours: 1,
                    all_day: false,
                    travel_time: TravelTime::ThirtyMinutes,
                    alert: AlertTime::TwoHours,
                    notes: None,
                }, date, NaiveTime::from_hms_opt(hour, minute, 0).unwrap())?;
                count += 1;
            }
        }
    }

    Ok(count)
}

/// Generate social events
fn generate_social_events(db: &Database, start: NaiveDate, end: NaiveDate) -> Result<usize, Box<dyn Error>> {
    let mut count = 0;
    let year = start.year();

    let social = [
        ("Dinner with Friends", 1, 25, 19, 0, "Italian Restaurant"),
        ("Movie Night", 2, 14, 20, 0, "Cinema"),
        ("Game Night", 3, 9, 18, 0, "John's Place"),
        ("BBQ Party", 5, 28, 14, 0, "Backyard"),
        ("Beach Day", 7, 4, 10, 0, "Santa Monica Beach"),
        ("Concert", 8, 15, 20, 0, "The Forum"),
        ("Wine Tasting", 10, 12, 15, 0, "Napa Valley"),
        ("Thanksgiving Dinner", 11, 28, 16, 0, "Parents' House"),
        ("New Year's Eve Party", 12, 31, 21, 0, "Downtown Hotel"),
    ];

    for (summary, month, day, hour, minute, location) in social {
        if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
            if date >= start && date <= end {
                insert_event(db, "personal", EventTemplate {
                    summary,
                    location: Some(location),
                    duration_hours: 3,
                    all_day: false,
                    travel_time: TravelTime::ThirtyMinutes,
                    alert: AlertTime::OneHour,
                    notes: None,
                }, date, NaiveTime::from_hms_opt(hour, minute, 0).unwrap())?;
                count += 1;
            }
        }
    }

    Ok(count)
}

/// Generate health and fitness events using recurring events
fn generate_health_fitness(db: &Database, start: NaiveDate, end: NaiveDate) -> Result<usize, Box<dyn Error>> {
    let mut count = 0;

    // Find first Tuesday in range
    let mut first_tuesday = start;
    while first_tuesday.weekday() != Weekday::Tue && first_tuesday <= end {
        first_tuesday = first_tuesday.succ_opt().unwrap_or(first_tuesday);
    }

    if first_tuesday <= end {
        // Weekly Tuesday gym
        let event = CalendarEvent {
            uid: Uuid::new_v4().to_string(),
            summary: "Gym Workout".to_string(),
            location: Some("Fitness Center".to_string()),
            all_day: false,
            start: Utc.from_utc_datetime(&first_tuesday.and_hms_opt(6, 30, 0).unwrap()),
            end: Utc.from_utc_datetime(&first_tuesday.and_hms_opt(7, 30, 0).unwrap()),
            travel_time: TravelTime::FifteenMinutes,
            repeat: RepeatFrequency::Weekly,
            repeat_until: Some(end),
            exception_dates: vec![],
            invitees: vec![],
            alert: AlertTime::ThirtyMinutes,
            alert_second: None,
            attachments: vec![],
            url: None,
            notes: Some("Strength training day".to_string()),
        };
        db.insert_event("personal", &event)?;
        count += 1;
    }

    // Find first Thursday in range
    let mut first_thursday = start;
    while first_thursday.weekday() != Weekday::Thu && first_thursday <= end {
        first_thursday = first_thursday.succ_opt().unwrap_or(first_thursday);
    }

    if first_thursday <= end {
        // Weekly Thursday gym
        let event = CalendarEvent {
            uid: Uuid::new_v4().to_string(),
            summary: "Gym Workout".to_string(),
            location: Some("Fitness Center".to_string()),
            all_day: false,
            start: Utc.from_utc_datetime(&first_thursday.and_hms_opt(6, 30, 0).unwrap()),
            end: Utc.from_utc_datetime(&first_thursday.and_hms_opt(7, 30, 0).unwrap()),
            travel_time: TravelTime::FifteenMinutes,
            repeat: RepeatFrequency::Weekly,
            repeat_until: Some(end),
            exception_dates: vec![],
            invitees: vec![],
            alert: AlertTime::ThirtyMinutes,
            alert_second: None,
            attachments: vec![],
            url: None,
            notes: Some("Strength training day".to_string()),
        };
        db.insert_event("personal", &event)?;
        count += 1;
    }

    // Find first Saturday in range
    let mut first_saturday = start;
    while first_saturday.weekday() != Weekday::Sat && first_saturday <= end {
        first_saturday = first_saturday.succ_opt().unwrap_or(first_saturday);
    }

    if first_saturday <= end {
        // Weekly Saturday run
        let event = CalendarEvent {
            uid: Uuid::new_v4().to_string(),
            summary: "Morning Run".to_string(),
            location: Some("Park Trail".to_string()),
            all_day: false,
            start: Utc.from_utc_datetime(&first_saturday.and_hms_opt(7, 0, 0).unwrap()),
            end: Utc.from_utc_datetime(&first_saturday.and_hms_opt(8, 0, 0).unwrap()),
            travel_time: TravelTime::None,
            repeat: RepeatFrequency::Weekly,
            repeat_until: Some(end),
            exception_dates: vec![],
            invitees: vec![],
            alert: AlertTime::FifteenMinutes,
            alert_second: None,
            attachments: vec![],
            url: None,
            notes: Some("5K run".to_string()),
        };
        db.insert_event("personal", &event)?;
        count += 1;
    }

    Ok(count)
}

/// Generate US holidays
fn generate_holidays(db: &Database, year: i32) -> Result<usize, Box<dyn Error>> {
    let holidays = [
        ("New Year's Day", 1, 1),
        ("Martin Luther King Jr. Day", 1, 20), // Third Monday, approximated
        ("Presidents' Day", 2, 17), // Third Monday, approximated
        ("Memorial Day", 5, 26), // Last Monday, approximated
        ("Independence Day", 7, 4),
        ("Labor Day", 9, 1), // First Monday, approximated
        ("Columbus Day", 10, 13), // Second Monday, approximated
        ("Veterans Day", 11, 11),
        ("Thanksgiving", 11, 27), // Fourth Thursday, approximated
        ("Christmas Eve", 12, 24),
        ("Christmas Day", 12, 25),
        ("New Year's Eve", 12, 31),
    ];

    let mut count = 0;

    for (summary, month, day) in holidays {
        if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
            insert_event(db, "personal", EventTemplate {
                summary,
                location: None,
                duration_hours: 0,
                all_day: true,
                travel_time: TravelTime::None,
                alert: AlertTime::OneDay,
                notes: Some("Holiday"),
            }, date, NaiveTime::from_hms_opt(0, 0, 0).unwrap())?;
            count += 1;
        }
    }

    Ok(count)
}

/// Generate various random events to fill the calendar
fn generate_varied_events(db: &Database, start: NaiveDate, end: NaiveDate) -> Result<usize, Box<dyn Error>> {
    let mut count = 0;
    let year = start.year();

    // Multi-day events
    let vacations = [
        ("Spring Break Trip", 3, 20, 7),
        ("Summer Vacation", 7, 15, 14),
        ("Holiday Break", 12, 23, 10),
    ];

    for (summary, month, start_day, duration_days) in vacations {
        if let Some(start_date) = NaiveDate::from_ymd_opt(year, month, start_day) {
            if let Some(end_date) = start_date.checked_add_signed(Duration::days(duration_days - 1)) {
                if start_date >= start && end_date <= end {
                    let event = CalendarEvent {
                        uid: Uuid::new_v4().to_string(),
                        summary: summary.to_string(),
                        location: Some("Away".to_string()),
                        all_day: true,
                        start: Utc.from_utc_datetime(&start_date.and_hms_opt(0, 0, 0).unwrap()),
                        end: Utc.from_utc_datetime(&end_date.and_hms_opt(23, 59, 59).unwrap()),
                        travel_time: TravelTime::None,
                        repeat: RepeatFrequency::Never,
                        repeat_until: None,
                        exception_dates: vec![],
                        invitees: vec![],
                        alert: AlertTime::OneWeek,
                        alert_second: None,
                        attachments: vec![],
                        url: None,
                        notes: Some("Time off - out of office".to_string()),
                    };
                    db.insert_event("personal", &event)?;
                    count += 1;
                }
            }
        }
    }

    // Scattered single events with variety
    let misc_events = [
        ("Coffee with Mentor", "personal", 1, 10, 10, 0, 1, "Local Cafe"),
        ("Book Club", "personal", 1, 28, 19, 0, 2, "Library"),
        ("Team Building Event", "work", 2, 5, 13, 0, 4, "Escape Room"),
        ("Project Kickoff", "work", 2, 18, 10, 0, 2, "Board Room"),
        ("Networking Event", "work", 3, 5, 18, 0, 3, "Tech Hub"),
        ("Volunteer Work", "personal", 3, 22, 9, 0, 4, "Food Bank"),
        ("Photography Class", "personal", 4, 15, 14, 0, 2, "Community Center"),
        ("Client Presentation", "work", 4, 25, 11, 0, 1, "Client Office"),
        ("Cooking Class", "personal", 5, 10, 18, 0, 3, "Culinary School"),
        ("Tech Conference", "work", 5, 20, 9, 0, 8, "Convention Center"),
        ("Family Reunion", "personal", 6, 8, 12, 0, 6, "Grandparents' House"),
        ("Performance Review", "work", 6, 25, 14, 0, 1, "HR Office"),
        ("Art Gallery Opening", "personal", 7, 10, 19, 0, 2, "Downtown Gallery"),
        ("Strategy Meeting", "work", 7, 28, 9, 0, 3, "Executive Suite"),
        ("Yoga Retreat", "personal", 8, 3, 8, 0, 8, "Mountain Resort"),
        ("Product Demo", "work", 8, 28, 15, 0, 2, "Demo Room"),
        ("Language Class", "personal", 9, 10, 18, 30, 1, "Language Center"),
        ("Budget Review", "work", 9, 28, 10, 0, 2, "Finance Room"),
        ("Pottery Workshop", "personal", 10, 5, 10, 0, 3, "Art Studio"),
        ("Interview Candidate", "work", 10, 18, 14, 0, 1, "Meeting Room C"),
        ("Theater Show", "personal", 11, 8, 19, 30, 3, "Broadway Theater"),
        ("Year-End Party", "work", 12, 18, 18, 0, 4, "Hotel Ballroom"),
    ];

    for (summary, calendar, month, day, hour, minute, duration, location) in misc_events {
        if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
            if date >= start && date <= end {
                insert_event(db, calendar, EventTemplate {
                    summary,
                    location: Some(location),
                    duration_hours: duration,
                    all_day: false,
                    travel_time: TravelTime::FifteenMinutes,
                    alert: AlertTime::OneHour,
                    notes: None,
                }, date, NaiveTime::from_hms_opt(hour, minute, 0).unwrap())?;
                count += 1;
            }
        }
    }

    Ok(count)
}

/// Helper to insert an event
fn insert_event(
    db: &Database,
    calendar_id: &str,
    template: EventTemplate,
    date: NaiveDate,
    time: NaiveTime,
) -> Result<(), Box<dyn Error>> {
    let start_datetime = date.and_time(time);
    let end_datetime = if template.all_day {
        date.and_hms_opt(23, 59, 59).unwrap()
    } else {
        start_datetime + Duration::hours(template.duration_hours)
    };

    let event = CalendarEvent {
        uid: Uuid::new_v4().to_string(),
        summary: template.summary.to_string(),
        location: template.location.map(String::from),
        all_day: template.all_day,
        start: Utc.from_utc_datetime(&start_datetime),
        end: Utc.from_utc_datetime(&end_datetime),
        travel_time: template.travel_time,
        repeat: RepeatFrequency::Never,
        repeat_until: None,
        exception_dates: vec![],
        invitees: vec![],
        alert: template.alert,
        alert_second: None,
        attachments: vec![],
        url: None,
        notes: template.notes.map(String::from),
    };

    db.insert_event(calendar_id, &event)?;
    Ok(())
}
