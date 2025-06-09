use crate::link::{Link, LinkProps};
use craft::components::{Component, ComponentSpecification, Props};
use craft::elements::{Container, ElementStyles, Text};
use craft::style::{AlignItems, Display, FlexDirection, JustifyContent, Overflow, Unit, Weight};
use craft::{palette, Color};
use crate::theme::{wrapper, WRAPPER_PADDING_LEFT, WRAPPER_PADDING_RIGHT};

fn hero_intro() -> ComponentSpecification {
    let craft_text = "A reactive GUI framework for Rust";

    wrapper()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .padding(Unit::Px(100.0), WRAPPER_PADDING_LEFT, Unit::Px(100.0), WRAPPER_PADDING_RIGHT)
        .push(Text::new(craft_text)
            .font_size(32.0)
            .font_weight(Weight::BOLD)
            .margin("0px", "0px", "8px", "0px")
        )
        .push(
            Text::new("Build your UI with regular Rust code.")
                .font_size(20.0)
                .color(Color::from_rgb8(60, 60, 60))
        )
        .push(
            Container::new()
                .margin("32px", "0px", "0px", "0px")
                .push(
                    Link::component()
                        .props(Props::new(LinkProps {
                            href: "https://github.com/craft-gui/craft".to_string(),
                        }))
                        .push(Text::new("GitHub")
                            .display(Display::Flex)
                            .align_items(AlignItems::Center)
                            .justify_content(JustifyContent::Center)
                            .font_size(18.0)
                            .min_width("100px")
                            .border_radius(8.0, 8.0, 8.0, 8.0)
                            .padding("8px", "20px", "8px", "20px")
                            .background(Color::from_rgb8(18, 14, 15))
                            .color(palette::css::WHITE))
                    ,
                )
        )
        .component()
}

fn hero_features() -> ComponentSpecification {
    fn hero_item(title: &str, text: &str) -> Container {
        let sub_title_color = Color::from_rgb8(70, 70,70);
        
        Container::new()
            .gap("10px")
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .push(Text::new(title).font_size(24.0))
            .push(Text::new(text).font_size(16.0).color(sub_title_color))
    }
    
    Container::new()
        .background(Color::from_rgb8(177, 200, 211))
        .width("100%")
        .push(
            wrapper()
                .padding(Unit::Px(100.0), WRAPPER_PADDING_LEFT, Unit::Px(100.0), WRAPPER_PADDING_RIGHT)
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .gap("20px")
                .push(Text::new("Goals").font_size(32.0).font_weight(Weight::SEMIBOLD).margin("0px", "0px", "10px", "0px"))
                .push(hero_item("1. Reactive", "When your data changes, we automatically re-run your view function."))
                .push(hero_item("2. Components", "Components are reusable blocks that manage their own state and define both how they are rendered and how they respond to updates."))
                .push(hero_item("3. Pure Rust without macros", "No macros."))
                .push(hero_item("4. Web-like styling", "We use Taffy, an implementation of the CSS flexbox, block, and grid layout algorithms, for simple and familiar styling."))
                .push(hero_item("5. Cross Platform", "Currently we support Windows, macOS, Linux, Web, and Android."))

        )
        .component()
}

pub(crate) fn index_page() -> ComponentSpecification {
    Container::new()
        .width("100%")
        .overflow(Overflow::Scroll)
        .push(
            Container::new()
                .display(Display::Flex)
                .width("100%")
                .margin(Unit::Px(0.0), Unit::Auto, Unit::Px(0.0), Unit::Auto)
                .flex_direction(FlexDirection::Column)
                .flex_grow(1.0)
                .push(hero_intro())
                .push(hero_features())
        ) .component()
}
