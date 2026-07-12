use icu::locale::Locale;

pub fn get_locale_string() -> Option<String> {
    sys_locale::get_locale()
}

pub fn get_locale_string_or_default() -> String {
    get_locale_string().unwrap_or(String::from("en-US"))
}

pub fn get_locale_or_default() -> Locale {
    Locale::try_from_str(&get_locale_string_or_default()).expect("Failed to parse system locale")
}