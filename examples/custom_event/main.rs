use retgui::elements::{Container, Element, Text, Window};
use retgui::style::{AlignItems, FlexDirection, JustifyContent};
use retgui::{App, Color, RetGuiOptions, pct, px, retgui_main, rgb};

use util::setup_logging;

struct Greeting {
    name: String,
}

pub fn custom_event(app: &mut App) -> Container {
    let message = Text::new(app, "No event received yet");

    let receiver = Container::new(app)
        .edit(app)
        .add_custom_event_listener(move |event, app| {
            if let Some(greeting) = event.data::<Greeting>() {
                message.edit(app).text(&format!("Hello, {}!", greeting.name)).finish();
            }
        })
        .push(message)
        .finish();

    let button_label = Text::new(app, "Send custom event")
        .edit(app)
        .color(Color::WHITE)
        .finish();
    let button = Container::new(app)
        .edit(app)
        .padding(px(12), px(20), px(12), px(20))
        .background_color(rgb(59, 130, 246))
        .add_click_listener(move |_event, app| {
            receiver.emit_custom_event(
                app,
                Greeting {
                    name: "Mary".to_string(),
                },
            );
        })
        .push(button_label)
        .finish();

    Container::new(app)
        .edit(app)
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
    let mut app = App::new();
    let content = custom_event(&mut app);
    Window::new(&mut app, "Custom Event")
        .edit(&mut app)
        .width(pct(100))
        .height(pct(100))
        .push(content)
        .finish();
    retgui_main(app, RetGuiOptions::basic("Custom Event"));
}
