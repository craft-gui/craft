#![cfg(target_os = "android")]

use craft::{AndroidApp, craft_main, CraftOptions};
use craft::components::Component;
use util::setup_logging;

#[path = "main.rs"]
mod counter;
use counter::Counter;

#[unsafe(no_mangle)]
pub unsafe fn android_main(app: AndroidApp) {
    setup_logging();
    craft_main(
        Counter::component(),
        (),
        CraftOptions::basic("Counter"),
        app,
    );
}
