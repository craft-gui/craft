#![cfg(target_os = "android")]

use crate::counter::counter;
use craft_retained::{craft_main, AndroidApp, CraftOptions};
use util::setup_logging;

#[path = "main.rs"]
mod counter;

#[unsafe(no_mangle)]
pub unsafe fn android_main(app: AndroidApp) {
    let counter = counter();

    setup_logging();
    craft_main(counter, CraftOptions::basic("Counter"), app);
}
