#[cfg(windows)]
use windows::{Win32::Globalization::{GetLocaleInfoEx, LOCALE_IFIRSTDAYOFWEEK}, core::PCWSTR};

include!(concat!(env!("OUT_DIR"), "/week_data.rs"));

/// Returns the first day of the week for a given territory code (e.g., "US", "GB").
///
/// Territory codes should be provided as uppercase ISO 3166-1 alpha-2 codes.
/// If the territory is not found, it defaults to the world standard ("001").
pub fn first_day_for_territory(territory: &str) -> &'static str {
    FIRST_DAY
        .iter()
        .find(|(t, _)| *t == territory)
        .map(|(_, day)| *day)
        .unwrap_or_else(|| {
            FIRST_DAY
                .iter()
                .find(|(t, _)| *t == "001")
                .map(|(_, day)| *day)
                .unwrap_or("mon")
        })
}

/// Returns the first day of the week for the current system user.
///
/// This queries the native OS APIs where possible (Windows, Apple, WebAssembly),
/// and falls back to detecting the system locale (Android, Linux) to query the CLDR table.
pub fn system_first_day_of_week() -> &'static str {
    #[cfg(windows)]
    {
        if let Some(day_val) = windows_first_day_of_week() {
            return match day_val {
                0 => "mon",
                1 => "tue",
                2 => "wed",
                3 => "thu",
                4 => "fri",
                5 => "sat",
                6 => "sun",
                _ => first_day_for_territory("001"),
            };
        }
    }

    #[cfg(target_vendor = "apple")]
    {
        if let Some(day_val) = apple_first_day_of_week() {
            return match day_val {
                1 => "sun",
                2 => "mon",
                3 => "tue",
                4 => "wed",
                5 => "thu",
                6 => "fri",
                7 => "sat",
                _ => first_day_for_territory("001"),
            };
        }
    }

    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    {
        if let Some(locale) = wasm_browser_locale() {
            if let Some(day) = lookup_locale_string(&locale) {
                return day;
            }
        }
    }

    #[cfg(any(
        target_os = "android",
        target_os = "linux",
        target_os = "freebsd",
        target_os = "openbsd"
    ))]
    {
        if let Some(locale) = sys_locale::get_locale()
            && let Some(day) = lookup_locale_string(&locale)
        {
            return day;
        }
    }
    first_day_for_territory("001")
}

#[cfg(any(
    all(target_arch = "wasm32", target_os = "unknown"),
    target_os = "android",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "openbsd"
))]
/// Helper function to parse standard locale strings (e.g., "en-US", "en_US.UTF-8")
fn lookup_locale_string(locale: &str) -> Option<&'static str> {
    let upper_locale = locale.to_uppercase();
    for part in upper_locale.split(&['-', '_', '.'][..]).rev() {
        if let Some((_, day)) = FIRST_DAY.iter().find(|(t, _)| *t == part) {
            return Some(day);
        }
    }
    None
}

#[cfg(windows)]
fn windows_first_day_of_week() -> Option<u32> {
    let mut buf = [0u16; 8];

    let len = unsafe { GetLocaleInfoEx(PCWSTR::null(), LOCALE_IFIRSTDAYOFWEEK, Some(&mut buf)) };

    if len == 0 {
        return None;
    }

    let value = String::from_utf16_lossy(&buf[..len as usize - 1]);

    value.parse::<u32>().ok()
}

#[cfg(target_vendor = "apple")]
fn apple_first_day_of_week() -> Option<u32> {
    use std::ffi::c_void;

    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFCalendarCopyCurrent() -> *mut c_void;
        fn CFCalendarGetFirstWeekday(calendar: *mut c_void) -> isize;
        fn CFRelease(cf: *mut c_void);
    }

    unsafe {
        let cal = CFCalendarCopyCurrent();
        if cal.is_null() {
            return None;
        }

        let day = CFCalendarGetFirstWeekday(cal);
        CFRelease(cal);

        Some(day as u32)
    }
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
fn wasm_browser_locale() -> Option<String> {
    web_sys::window().and_then(|win| win.navigator().language())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monday_start() {
        assert_eq!(first_day_for_territory("DE"), "mon");
        assert_eq!(first_day_for_territory("FR"), "mon");
    }

    #[test]
    fn test_sunday_start() {
        assert_eq!(first_day_for_territory("US"), "sun");
        assert_eq!(first_day_for_territory("CA"), "sun");
        assert_eq!(first_day_for_territory("JP"), "sun");
    }

    #[test]
    fn test_saturday_start() {
        assert_eq!(first_day_for_territory("EG"), "sat");
        assert_eq!(first_day_for_territory("AF"), "sat");
    }

    #[test]
    fn test_friday_start() {
        assert_eq!(first_day_for_territory("MV"), "fri");
    }

    #[test]
    fn test_gb_variant_fix() {
        assert_eq!(first_day_for_territory("GB"), "mon");
    }

    #[test]
    fn test_fallback_logic() {
        assert_eq!(first_day_for_territory("XX"), "mon");
        assert_eq!(first_day_for_territory(""), "mon");
    }

    #[test]
    fn test_world_default() {
        assert_eq!(first_day_for_territory("001"), "mon");
    }

    #[test]
    fn test_system_resolution() {
        let day = system_first_day_of_week();
        assert!(["mon", "tue", "wed", "thu", "fri", "sat", "sun"].contains(&day));
    }
}
