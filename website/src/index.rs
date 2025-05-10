use crate::link::{Link, LinkProps};
use craft::components::{Component, ComponentSpecification, Props};
use craft::elements::{Container, ElementStyles, Text};
use craft::style::{AlignItems, Display, FlexDirection, JustifyContent, Overflow, Unit, Weight};
use craft::{palette, Color};

fn hero_intro() -> ComponentSpecification {
    #[cfg(not(target_arch = "wasm32"))]
    let craft_text = "📜 Craft";

    // FIXME: Emojis aren't showing on the web.
    #[cfg(target_arch = "wasm32")]
    let craft_text = "Craft";

    Container::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .margin("100px", "5%", "100px", "5%")
        .width("600px")
        .min_width("330px")
        .max_width("100%")
        .push(Text::new(craft_text)
            .font_size(32.0)
            .font_weight(Weight::BOLD)
            .margin("0px", "0px", "8px", "0px")
        )
        .push(
            Text::new("A reactive GUI framework that allows developers to build interactive graphical user interfaces efficiently and elegantly.")
                .font_size(24.0)
            ,
        )
        .push(
            Container::new()
                .margin("16px", "0px", "0px", "0px")
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

    Container::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .padding("5%", "5%", "0%", "5%")
        .width("100%")
        .gap("50px")
        .push(

            Container::new()
                .gap("10px")
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .push(Text::new("Goals:").font_size(32.0)
                    .font_weight(Weight::SEMIBOLD).margin("0px", "0px", "10px", "0px"))
                .push(Text::new("1. Reactive").font_size(24.0))
                .push(Text::new("When your data changes, we automatically re-run your view function.").font_size(16.0).color(palette::css::GRAY))
                .push(Text::new("2. Components").font_size(24.0))
                .push(Text::new("Components are reusable blocks that manage their own state and define both how they are rendered and how they respond to updates.").font_size(16.0).color(palette::css::GRAY))
                .push(Text::new("3. Pure Rust without macros").font_size(24.0))
                .push(Text::new("No macros.").font_size(16.0).color(palette::css::GRAY))
                .push(Text::new("4. Web-like styling").font_size(24.0))
                .push(Text::new("We use Taffy, an implementation of the CSS flexbox, block, and grid layout algorithms, for simple and familiar styling.").font_size(16.0).color(palette::css::GRAY))
                .push(Text::new("5. Cross Platform").font_size(24.0))
                .push(Text::new("Currently we support Windows, macOS, Linux, Web, and Android.").font_size(16.0).color(palette::css::GRAY))

        )
        .component()
}

pub(crate) fn index_page() -> ComponentSpecification {
    Container::new()
        .display(Display::Flex)
        .width("100%")
        .flex_direction(FlexDirection::Column)
        .flex_grow(1.0) // Take up remaining space
        .overflow(Overflow::Scroll) // Allow content overflow to scroll
        .padding(Unit::Px(20.0), Unit::Px(20.0), Unit::Percentage(10.0), Unit::Px(20.0))
        .push(hero_intro())
        .push(hero_features())
        .component()
}
