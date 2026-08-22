use retgui::elements::{Container, Element, Text, Window};
use retgui::style::{AlignItems, FlexDirection, JustifyContent};
use retgui::{pct, px, retgui_main, rgb, Color, RetGuiOptions};

use util::setup_logging;

struct Greeting {
    name: String,
}

pub fn custom_event() -> Container {
    let message = Text::new("No event received yet");

    let message_clone = message.clone();
    let receiver = Container::new()
        .on_custom_event(move |event| {
            let message_clone = message_clone.clone();
            if let Some(greeting) = event.data::<Greeting>() {
                message_clone.text(&format!("Hello, {}!", greeting.name));
            }
        })
        .push(message);

    let receiver_clone = receiver.clone();
    let button = Container::new()
        .padding(px(12), px(20), px(12), px(20))
        .background_color(rgb(59, 130, 246))
        .on_click(move |_| {
            receiver_clone.emit_custom_event(Greeting {
                name: "Mary".to_string(),
            });
        })
        .push(Text::new("Send custom event").color(Color::WHITE));

    Container::new()
        .flex_direction(FlexDirection::Column)
        .justify_content(JustifyContent::Center)
        .align_items(AlignItems::Center)
        .width(pct(100))
        .height(pct(100))
        .row_gap(px(20))
        .push(receiver)
        .push(button)
}

pub fn main() {
    setup_logging();

    Window::new("Custom Event")
        .width(pct(100))
        .height(pct(100))
        .push(custom_event());

    retgui_main(RetGuiOptions::basic("Custom Event"));
}
