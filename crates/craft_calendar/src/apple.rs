use std::ffi::c_void;

use icu::calendar::types::{Weekday};
use icu::calendar::{Date, Gregorian};

pub(super) fn first_day_of_week() -> Option<Weekday> {
    use std::ffi::c_void;

    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFCalendarCopyCurrent() -> *mut c_void;
        fn CFCalendarGetFirstWeekday(calendar: *mut c_void) -> isize;
        fn CFRelease(cf: *mut c_void);
    }

    unsafe {
        let calendar = CFCalendarCopyCurrent();
        if calendar.is_null() {
            return None;
        }

        // https://developer.apple.com/documentation/foundation/calendar/firstweekday
        let day = CFCalendarGetFirstWeekday(calendar);
        CFRelease(calendar);

        match day {
            1 => Some(Weekday::Sunday),
            2 => Some(Weekday::Monday),
            3 => Some(Weekday::Tuesday),
            4 => Some(Weekday::Wednesday),
            5 => Some(Weekday::Thursday),
            6 => Some(Weekday::Friday),
            7 => Some(Weekday::Saturday),
            _ => None,
        }
    }
}

pub fn day_abbreviation(day: Weekday) -> Option<String> {
    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        static kCFDateFormatterVeryShortWeekdaySymbols: *mut c_void;

        fn CFLocaleCopyCurrent() -> *mut c_void;
        fn CFDateFormatterCreate(
            allocator: *mut c_void,
            locale: *mut c_void,
            dateStyle: isize,
            timeStyle: isize,
        ) -> *mut c_void;
        fn CFDateFormatterCopyProperty(formatter: *mut c_void, key: *mut c_void) -> *mut c_void;
        fn CFArrayGetValueAtIndex(theArray: *mut c_void, idx: isize) -> *mut c_void;
        fn CFStringGetCString(theString: *mut c_void, buffer: *mut i8, bufferSize: isize, encoding: u32) -> u8;
        fn CFRelease(cf: *mut c_void);
    }

    unsafe {
        let locale = CFLocaleCopyCurrent();
        if locale.is_null() {
            return None;
        }

        let formatter = CFDateFormatterCreate(std::ptr::null_mut(), locale, 0, 0);
        CFRelease(locale);
        if formatter.is_null() {
            return None;
        }

        let symbols_array = CFDateFormatterCopyProperty(formatter, kCFDateFormatterVeryShortWeekdaySymbols);
        CFRelease(formatter);
        if symbols_array.is_null() {
            return None;
        }

        // CFArray of weekday symbols is 0-indexed, starting with Sunday
        let index = match day {
            Weekday::Sunday => 0,
            Weekday::Monday => 1,
            Weekday::Tuesday => 2,
            Weekday::Wednesday => 3,
            Weekday::Thursday => 4,
            Weekday::Friday => 5,
            Weekday::Saturday => 6,
        };

        let cf_string = CFArrayGetValueAtIndex(symbols_array, index);
        if cf_string.is_null() {
            CFRelease(symbols_array);
            return None;
        }

        let mut buffer = [0u8; 64];
        let success = CFStringGetCString(
            cf_string,
            buffer.as_mut_ptr() as *mut i8,
            buffer.len() as isize,
            0x08000100,
        );

        CFRelease(symbols_array);

        if success == 0 {
            return None;
        }

        let c_str = std::ffi::CStr::from_ptr(buffer.as_ptr() as *const i8);
        Some(c_str.to_string_lossy().into_owned())
    }
}

