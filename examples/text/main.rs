use util::setup_logging;

use craft::components::ComponentId;
use craft::components::ComponentSpecification;
use craft::components::{Component, Event};
use craft::elements::TextInput;
use craft::elements::{Container, Font, Text};
use craft::elements::ElementStyles;
use craft::resource_manager::ResourceIdentifier;
use craft::style::Display::Block;
use craft::style::Overflow::Scroll;
use craft::style::Unit;
use craft::style::{FlexDirection, FontStyle, TextStyleProperty, Weight};
use craft::text::RangedStyles;
use craft::CraftOptions;
use craft::WindowContext;
use craft::{craft_main, rgb};

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
        let text = "Rich text includes color, bold, italic, links, underline, and background.";
        let mut rich_text = TextInput::new(text)
            .border_width(0, 0, 0, 0)
            .disabled()
            .on_link_clicked(
                move |_: &mut Self, _d: &mut (), event: &mut Event, link: &str| {
                    println!("Link clicked: {}", link);
            });

        let ranged_styles = vec![
            (19..24, TextStyleProperty::Color(rgb(255, 0, 0))),
            (26..30, TextStyleProperty::FontWeight(Weight::BOLD)),
            (32..38, TextStyleProperty::FontStyle(FontStyle::Italic)),
            (40..45, TextStyleProperty::Link("craftgui.com".to_string())),
            (47..56, TextStyleProperty::UnderlineBrush(rgb(255, 0, 0))),
            (47..56, TextStyleProperty::UnderlineSize(1.0)),
            (47..56, TextStyleProperty::UnderlineOffset(-1.0)),
            (47..56, TextStyleProperty::Underline(true)),
            (62..72, TextStyleProperty::BackgroundColor(rgb(200, 200, 200))),
        ];
        let ranged_styles = RangedStyles::new(ranged_styles);
        rich_text.ranged_styles = Some(ranged_styles);


        Container::new()
            .height(Unit::Px(500.0))
            .display(Block)
            .flex_direction(FlexDirection::Row)
            .push(Text::new("Hello, World!"))
            .push(rich_text)
            .push(Font::new(ResourceIdentifier::Url(FONT.to_string())))
            .push(Text::new("search home").font_family("Material Symbols Outlined").font_size(24.0))
            .push(
                TextInput::new(include_str!("../counter/main.rs"))
                    .height(Unit::Px(600.0))
                    .width(Unit::Px(800.0))
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
