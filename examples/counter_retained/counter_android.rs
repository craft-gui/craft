#![cfg(target_os = "android")]

use craft_retained::{AndroidApp, craft_set_android_app};

use crate::counter::main;

#[path = "main.rs"]
mod counter;

#[unsafe(no_mangle)]
pub unsafe fn android_main(app: AndroidApp) {
    craft_set_android_app(app);
    main();
}
