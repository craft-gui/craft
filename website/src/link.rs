use std::rc::Rc;

use retgui::App;
use retgui::elements::{Container, Element};
use retgui::events::PointerButton;

#[allow(non_snake_case)]
pub fn Link<F>(app: &mut App, on_click: F) -> Container
where
    F: Fn(&mut App) + 'static,
{
    let on_click = Rc::new(on_click);

    Container::new(app)
        .edit(app)
        .add_pointer_button_up_listener(move |event, app| {
            if event.button == Some(PointerButton::Left) {
                on_click(app);
            }
        })
        .finish()
}
