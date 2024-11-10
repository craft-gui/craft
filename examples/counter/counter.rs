use oku::RendererType::Wgpu;

use oku::components::{Component, ComponentId, ComponentSpecification, UpdateResult};
use oku::elements::{Container, Text};
use oku::engine::events::OkuEvent::PointerButtonEvent;
use oku::engine::events::{ButtonSource, ElementState, Message, MouseButton};
use oku::oku_main_with_options;
use oku::style::FlexDirection;
use oku::OkuOptions;

#[derive(Default, Copy, Clone)]
pub struct Counter {
    count: u64,
}

impl Component for Counter {
    type Props = ();

    fn view(
        state: &Self,
        _props: Option<&Self::Props>,
        _children: Vec<ComponentSpecification>,
        _id: ComponentId,
    ) -> ComponentSpecification {
        Container::new()
            .flex_direction(FlexDirection::Column)
            .component()
            .push(Text::new(format!("Counter: {}", state.count).as_str()).component())
            .push(Container::new().component())
            .push(Text::new("increment").id("increment").component())
    }

    fn update(state: &mut Self, _id: ComponentId, message: Message, source_element: Option<String>) -> UpdateResult {
        if source_element.as_deref() != Some("increment") {
            return UpdateResult::default();
        }

        if let Message::OkuMessage(PointerButtonEvent(pointer_button)) = message {
            if pointer_button.button == ButtonSource::Mouse(MouseButton::Left)
                && pointer_button.state == ElementState::Pressed
            {
                state.count += 1
            }
        };

        UpdateResult::new(true, None)
    }
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    oku_main_with_options(
        Counter::component(),
        Some(OkuOptions {
            renderer: Wgpu,
            window_title: "counter".to_string(),
        }),
    );
}
