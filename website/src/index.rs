use crate::theme::{wrapper, WRAPPER_PADDING_LEFT, WRAPPER_PADDING_RIGHT};
use crate::web_link::{WebLink, WebLinkProps};
use craft::components::{Component, ComponentSpecification, Props};
use craft::elements::{Container, ElementStyles, Text, TinyVg};
use craft::resource_manager::ResourceIdentifier;
use craft::style::{AlignItems, Display, FlexDirection, JustifyContent, Overflow, Unit, Weight, Wrap};
use craft::{palette, Color};
use std::path::PathBuf;
use crate::link::{Link, LinkProps};

fn hero_intro() -> ComponentSpecification {
    let bg_wrapper =
        Container::new()
            .width("100%")
            .background(Color::from_rgb8(45, 48, 53))
            ;

    let mut inner_wrapper = wrapper()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .padding(Unit::Px(100.0), WRAPPER_PADDING_RIGHT, Unit::Px(100.0), WRAPPER_PADDING_LEFT)
        ;

    inner_wrapper.push_in_place(
        Text::new("A Reactive GUI Framework for Rust")
            .color(Color::WHITE)
            .font_size(56.0)
            .line_height(1.0)
            .max_width("680px")
            .font_weight(Weight::BOLD)
            .margin("0px", "0px", "32px", "0px")
            .component()
    );

    inner_wrapper.push_in_place(
            Text::new("Build your UI with regular Rust code.")
                .line_height(1.0)
                .color(Color::WHITE)
                .font_size(20.0)
                .component()
    );

    let github_button = Text::new("GitHub")
        .display(Display::Flex)
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::Center)
        .font_size(22.0)
        .min_width("100px")
        .border_width("1px", "1px", "1px", "1px")
        .border_radius(8.0, 8.0, 8.0, 8.0)
        .padding("8px", "20px", "8px", "20px")
        .border_color(palette::css::WHITE)
        .color(palette::css::WHITE);

    let craft_button = Text::new("Learn Craft")
        .display(Display::Flex)
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::Center)
        .font_size(22.0)
        .min_width("100px")
        .border_radius(8.0, 8.0, 8.0, 8.0)
        .padding("8px", "20px", "8px", "20px")
        .background(Color::from_rgb8(69, 117, 230))
        .color(palette::css::WHITE);

    let buttons = Container::new()
        .display(Display::Flex)
        .wrap(Wrap::Wrap)
        .gap("17px")
        .margin("40px", "0px", "0px", "0px")
        .push(
            Link::component()
                .props(Props::new(LinkProps {
                    href: "/docs".to_string(),
                }))
                .push(craft_button)
        )
        .push(
            WebLink::component()
                .props(Props::new(WebLinkProps {
                    href: "https://github.com/craft-gui/craft".to_string(),
                }))
                .push(github_button)
        );

    inner_wrapper.push_in_place(buttons.component());

    bg_wrapper
        .push(inner_wrapper)
        .component()
}

fn hero_features() -> ComponentSpecification {
    fn hero_item(title: &str, text: &str, icon: ResourceIdentifier) -> Container {
        let sub_title_color = Color::from_rgb8(70, 70,70);

        let icon_title = Container::new()
            .push(TinyVg::new(icon))
            .push(Text::new(title).font_weight(Weight::MEDIUM).font_size(24.0).margin(0, 0, 0, 10));

        Container::new()
            .gap("10px")
            .flex_grow(1.0)
            .flex_shrink(1.0)
            .min_width("320px")
            .flex_basis("50%")
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .push(icon_title)
            .push(Text::new(text).font_size(18.0).color(sub_title_color))
    }

    Container::new()
        .background(Color::from_rgb8(247, 247, 247))
        .width("100%")
        .push(
            wrapper()
                .padding(Unit::Px(100.0), WRAPPER_PADDING_LEFT, Unit::Px(100.0), WRAPPER_PADDING_RIGHT)
                .display(Display::Flex)
                .wrap(Wrap::Wrap)
                .column_gap(50)
                .push(Text::new("Features").width("100%").font_size(36.0).font_weight(Weight::SEMIBOLD))
                .push(
                    hero_item(
                        "Reactive",
                        "When your data changes, we automatically re-run your view function.",
                        ResourceIdentifier::Bytes(include_bytes!("../assets/electric_bolt_24dp_000000_FILL0_wght400_GRAD0_opsz24.tvg"))
                    )
                )
                .push(
                    hero_item(
                        "Components",
                        "Components are reusable blocks that manage their own state and define both how they are rendered and how they respond to updates.",
                        ResourceIdentifier::Bytes(include_bytes!("../assets/view_comfy_24dp_000000_FILL0_wght400_GRAD0_opsz24.tvg"))
                    )
                )
                .push(
                    hero_item(
                        "Pure Rust without macros",
                        "No macros.",
                        ResourceIdentifier::Bytes(include_bytes!("../assets/code_24dp_000000_FILL0_wght400_GRAD0_opsz24.tvg"))
                    )
                )
                .push(
                    hero_item(
                        "Web-like styling",
                        "We use Taffy, an implementation of the CSS flexbox, block, and grid layout algorithms, for simple and familiar styling.",
                        ResourceIdentifier::Bytes(include_bytes!("../assets/brush_24dp_000000_FILL0_wght400_GRAD0_opsz24.tvg"))
                    )
                )
                .push(
                    hero_item(
                        "Cross Platform",
                        "Currently we support Windows, macOS, Linux, Web, and Android.",
                        ResourceIdentifier::Bytes(include_bytes!("../assets/devices_24dp_000000_FILL0_wght400_GRAD0_opsz24.tvg"))
                    )
                )

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
