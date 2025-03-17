#[path = "../util.rs"]
mod util;

use util::setup_logging;

use oku::components::ComponentSpecification;
use oku::components::{Component, UpdateResult};
use oku::elements::ElementStyles;
use oku::elements::TextInput;
use oku::elements::{Container, Font, Text};
use oku::events::Message;
use oku::events::{Event, OkuMessage};
use oku::oku_main_with_options;
use oku::resource_manager::ResourceIdentifier;
use oku::style::FlexDirection;
use oku::OkuOptions;
use oku::RendererType;

#[derive(Default, Copy, Clone)]
pub struct TextState {}

const FONT: &str =
    "https://github.com/google/material-design-icons/raw/refs/heads/master/variablefont/MaterialSymbolsOutlined%5BFILL%2CGRAD%2Copsz%2Cwght%5D.ttf";

impl Component for TextState {
    type Props = ();

    fn view_with_no_global_state(
        _state: &Self,
        _props: &Self::Props,
        _children: Vec<ComponentSpecification>,
    ) -> ComponentSpecification {
        Container::new()
            .flex_direction(FlexDirection::Row)
            .push(Text::new("Hello, World!").id("hello_text"))
            .push(Font::new(ResourceIdentifier::Url(FONT.to_string())))
            .push(Text::new("search home").font_family("Material Symbols Outlined").font_size(24.0))
            .push(TextInput::new("Test").flex_direction(FlexDirection::Column).id("text_input"))
            .push(Text::new("search home").font_family("Material Symbols Outlined").font_size(24.0))
            .component()
    }

    fn update_with_no_global_state(_state: &mut Self, _props: &Self::Props, event: Event) -> UpdateResult {
        println!("Source: {:?}", event.target);
        if let Message::OkuMessage(OkuMessage::TextInputChanged(new_val)) = event.message {
            println!("new text: {}", new_val);
        }

        UpdateResult::new()
    }
}

#[allow(dead_code)]
fn main() {
    setup_logging();

    oku_main_with_options(
        TextState::component(),
        Box::new(()),
        Some(OkuOptions {
            renderer: RendererType::default(),
            window_title: "text".to_string(),
        }),
    );
}
