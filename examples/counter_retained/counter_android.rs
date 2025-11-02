#![cfg(target_os = "android")]

use craft::components::Component;
use craft::{craft_main, AndroidApp, CraftOptions};
use util::setup_logging;

#[path = "main.rs"]
mod counter;
use counter::Counter;

#[unsafe(no_mangle)]
pub unsafe fn android_main(app: AndroidApp) {
    setup_logging();
    craft_main(Counter::component(), (), CraftOptions::basic("Counter"), app);
}
