#[path = "../util.rs"]
mod util;

use util::setup_logging;

use oku::components::ComponentSpecification;
use oku::components::{Component, UpdateResult};
use oku::elements::ElementStyles;
use oku::elements::TextInput;
use oku::elements::{Container, Font, Text};
use oku::events::Event;
use oku::oku_main_with_options;
use oku::resource_manager::ResourceIdentifier;
use oku::style::Display::Block;
use oku::style::FlexDirection;
use oku::style::Overflow::Scroll;
use oku::style::Unit;
use oku::OkuOptions;
use oku::RendererType;
use oku::components::ComponentId;

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
        _id: ComponentId,
    ) -> ComponentSpecification {
        Container::new()
            .height(Unit::Px(500.0))
            .display(Block)
            .flex_direction(FlexDirection::Row)
            .push(Text::new("Hello, World!").id("hello_text"))
            .push(Font::new(ResourceIdentifier::Url(FONT.to_string())))
            .push(Text::new("search home").font_family("Material Symbols Outlined").font_size(24.0))
            .push(TextInput::new(include_str!("../../Cargo.lock")).height(Unit::Px(500.0)).display(Block).overflow(Scroll).id("text_input"))
            .push(Text::new("search home").font_family("Material Symbols Outlined").font_size(24.0))
            .component()
    }

    fn update_with_no_global_state(_state: &mut Self, _props: &Self::Props, _event: Event) -> UpdateResult {
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
