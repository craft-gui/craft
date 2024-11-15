
use oku::components::{Component, ComponentId, UpdateResult};
use oku::elements::text_input::TextInput;
use oku::engine::events::Message;
use oku::oku_main_with_options;
use oku::style::FlexDirection;
use oku::OkuOptions;
use oku_core::components::ComponentSpecification;
use oku_core::engine::events::{Event, OkuMessage};
use oku_core::RendererType::{Wgpu};

#[derive(Default, Copy, Clone)]
pub struct TextState {}

impl Component for TextState {
    type Props = ();

    fn view(
        _state: &Self,
        _props: &Self::Props,
        _children: Vec<ComponentSpecification>,
    ) -> ComponentSpecification {
        TextInput::new("Test").flex_direction(FlexDirection::Column).id("text_input").component()
    }

    fn update(
        state: &mut Self,
        _props: &Self::Props,
        event: Event,
    ) -> UpdateResult {
        println!("Source: {:?}", event.target);
        if let Message::OkuMessage(OkuMessage::TextInputChanged(new_val)) = event.message {
            println!("new text: {}", new_val);
        }

        UpdateResult::new()
    }
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    oku_main_with_options(
        TextState::component().into(),
        Some(OkuOptions {
            renderer: Wgpu,
            window_title: "text".to_string(),
        }),
    );
}
