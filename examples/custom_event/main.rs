use retgui::elements::{Container, Element, Elements, Text, Window};
use retgui::style::{AlignItems, FlexDirection, JustifyContent};
use retgui::{Color, RetGuiOptions, pct, px, retgui_main, rgb};

use util::setup_logging;

struct Greeting {
    name: String,
}

pub fn custom_event(elements: &mut Elements) -> Container {
    let message = Text::new(elements, "No event received yet");

    let receiver = Container::new(elements)
        .edit(elements)
        .on_custom_event(move |event, elements| {
            if let Some(greeting) = event.data::<Greeting>() {
                message
                    .edit(elements)
                    .text(&format!("Hello, {}!", greeting.name))
                    .finish();
            }
        })
        .push(message)
        .finish();

    let button_label = Text::new(elements, "Send custom event")
        .edit(elements)
        .color(Color::WHITE)
        .finish();
    let button = Container::new(elements)
        .edit(elements)
        .padding(px(12), px(20), px(12), px(20))
        .background_color(rgb(59, 130, 246))
        .on_click(move |_event, elements| {
            receiver.emit_custom_event(
                elements,
                Greeting {
                    name: "Mary".to_string(),
                },
            );
        })
        .push(button_label)
        .finish();

    Container::new(elements)
        .edit(elements)
        .flex_direction(FlexDirection::Column)
        .justify_content(JustifyContent::Center)
        .align_items(AlignItems::Center)
        .width(pct(100))
        .height(pct(100))
        .row_gap(px(20))
        .push(receiver)
        .push(button)
        .finish()
}

pub fn main() {
    setup_logging();
    let mut elements = Elements::new();
    let content = custom_event(&mut elements);
    Window::new(&mut elements, "Custom Event")
        .edit(&mut elements)
        .width(pct(100))
        .height(pct(100))
        .push(content)
        .finish();
    retgui_main(elements, RetGuiOptions::basic("Custom Event"));
}
