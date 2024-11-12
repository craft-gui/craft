use oku::components::ComponentSpecification;
use oku::elements::{Container, Text};

use oku::RendererType::Wgpu;
use oku::{oku_main_with_options, OkuOptions};

use oku::engine::events::OkuEvent::PointerButtonEvent;
use oku::engine::events::{ButtonSource, ElementState, Message, MouseButton};
use oku::components::{Component, ComponentId, UpdateResult};
use oku::style::FlexDirection;

#[derive(Default, Copy, Clone)]
pub struct Accordion {
    show_content: bool,
}

impl Component for Accordion {
    type Props = ();

    fn view(
        state: &Self,
        _props: Option<&Self::Props>,
        _children: Vec<ComponentSpecification>,
        id: ComponentId,
    ) -> ComponentSpecification {
        let accordion_content =
            if state.show_content { Text::new("My content!").component() } else { Container::new().component() };

        Container::new()
            .margin(14.0, 0.0, 0.0, 14.0)
            .flex_direction(FlexDirection::Column)
            .component()
            .push(
                Container::new()
                    .id("accordion_header")
                    .component()
                    .push(Text::new("Accordion Example").id("accordion_header").component()),
            )
            .push(accordion_content)
    }

    fn update(state: &mut Self, _props: Option<&Self::Props>, id: ComponentId, message: Message, source_element: Option<String>) -> UpdateResult {
        if source_element.as_deref() != Some("accordion_header") {
            return UpdateResult::default();
        }

        if let Message::OkuMessage(PointerButtonEvent(pointer_button)) = message {
            if pointer_button.button == ButtonSource::Mouse(MouseButton::Left)
                && pointer_button.state == ElementState::Pressed
            {
                state.show_content = !state.show_content
            }
        }

        UpdateResult::new()
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE) // Set the maximum log level you want to capture
        .init();

    oku_main_with_options(
        Container::new().component().children(vec![Accordion::component(), Accordion::component()]),
        Some(OkuOptions {
            renderer: Wgpu,
            window_title: "accordion".to_string(),
        }),
    );
}
