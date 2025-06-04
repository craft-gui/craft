use util::setup_logging;

use craft::components::Component;
use craft::components::ComponentId;
use craft::components::ComponentSpecification;
use craft::craft_main;
use craft::elements::ElementStyles;
use craft::elements::TextInput;
use craft::elements::{Container, Font, Text};
use craft::resource_manager::ResourceIdentifier;
use craft::style::Display::Block;
use craft::style::FlexDirection;
use craft::style::Overflow::Scroll;
use craft::style::Unit;
use craft::CraftOptions;
use craft::WindowContext;

#[derive(Default, Copy, Clone)]
pub struct TextState {}

const FONT: &str =
    "https://github.com/google/material-design-icons/raw/refs/heads/master/variablefont/MaterialSymbolsOutlined%5BFILL%2CGRAD%2Copsz%2Cwght%5D.ttf";

impl Component for TextState {
    type GlobalState = ();
    type Props = ();
    type Message = ();

    fn view(
        &self,
        _props: &Self::Props,
        _global_state: &Self::GlobalState,
        _children: Vec<ComponentSpecification>,
        _id: ComponentId,
        _window: &WindowContext,
    ) -> ComponentSpecification {
        Container::new()
            .height(Unit::Px(500.0))
            .display(Block)
            .flex_direction(FlexDirection::Row)
            .push(Text::new("Hello, World!"))
            .push(Font::new(ResourceIdentifier::Url(FONT.to_string())))
            .push(Text::new("search home").font_family("Material Symbols Outlined").font_size(24.0))
            .push(
                TextInput::new(include_str!("../../Cargo.lock"))
                    .height(Unit::Px(500.0))
                    .display(Block)
                    .overflow(Scroll),
            )
            .push(Text::new("search home").font_family("Material Symbols Outlined").font_size(24.0))
            .component()
    }
}

#[allow(dead_code)]
fn main() {
    setup_logging();
    craft_main(TextState::component(), (), CraftOptions::basic("Text"));
}
