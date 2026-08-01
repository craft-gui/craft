#![cfg(target_os = "android")]

use retgui_retained::{AndroidApp, retgui_set_android_app};

use crate::counter::main;

#[path = "main.rs"]
mod counter;

#[unsafe(no_mangle)]
pub unsafe fn android_main(app: AndroidApp) {
    retgui_set_android_app(app);
    main();
}
