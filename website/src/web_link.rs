use retgui::App;
use retgui::elements::{Container, Element};
use retgui::events::PointerButton;

#[allow(non_snake_case)]
pub fn WebLink(app: &mut App, href: &str) -> Container {
    let href = href.to_string();

    Container::new(app)
        .edit(app)
        .add_pointer_button_up_listener(move |event, _app| {
            if event.button == Some(PointerButton::Left) {
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
        .finish()
}
