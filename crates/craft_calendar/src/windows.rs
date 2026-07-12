use icu::calendar::types::{Month, Weekday};
use icu::calendar::{Date, Gregorian};

use windows::core::PCWSTR;
use windows::Win32::Foundation::SYSTEMTIME;
use windows::Win32::Globalization::{GetDateFormatEx, GetLocaleInfoEx, ENUM_DATE_FORMATS_FLAGS, LOCALE_IFIRSTDAYOFWEEK, LOCALE_SMONTHNAME1, LOCALE_SMONTHNAME10, LOCALE_SMONTHNAME11, LOCALE_SMONTHNAME12, LOCALE_SMONTHNAME2, LOCALE_SMONTHNAME3, LOCALE_SMONTHNAME4, LOCALE_SMONTHNAME5, LOCALE_SMONTHNAME6, LOCALE_SMONTHNAME7, LOCALE_SMONTHNAME8, LOCALE_SMONTHNAME9, LOCALE_SSHORTESTDAYNAME1, LOCALE_SSHORTESTDAYNAME2, LOCALE_SSHORTESTDAYNAME3, LOCALE_SSHORTESTDAYNAME4, LOCALE_SSHORTESTDAYNAME5, LOCALE_SSHORTESTDAYNAME6, LOCALE_SSHORTESTDAYNAME7};

pub(super) fn first_day_of_week() -> Option<Weekday> {
    let mut buffer = [0u16; 8];

    let len = unsafe { GetLocaleInfoEx(PCWSTR::null(), LOCALE_IFIRSTDAYOFWEEK, Some(&mut buffer)) };

    if len == 0 {
        return None;
    }

    let value = String::from_utf16_lossy(&buffer[..len as usize - 1]);

    // https://learn.microsoft.com/en-us/windows/win32/intl/locale-ifirstdayofweek
    match value.parse::<u32>() {
        Ok(0) => Some(Weekday::Monday),
        Ok(1) => Some(Weekday::Tuesday),
        Ok(2) => Some(Weekday::Wednesday),
        Ok(3) => Some(Weekday::Thursday),
        Ok(4) => Some(Weekday::Friday),
        Ok(5) => Some(Weekday::Saturday),
        Ok(6) => Some(Weekday::Sunday),
        Ok(_) => unreachable!(),
        Err(_) => None,
    }
}

pub(super) fn day_abbreviation(day: Weekday) -> Option<String> {
    // https://learn.microsoft.com/en-us/windows/win32/intl/locale-sshortestdayname-constants
    let windows_day = match day {
        Weekday::Monday => LOCALE_SSHORTESTDAYNAME1,
        Weekday::Tuesday => LOCALE_SSHORTESTDAYNAME2,
        Weekday::Wednesday => LOCALE_SSHORTESTDAYNAME3,
        Weekday::Thursday => LOCALE_SSHORTESTDAYNAME4,
        Weekday::Friday => LOCALE_SSHORTESTDAYNAME5,
        Weekday::Saturday => LOCALE_SSHORTESTDAYNAME6,
        Weekday::Sunday => LOCALE_SSHORTESTDAYNAME7,
    };
    let mut buffer = [0u16; 8];

    let len = unsafe { GetLocaleInfoEx(PCWSTR::null(), windows_day, Some(&mut buffer)) };

    if len == 0 {
        return None;
    }

    Some(String::from_utf16_lossy(&buffer[..len as usize - 1]))
}

pub fn format_date_day_number(date: &Date<Gregorian>) -> String {
    let date_iso = date.to_calendar(Gregorian);

    let st = SYSTEMTIME {
        wYear: date_iso.year().extended_year() as u16,
        wMonth: date_iso.month().ordinal as u16,
        wDay: date_iso.day_of_month().0 as u16,
        wDayOfWeek: 0,
        ..Default::default()
    };

    let format_str: Vec<u16> = "d".encode_utf16().chain(std::iter::once(0)).collect();
    let mut buffer = [0u16; 32];

    let len = unsafe {
        GetDateFormatEx(
            PCWSTR::null(),
            ENUM_DATE_FORMATS_FLAGS(0),
            Some(&st),
            PCWSTR::from_raw(format_str.as_ptr()),
            Some(&mut buffer),
            None,
        )
    };

    if len == 0 {
        return String::new();
    }

    String::from_utf16_lossy(&buffer[..len as usize - 1])
}

pub fn month_name(month: Month) -> Option<String> {
    let windows_month = match month.number() {
        0 => LOCALE_SMONTHNAME1,
        1 => LOCALE_SMONTHNAME2,
        2 => LOCALE_SMONTHNAME3,
        3 => LOCALE_SMONTHNAME4,
        4 => LOCALE_SMONTHNAME5,
        5 => LOCALE_SMONTHNAME6,
        6 => LOCALE_SMONTHNAME7,
        7 => LOCALE_SMONTHNAME8,
        8 => LOCALE_SMONTHNAME9,
        9 => LOCALE_SMONTHNAME10,
        10 => LOCALE_SMONTHNAME11,
        11 => LOCALE_SMONTHNAME12,
        _ => unreachable!(),
    };

    let mut buffer = [0u16; 32];

    let len = unsafe { GetLocaleInfoEx(PCWSTR::null(), windows_month, Some(&mut buffer)) };

    if len == 0 {
        return None;
    }

    Some(String::from_utf16_lossy(&buffer[..len as usize - 1]))
}

pub fn year_name(year: i32) -> String {
    let st = SYSTEMTIME {
        wYear: year as u16,
        wMonth: 1,
        wDay: 1,
        ..Default::default()
    };

    let format_str: Vec<u16> = "yyyy".encode_utf16().chain(std::iter::once(0)).collect();
    let mut buffer = [0u16; 16];

    let len = unsafe {
        GetDateFormatEx(
            PCWSTR::null(),
            ENUM_DATE_FORMATS_FLAGS(0),
            Some(&st),
            PCWSTR::from_raw(format_str.as_ptr()),
            Some(&mut buffer),
            None,
        )
    };

    if len == 0 {
        return year.to_string();
    }
    String::from_utf16_lossy(&buffer[..len as usize - 1])
}