pub fn format_date_day_number(date: &Date<Gregorian>) -> String {
    let date_iso = date.to_calendar(Gregorian);

    #[repr(C)]
    struct CFGregorianDate {
        year: i32,
        month: i8,
        day: i8,
        hour: i8,
        minute: i8,
        second: f64,
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFLocaleCopyCurrent() -> *mut c_void;
        fn CFDateFormatterCreate(
            allocator: *mut c_void,
            locale: *mut c_void,
            dateStyle: isize,
            timeStyle: isize,
        ) -> *mut c_void;
        fn CFStringCreateWithBytes(
            alloc: *mut c_void,
            bytes: *const u8,
            numBytes: isize,
            encoding: u32,
            isExternalRepresentation: u8,
        ) -> *mut c_void;
        fn CFDateFormatterSetFormat(formatter: *mut c_void, formatString: *mut c_void);
        fn CFGregorianDateGetAbsoluteTime(gdate: CFGregorianDate, tz: *mut c_void) -> f64;
        fn CFDateFormatterCreateStringWithAbsoluteTime(
            allocator: *mut c_void,
            formatter: *mut c_void,
            at: f64,
        ) -> *mut c_void;
        fn CFStringGetCString(theString: *mut c_void, buffer: *mut i8, bufferSize: isize, encoding: u32) -> u8;
        fn CFRelease(cf: *mut c_void);
    }

    let gdate = CFGregorianDate {
        year: date_iso.year().extended_year(),
        month: date_iso.month().ordinal as i8,
        day: date_iso.day_of_month().0 as i8,
        hour: 12, // Use noon to safely avoid timezone border edge-cases
        minute: 0,
        second: 0.0,
    };

    unsafe {
        let locale = CFLocaleCopyCurrent();
        if locale.is_null() {
            return String::new();
        }

        let formatter = CFDateFormatterCreate(std::ptr::null_mut(), locale, 0, 0);
        CFRelease(locale);
        if formatter.is_null() {
            return String::new();
        }

        // Create CFString for format string "d"
        let format_bytes = b"d";
        let format_string = CFStringCreateWithBytes(
            std::ptr::null_mut(),
            format_bytes.as_ptr(),
            format_bytes.len() as isize,
            0x08000100, // kCFStringEncodingUTF8
            0,
        );

        if format_string.is_null() {
            CFRelease(formatter);
            return String::new();
        }

        CFDateFormatterSetFormat(formatter, format_string);
        CFRelease(format_string);

        // Convert Gregorian date to absolute time relative to local timezone
        let absolute_time = CFGregorianDateGetAbsoluteTime(gdate, std::ptr::null_mut());

        // Format absolute time to a CFString
        let result_string = CFDateFormatterCreateStringWithAbsoluteTime(std::ptr::null_mut(), formatter, absolute_time);
        CFRelease(formatter);

        if result_string.is_null() {
            return String::new();
        }

        let mut buffer = [0u8; 32];
        let success = CFStringGetCString(
            result_string,
            buffer.as_mut_ptr() as *mut i8,
            buffer.len() as isize,
            0x08000100, // kCFStringEncodingUTF8
        );
        CFRelease(result_string);

        if success == 0 {
            return String::new();
        }

        let c_str = std::ffi::CStr::from_ptr(buffer.as_ptr() as *const i8);
        c_str.to_string_lossy().into_owned()
    }
}

pub fn month_name(month: icu::calendar::types::Month) -> Option<String> {
    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        static kCFDateFormatterMonthSymbols: *mut c_void;

        fn CFLocaleCopyCurrent() -> *mut c_void;
        fn CFDateFormatterCreate(
            allocator: *mut c_void,
            locale: *mut c_void,
            dateStyle: isize,
            timeStyle: isize,
        ) -> *mut c_void;
        fn CFDateFormatterCopyProperty(formatter: *mut c_void, key: *mut c_void) -> *mut c_void;
        fn CFArrayGetValueAtIndex(theArray: *mut c_void, idx: isize) -> *mut c_void;
        fn CFStringGetCString(theString: *mut c_void, buffer: *mut i8, bufferSize: isize, encoding: u32) -> u8;
        fn CFRelease(cf: *mut c_void);
    }

    // Convert 1-based month layout to CoreFoundation's 0-indexed array layout
    let month_num = month.number();
    if !(1..=12).contains(&month_num) {
        return None;
    }
    let index = (month_num - 1) as isize;

    unsafe {
        let locale = CFLocaleCopyCurrent();
        if locale.is_null() {
            return None;
        }

        let formatter = CFDateFormatterCreate(std::ptr::null_mut(), locale, 0, 0);
        CFRelease(locale);
        if formatter.is_null() {
            return None;
        }

        // Fetch full month names array
        let symbols_array = CFDateFormatterCopyProperty(formatter, kCFDateFormatterMonthSymbols);
        CFRelease(formatter);
        if symbols_array.is_null() {
            return None;
        }

        let cf_string = CFArrayGetValueAtIndex(symbols_array, index);
        if cf_string.is_null() {
            CFRelease(symbols_array);
            return None;
        }

        let mut buffer = [0u8; 64];
        let success = CFStringGetCString(
            cf_string,
            buffer.as_mut_ptr() as *mut i8,
            buffer.len() as isize,
            0x08000100, // kCFStringEncodingUTF8
        );

        CFRelease(symbols_array);

        if success == 0 {
            return None;
        }

        let c_str = std::ffi::CStr::from_ptr(buffer.as_ptr() as *const i8);
        Some(c_str.to_string_lossy().into_owned())
    }
}

