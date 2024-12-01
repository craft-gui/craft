use oku::components::{Component, ComponentSpecification, UpdateResult};
use oku::elements::{Container, Text};
use oku::engine::events::{ButtonSource, ElementState, Event, Message, MouseButton};
use oku::oku_main_with_options;
use oku::style::{AlignItems, FlexDirection, JustifyContent};
use oku::OkuOptions;

use oku_core::elements::ElementStyles;
use oku_core::engine::events::OkuMessage::PointerButtonEvent;
use oku_core::engine::renderer::color::Color;
use oku_core::style::{Display, Unit};
use oku_core::RendererType::Vello;

#[derive(Default, Copy, Clone)]
pub struct Counter {
    count: i64,
}

impl Component for Counter {
    type Props = ();

    fn view(state: &Self, _props: &Self::Props, _children: Vec<ComponentSpecification>) -> ComponentSpecification {
        Container::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .justify_content(JustifyContent::Center)
            .align_items(AlignItems::Center)
            .width(Unit::Percentage(100.0))
            .height(Unit::Percentage(100.0))
            .background(Color::rgba(250, 250, 250, 255))
            .gap(Unit::Px(20.0))
            .push(
                Text::new(format!("{}", state.count).as_str())
                    .font_size(72.0)
                    .color(Color::rgba(50, 50, 50, 255)),
            )
            .push(
                Container::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Row)
                    .gap(Unit::Px(20.0))
                    .push(create_button("-", "decrement", Color::rgba(244, 67, 54, 255)))
                    .push(create_button("+", "increment", Color::rgba(76, 175, 80, 255))),
            )
            .component()
    }

    fn update(state: &mut Self, _props: &Self::Props, message: Event) -> UpdateResult {
        if let Some(target) = message.target.as_deref() {
            if let Message::OkuMessage(PointerButtonEvent(pointer_button)) = message.message {
                if pointer_button.button == ButtonSource::Mouse(MouseButton::Left)
                    && pointer_button.state == ElementState::Pressed
                {
                    match target {
                        "increment" => state.count += 1,
                        "decrement" => state.count -= 1,
                        _ => return UpdateResult::default(),
                    }
                    return UpdateResult::new().prevent_propagate();
                }
            }
        }
        UpdateResult::default()
    }
}

fn create_button(label: &str, id: &str, color: Color) -> ComponentSpecification {
    Container::new()
        .padding(Unit::Px(15.0), Unit::Px(30.0), Unit::Px(15.0), Unit::Px(30.0))
        .background(color)
        .display(Display::Flex)
        .justify_content(JustifyContent::Center)
        .align_items(AlignItems::Center)
        .push(
            Text::new(label)
                .id(id)
                .font_size(24.0)
                .color(Color::WHITE)
                .width(Unit::Percentage(100.0))
                .height(Unit::Percentage(100.0)),
        )
        .id(id)
        .component()
}

#[cfg(not(target_os = "android"))]
fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    oku_main_with_options(
        Counter::component(),
        Some(OkuOptions {
            renderer: Vello,
            window_title: "Counter".to_string(),
        }),
    );
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: AndroidApp) {
    #[cfg(not(target_arch = "wasm32"))]
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    oku_main_with_options(
        Counter::component(),
        Some(OkuOptions {
            renderer: Vello,
            window_title: "Counter".to_string(),
        }),
        app,
    );
}
