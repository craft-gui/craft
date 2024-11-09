use oku::RendererType::Wgpu;

use oku::engine::events::{Message};
use oku_core::components::ComponentSpecification;
use oku::oku_main_with_options;
use oku::components::{Component, ComponentId, UpdateResult};
use oku::style::{FlexDirection};
use oku::OkuOptions;
use oku::elements::text_input::TextInput;

#[derive(Default, Copy, Clone)]
pub struct TextState {
}

impl Component for TextState {
    type Props = ();

    fn view(
        state: &Self,
        _props: Option<&Self::Props>,
        _children: Vec<ComponentSpecification>,
        _id: ComponentId,
    ) -> ComponentSpecification {
        ComponentSpecification {
            component: TextInput::new("Test").flex_direction(FlexDirection::Column).into(),
            key: None,
            props: None,
            children: Vec::new(),
        }
    }

    fn update(state: &mut Self, _id: ComponentId, message: Message, source_element: Option<String>) -> UpdateResult {
        UpdateResult::new(true, None)
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