pub fn year_name(year: i32) -> String {
    #[repr(C)]
    struct CFGregorianDate {
        year: i32,
        month: i8,
        day: i8,
        hour: i8,
        minute: i8,
        second: f64,
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFLocaleCopyCurrent() -> *mut c_void;
        fn CFDateFormatterCreate(
            allocator: *mut c_void,
            locale: *mut c_void,
            dateStyle: isize,
            timeStyle: isize,
        ) -> *mut c_void;
        fn CFStringCreateWithBytes(
            alloc: *mut c_void,
            bytes: *const u8,
            numBytes: isize,
            encoding: u32,
            isExternalRepresentation: u8,
        ) -> *mut c_void;
        fn CFDateFormatterSetFormat(formatter: *mut c_void, formatString: *mut c_void);
        fn CFGregorianDateGetAbsoluteTime(gdate: CFGregorianDate, tz: *mut c_void) -> f64;
        fn CFDateFormatterCreateStringWithAbsoluteTime(
            allocator: *mut c_void,
            formatter: *mut c_void,
            at: f64,
        ) -> *mut c_void;
        fn CFStringGetCString(theString: *mut c_void, buffer: *mut i8, bufferSize: isize, encoding: u32) -> u8;
        fn CFRelease(cf: *mut c_void);
    }

    let gdate = CFGregorianDate {
        year,
        month: 1,
        day: 1,
        hour: 12, // Avoid timezone edge cases
        minute: 0,
        second: 0.0,
    };

    unsafe {
        let locale = CFLocaleCopyCurrent();
        if locale.is_null() {
            return year.to_string();
        }

        let formatter = CFDateFormatterCreate(std::ptr::null_mut(), locale, 0, 0);
        CFRelease(locale);
        if formatter.is_null() {
            return year.to_string();
        }

        let format_bytes = b"yyyy";
        let format_string = CFStringCreateWithBytes(
            std::ptr::null_mut(),
            format_bytes.as_ptr(),
            format_bytes.len() as isize,
            0x08000100, // kCFStringEncodingUTF8
            0,
        );

        if format_string.is_null() {
            CFRelease(formatter);
            return year.to_string();
        }

        CFDateFormatterSetFormat(formatter, format_string);
        CFRelease(format_string);

        let absolute_time = CFGregorianDateGetAbsoluteTime(gdate, std::ptr::null_mut());

        let result_string = CFDateFormatterCreateStringWithAbsoluteTime(std::ptr::null_mut(), formatter, absolute_time);
        CFRelease(formatter);

        if result_string.is_null() {
            return year.to_string();
        }

        let mut buffer = [0u8; 32];
        let success = CFStringGetCString(
            result_string,
            buffer.as_mut_ptr() as *mut i8,
            buffer.len() as isize,
            0x08000100, // kCFStringEncodingUTF8
        );
        CFRelease(result_string);

        if success == 0 {
            return year.to_string();
        }

        let c_str = std::ffi::CStr::from_ptr(buffer.as_ptr() as *const i8);
        c_str.to_string_lossy().into_owned()
    }
}