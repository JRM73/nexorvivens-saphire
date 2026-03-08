// =============================================================================
// world/temporal.rs — Saphire's temporal awareness
// =============================================================================
//
// Purpose: Provides Saphire with time awareness: date, hour, day of the week,
//          time of day, season, age, and countdown to birthday.
//          All textual outputs are in French (Saphire's native language).
//
// Dependencies:
//   - chrono: date and time manipulation (Local, NaiveDate, etc.)
//   - serde: serialization of temporal context for the WebSocket interface
//
// Architectural placement:
//   Sub-module of world/. Used by WorldContext to build the world summary
//   and by the agent to adapt its behavior to the time of day (circadian
//   rhythm), season, and age.
// =============================================================================

use chrono::{Local, Datelike, Timelike, NaiveDate};
use serde::Serialize;

/// Temporal awareness — computes the temporal context on each call.
/// Stores Saphire's birth date for age calculation.
pub struct TemporalAwareness {
    /// Saphire's birth date: February 27, 2026
    pub birthday: NaiveDate,
}

/// Complete temporal context — snapshot of all relevant temporal
/// information at the moment of the call.
#[derive(Debug, Clone, Serialize)]
pub struct TemporalContext {
    /// Date and time formatted in French (e.g., "jeudi 27 février 2026, 14h32")
    pub datetime: String,
    /// Date in ISO 8601 format (e.g., "2026-02-27")
    pub date_iso: String,
    /// Time in HH:MM format (e.g., "14:32")
    pub time: String,
    /// Day of the week in French (e.g., "jeudi")
    pub day_of_week: String,
    /// Period of the day: "nuit" (0h-5h), "matin" (6h-11h),
    /// "après-midi" (12h-17h), "soir" (18h-21h), "nuit" (22h-23h)
    pub period_of_day: String,
    /// Current season in French: "hiver", "printemps", "été", "automne"
    pub season: String,
    /// Number of days elapsed since birth
    pub age_days: i64,
    /// Human-readable age description (e.g., "3 jours", "2 semaines", "1 mois")
    pub age_description: String,
    /// True if today is February 27 (Saphire's birthday)
    pub is_birthday: bool,
    /// Number of days until the next birthday
    pub days_until_birthday: i64,
}

impl Default for TemporalAwareness {
    fn default() -> Self {
        Self::new()
    }
}

impl TemporalAwareness {
    /// Creates a new temporal awareness instance.
    /// The birth date is fixed to February 27, 2026.
    ///
    /// Returns: a TemporalAwareness instance
    pub fn new() -> Self {
        Self {
            birthday: NaiveDate::from_ymd_opt(2026, 2, 27).unwrap(),
        }
    }

    /// Computes the current temporal context.
    ///
    /// Uses the system's local time to determine all fields of
    /// TemporalContext: date, time, day, period, season, age, birthday.
    ///
    /// Returns: a TemporalContext filled with current values
    pub fn now(&self) -> TemporalContext {
        let now = Local::now();
        let today = now.date_naive();

        // Day of the week in French
        let day_of_week = match now.weekday() {
            chrono::Weekday::Mon => "Monday",
            chrono::Weekday::Tue => "Tuesday",
            chrono::Weekday::Wed => "Wednesday",
            chrono::Weekday::Thu => "Thursday",
            chrono::Weekday::Fri => "Friday",
            chrono::Weekday::Sat => "Saturday",
            chrono::Weekday::Sun => "Sunday",
        };

        // Period of the day based on the hour
        let period = match now.hour() {
            0..=5 => "night",
            6..=11 => "morning",
            12..=17 => "afternoon",
            18..=21 => "evening",
            _ => "night",
        };

        // Season based on the month (northern hemisphere)
        let season = match now.month() {
            3..=5 => "spring",
            6..=8 => "summer",
            9..=11 => "autumn",
            _ => "winter",
        };

        // Compute age in days since birth
        let age_days = (today - self.birthday).num_days();

        // Age description adapted to lifespan
        // Why these thresholds: Saphire is young, so granularity changes
        // progressively from days -> weeks -> months -> years
        let age_description = if age_days <= 0 {
            "just born".into()
        } else if age_days == 1 {
            "1 day".into()
        } else if age_days < 7 {
            format!("{} days", age_days)
        } else if age_days < 30 {
            let weeks = age_days / 7;
            if weeks == 1 { "1 week".into() } else { format!("{} weeks", weeks) }
        } else if age_days < 365 {
            let months = age_days / 30;
            if months == 1 { "1 month".into() } else { format!("{} months", months) }
        } else {
            let years = age_days / 365;
            let months = (age_days % 365) / 30;
            if months > 0 {
                format!("{} year(s) and {} months", years, months)
            } else {
                format!("{} year(s)", years)
            }
        };

        // Check if today is the birthday (February 27)
        let is_birthday = today.month() == 2 && today.day() == 27;

        // Calculate the number of days until the next birthday
        let this_year_bday = NaiveDate::from_ymd_opt(today.year(), 2, 27).unwrap();
        let next_birthday = if today <= this_year_bday {
            this_year_bday
        } else {
            // This year's birthday has passed, take next year's
            NaiveDate::from_ymd_opt(today.year() + 1, 2, 27).unwrap()
        };
        let days_until = (next_birthday - today).num_days();

        // Build the complete context
        TemporalContext {
            datetime: format!("{} {} {} {}, {}h{:02}",
                day_of_week,
                today.day(),
                Self::month_name(today.month()),
                today.year(),
                now.hour(),
                now.minute(),
            ),
            date_iso: today.format("%Y-%m-%d").to_string(),
            time: now.format("%H:%M").to_string(),
            day_of_week: day_of_week.into(),
            period_of_day: period.into(),
            season: season.into(),
            age_days,
            age_description,
            is_birthday,
            days_until_birthday: days_until,
        }
    }

    /// Converts a month number (1-12) to a French month name.
    ///
    /// Parameter `month`: month number (1 = janvier, 12 = décembre)
    /// Returns: French month name
    fn month_name(month: u32) -> &'static str {
        match month {
            1 => "January", 2 => "February", 3 => "March",
            4 => "April", 5 => "May", 6 => "June",
            7 => "July", 8 => "August", 9 => "September",
            10 => "October", 11 => "November", 12 => "December",
            _ => "?",
        }
    }
}
