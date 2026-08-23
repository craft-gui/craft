use std::rc::Rc;

use retgui::elements::{Container, Element};
use retgui::events::PointerButton;

#[allow(non_snake_case)]
pub fn Link<F>(on_click: F) -> Container
where
    F: Fn() + 'static,
{
    let on_click = Rc::new(on_click);

    Container::new().on_pointer_button_up(move |event| {
        if event.button == Some(PointerButton::Left) {
            on_click();
        }
    })
}
