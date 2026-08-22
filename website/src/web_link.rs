use retgui::elements::{Container, Element};
use retgui::events::ui_events::pointer::PointerButton;

#[allow(non_snake_case)]
pub fn WebLink(href: &str) -> Container {
    let href = href.to_string();

    Container::new().on_pointer_button_up(move |event| {
        if event.button == Some(PointerButton::Primary) {
            #[cfg(target_arch = "wasm32")]
            {
                if let Some(win) = web_sys::window() {
                    // Use the captured owned string
                    let _ = win.open_with_url(&href);
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                open::that(&href).unwrap();
            }
        }
    })
}
