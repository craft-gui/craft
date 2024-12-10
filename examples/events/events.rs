use oku::elements::ElementStyles;
use oku::RendererType::Wgpu;

use oku::components::{Component, ComponentSpecification, UpdateResult};
use oku::elements::Container;
use oku_core::events::{ButtonSource, ElementState, Message, MouseButton};
use oku::oku_main_with_options;
use oku::OkuOptions;
use oku_core::events::Event;
use oku_core::events::OkuMessage::PointerButtonEvent;
use oku_core::renderer::color::Color;
use oku::style::Unit;
use oku::style::Position;

#[derive(Default, Copy, Clone)]
pub struct EventsExample {
}

impl Component for EventsExample {
    type Props = ();

    fn view(
        _state: &Self,
        _props: &Self::Props,
        _children: Vec<ComponentSpecification>,
    ) -> ComponentSpecification {
        Container::new()
            .background(Color::RED)
            .width(Unit::Px(400.0))
            .height(Unit::Px(400.0))
            .id("red")
            .push(
                Container::new()
                    .background(Color::GREEN)
                    .inset(Unit::Px(50.0), Unit::Px(50.0), Unit::Px(50.0), Unit::Px(50.0))
                    .position(Position::Absolute)
                    .width(Unit::Px(200.0))
                    .height(Unit::Px(200.0))
                    .id("green")
                    .push(
                        Container::new()
                            .background(Color::BLUE)
                            .inset(Unit::Px(50.0), Unit::Px(50.0), Unit::Px(50.0), Unit::Px(50.0))
                            .position(Position::Absolute)
                            .width(Unit::Px(100.0))
                            .height(Unit::Px(100.0))
                            .id("blue"),
                    ),
            ).component()
    }

    fn update(
        _state: &mut Self,
        _props: &Self::Props,
        event: Event,
    ) -> UpdateResult {
        if let Message::OkuMessage(PointerButtonEvent(pointer_button)) = event.message {
            if pointer_button.button.mouse_button() == MouseButton::Left
                && pointer_button.state == ElementState::Pressed
            {
                println!("Target: {:?}, Current Target: {:?}", event.target, event.current_target);

            }
        };

        UpdateResult::new()
    }
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    oku_main_with_options(
        EventsExample::component(),
        Some(OkuOptions {
            renderer: Wgpu,
            window_title: "events".to_string(),
        }),
    );
}
