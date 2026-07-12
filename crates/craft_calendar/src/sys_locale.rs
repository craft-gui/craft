pub fn get_locale() -> Option<String> {
    sys_locale::get_locale()
}

pub fn get_locale_or_default() -> String {
    get_locale().unwrap_or(String::from("en-US"))
}