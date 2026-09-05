pub mod sys_locale;

#[cfg(target_vendor = "apple")]
mod apple;

#[cfg(windows)]
mod windows;

use jiff::Zoned;

use icu_calendar::preferences::FirstDay;
use icu_calendar::week::WeekPreferences;
use icu_calendar::{Date, Gregorian};
use icu_datetime::{FixedCalendarDateTimeFormatter, fieldsets};

pub use icu_calendar::options::DateAddOptions;
pub use icu_calendar::types::{DateDuration, EraYear, Month, Weekday, YearInfo};
pub use icu_locale_core::Locale;

/// Returns the first day of the week for the current system user.
///
/// This queries the native OS APIs where possible (Windows, Apple, WebAssembly),
/// and falls back to detecting the system locale (Android, Linux) to query the CLDR table.
pub fn first_day_of_week(locale: &Locale) -> Weekday {
    cfg_select! {
        windows => windows::first_day_of_week().unwrap_or_else(|| fallback_first_day_of_week(locale)),
        target_vendor = "apple" => apple::first_day_of_week().unwrap_or_else(|| fallback_first_day_of_week(locale)),
        _ => fallback_first_day_of_week(locale),
    }
}

fn fallback_first_day_of_week(locale: &Locale) -> Weekday {
    match WeekPreferences::from(locale).first_weekday {
        None => Weekday::Monday,
        Some(day) => match day {
            FirstDay::Sun => Weekday::Sunday,
            FirstDay::Mon => Weekday::Monday,
            FirstDay::Tue => Weekday::Tuesday,
            FirstDay::Wed => Weekday::Wednesday,
            FirstDay::Thu => Weekday::Thursday,
            FirstDay::Fri => Weekday::Friday,
            FirstDay::Sat => Weekday::Saturday,
            _ => unreachable!(),
        },
    }
}

pub fn day_abbreviation(locale: &Locale, day: Weekday) -> String {
    cfg_select! {
        windows => windows::day_abbreviation(day).unwrap_or_else(|| fallback_day_abbreviation(locale, day)),
        target_vendor = "apple" => apple::day_abbreviation(day).unwrap_or_else(|| fallback_day_abbreviation(locale, day)),
        _ => fallback_day_abbreviation(locale, day),
    }
}

fn fallback_day_abbreviation(locale: &Locale, day: Weekday) -> String {
    let formatter =
        FixedCalendarDateTimeFormatter::<Gregorian, _>::try_new(locale.into(), fieldsets::E::short()).unwrap();
    formatter.format(&day).to_string()
}

pub fn current_date() -> Date<Gregorian> {
    let now = Zoned::now();

    Date::try_new_gregorian(now.year().into(), now.month() as u8, now.day() as u8)
        .expect("Invalid date provided by system")
}

pub fn current_month() -> Date<Gregorian> {
    let now = Zoned::now();

    Date::try_new_gregorian(now.year().into(), now.month() as u8, 1).expect("Invalid date provided by system")
}

pub fn current_calendar_start(start_of_week: Weekday, year: i32, month: Month) -> Date<Gregorian> {
    let mut start = Date::try_new_gregorian(year, month.number(), 1).expect("Invalid date provided by system");

    let start_of_week = start_of_week as i32;
    let current_weekday = start.weekday() as i32;
    let days_to_subtract = (current_weekday - start_of_week + 7) % 7;

    if days_to_subtract > 0 {
        start
            .try_add_with_options(DateDuration::for_days(-days_to_subtract), DateAddOptions::default())
            .expect("Invalid date provided by system");
    }

    start
}

pub fn format_date_day_number(_locale: &Locale, date: &Date<Gregorian>) -> String {
    cfg_select! {
        windows => windows::format_date_day_number(date),
        target_vendor = "apple" => apple::format_date_day_number(date),
        _ => {
            let formatter =
                FixedCalendarDateTimeFormatter::<Gregorian, _>::try_new(_locale.into(), fieldsets::D::short())
                    .expect("Failed to create formatter");

            formatter.format(date).to_string()
        }
    }
}

pub fn month_name(locale: &Locale, month: Month, year: i32) -> String {
    cfg_select! {
        windows => windows::month_name(month).unwrap_or_else(|| fallback_month_name(locale, month, year)),
        target_vendor = "apple" => apple::month_name(month).unwrap_or_else(|| fallback_month_name(locale, month, year)),
        _ => fallback_month_name(locale, month, year),
    }
}

fn fallback_month_name(locale: &Locale, month: Month, year: i32) -> String {
    let date = Date::try_new_gregorian(year, month.number(), 1).expect("Invalid date");

    let formatter = FixedCalendarDateTimeFormatter::<Gregorian, _>::try_new(locale.into(), fieldsets::M::long())
        .expect("Failed to create formatter");

    formatter.format(&date).to_string()
}

pub fn year_name(_locale: &Locale, year: i32) -> String {
    cfg_select! {
        windows => windows::year_name(year),
        target_vendor = "apple" => apple::year_name(year),
        _ => {
            let date = Date::try_new_gregorian(year, 1, 1).expect("Invalid date components");
            let formatter =
                FixedCalendarDateTimeFormatter::<Gregorian, _>::try_new(_locale.into(), fieldsets::Y::long())
                    .expect("Failed to create formatter");

            formatter.format(&date).to_string()
        }
    }
}
