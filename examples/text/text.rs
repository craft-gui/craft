
use oku::components::{Component, UpdateResult};
use oku::elements::text_input::TextInput;
use oku::engine::events::Message;
use oku::oku_main_with_options;
use oku::style::FlexDirection;
use oku::OkuOptions;
use oku::components::ComponentSpecification;
use oku::elements::ElementStyles;
use oku::engine::events::{Event, OkuMessage};
use oku::RendererType::{Wgpu};
use oku_core::RendererType::Vello;

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
        _state: &mut Self,
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
            renderer: Vello,
            window_title: "text".to_string(),
        }),
    );
}
