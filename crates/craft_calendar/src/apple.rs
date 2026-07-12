use std::ffi::c_void;

use icu::calendar::types::Weekday;

fn first_day_of_week() -> Option<Weekday> {
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
    extern "C" {
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
            _ => return None,
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
