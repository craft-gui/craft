use std::rc::Rc;

use retgui::elements::{Container, Element, Elements};
use retgui::events::PointerButton;

#[allow(non_snake_case)]
pub fn Link<F>(elements: &mut Elements, on_click: F) -> Container
where
    F: Fn(&mut Elements) + 'static,
{
    let on_click = Rc::new(on_click);

    Container::new(elements)
        .edit(elements)
        .on_pointer_button_up(move |event, elements| {
            if event.button == Some(PointerButton::Left) {
                on_click(elements);
            }
        })
        .finish()
}
