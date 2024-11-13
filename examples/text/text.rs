
use oku::components::{Component, ComponentId, UpdateResult};
use oku::elements::text_input::TextInput;
use oku::engine::events::Message;
use oku::oku_main_with_options;
use oku::style::FlexDirection;
use oku::OkuOptions;
use oku_core::components::ComponentSpecification;
use oku_core::engine::events::OkuEvent;
use oku_core::RendererType::Software;

#[derive(Default, Copy, Clone)]
pub struct TextState {}

impl Component for TextState {
    type Props = ();

    fn view(
        _state: &Self,
        _props: Option<&Self::Props>,
        _children: Vec<ComponentSpecification>,
        _id: ComponentId,
    ) -> ComponentSpecification {
        TextInput::new("Test").flex_direction(FlexDirection::Column).id("text_input").component()
    }

    fn update(
        state: &mut Self,
        _props: Option<&Self::Props>,
        _id: ComponentId,
        message: Message,
        source_element: Option<String>,
    ) -> UpdateResult {
        println!("Source: {:?}", source_element);
        if let Message::OkuMessage(OkuEvent::TextInputChanged(new_val)) = message {
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
            renderer: Software,
            window_title: "text".to_string(),
        }),
    );
}
