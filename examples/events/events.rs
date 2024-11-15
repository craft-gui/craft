use oku::RendererType::Wgpu;

use oku::components::{Component, ComponentId, ComponentSpecification, UpdateResult};
use oku::elements::Container;
use oku::engine::events::{ButtonSource, ElementState, Message, MouseButton};
use oku::oku_main_with_options;
use oku::OkuOptions;
use oku_core::engine::events::Event;
use oku_core::engine::events::OkuMessage::PointerButtonEvent;
use oku_core::engine::renderer::color::Color;
use oku_core::style::Unit;

#[derive(Default, Copy, Clone)]
pub struct Counter {
    count: u64,
}

impl Component for Counter {
    type Props = ();

    fn view(
        state: &Self,
        _props: &Self::Props,
        _children: Vec<ComponentSpecification>,
    ) -> ComponentSpecification {
        Container::new()
            .background(Color::RED)
            .width(Unit::Px(400.0))
            .height(Unit::Px(400.0))
            .id("red")
            .component()
            .push(
                Container::new()
                    .background(Color::BLUE)
                    .width(Unit::Px(200.0))
                    .height(Unit::Px(200.0))
                    .id("blue")
                    .component()
                    .push(
                        Container::new()
                            .background(Color::GREEN)
                            .width(Unit::Px(100.0))
                            .height(Unit::Px(100.0))
                            .id("green")
                            .component(),
                    ),
            )
    }

    fn update(
        state: &mut Self,
        _props: &Self::Props,
        event: Event,
    ) -> UpdateResult {
        println!("Update: {:?}", event.target);

        if event.target.as_deref() == Some("blue") {
            return UpdateResult::new().prevent_propagate();
        }

        if let Message::OkuMessage(PointerButtonEvent(pointer_button)) = event.message {
            if pointer_button.button == ButtonSource::Mouse(MouseButton::Left)
                && pointer_button.state == ElementState::Pressed
            {
                state.count += 1
            }
        };

        UpdateResult::new()
    }
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    oku_main_with_options(
        Counter::component(),
        Some(OkuOptions {
            renderer: Wgpu,
            window_title: "events".to_string(),
        }),
    );
}
