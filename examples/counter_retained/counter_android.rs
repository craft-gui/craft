#![cfg(target_os = "android")]

use craft_retained::{AndroidApp, CraftOptions, craft_main};
use util::setup_logging;

use crate::counter::counter;

#[path = "main.rs"]
mod counter;

#[unsafe(no_mangle)]
pub unsafe fn android_main(app: AndroidApp) {
    let counter = counter();

    setup_logging();
    craft_main(counter, CraftOptions::basic("Counter"), app);
}
