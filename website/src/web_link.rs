use retgui::elements::{Container, Element, Elements};
use retgui::events::PointerButton;

#[allow(non_snake_case)]
pub fn WebLink(elements: &mut Elements, href: &str) -> Container {
    let href = href.to_string();

    Container::new(elements)
        .edit(elements)
        .on_pointer_button_up(move |event, _elements| {
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
