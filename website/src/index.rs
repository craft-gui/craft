use craft_retained::elements::{Container, Element, Text, TinyVg};
use craft_retained::style::{AlignItems, Display, FlexDirection, FlexWrap, FontWeight, JustifyContent, Overflow, Unit};
use craft_retained::{Color, ResourceId, palette, pct, px, rgb};

use crate::link::Link;
use crate::router::NavigateFn;
use crate::theme::{WRAPPER_PADDING_LEFT, WRAPPER_PADDING_RIGHT, wrapper};
use crate::web_link::WebLink;

fn hero_intro(navigate_fn: NavigateFn) -> Container {
    let bg_wrapper = Container::new().width(pct(100)).background_color(rgb(45, 48, 53));

    let inner_wrapper = wrapper()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .padding(
            Unit::Px(100.0),
            WRAPPER_PADDING_RIGHT,
            Unit::Px(100.0),
            WRAPPER_PADDING_LEFT,
        );

    let inner_wrapper = inner_wrapper
        .push(
            Text::new("A Reactive GUI Framework for Rust")
                .color(Color::WHITE)
                /*.font_size(if window_ctx.inner_size().width <= MOBILE_MEDIA_QUERY_WIDTH {
                    36.0
                } else {
                    56.0
                })*/
                .font_size(56.0)
                .line_height(1.0)
                .max_width(px(680))
                .font_weight(FontWeight::BOLD)
                .margin(px(0), px(0), px(32), px(0)),
        )
        .push(
            Text::new("Build your UI with regular Rust code.")
                .line_height(1.0)
                .color(Color::WHITE)
                .font_size(20.0),
        );

    let github_button = Text::new("GitHub")
        .selectable(false)
        .display(Display::Flex)
        .align_items(Some(AlignItems::Center))
        .justify_content(Some(JustifyContent::Center))
        .font_size(22.0)
        .min_width(px(100))
        .border_width(px(1), px(1), px(1), px(1))
        .border_radius((8.0, 8.0), (8.0, 8.0), (8.0, 8.0), (8.0, 8.0))
        .padding(px(8), px(20), px(8), px(20))
        .border_color(
            palette::css::WHITE,
            palette::css::WHITE,
            palette::css::WHITE,
            palette::css::WHITE,
        )
        .color(palette::css::WHITE);

    let craft_button = Text::new("Learn Craft")
        .selectable(false)
        .display(Display::Flex)
        .align_items(Some(AlignItems::Center))
        .justify_content(Some(JustifyContent::Center))
        .font_size(22.0)
        .min_width(px(100))
        .border_radius((8.0, 8.0), (8.0, 8.0), (8.0, 8.0), (8.0, 8.0))
        .padding(px(8), px(20), px(8), px(20))
        .background_color(rgb(69, 117, 230))
        .color(palette::css::WHITE);

    let buttons = Container::new()
        .display(Display::Flex)
        .wrap(FlexWrap::Wrap)
        .gap(px(17), px(17))
        .margin(px(40), px(0), px(0), px(0))
        .push(Link(move || navigate_fn("/docs")).push(craft_button))
        .push(WebLink("https://github.com/craft-gui/craft").push(github_button));

    let inner_wrapper = inner_wrapper.push(buttons);

    bg_wrapper.push(inner_wrapper)
}

fn hero_features() -> Container {
    fn hero_item(title: &str, text: &str, icon: ResourceId) -> Container {
        let sub_title_color = Color::from_rgb8(70, 70, 70);

        let icon_title = Container::new().push(TinyVg::new(icon)).push(
            Text::new(title)
                .font_weight(FontWeight::MEDIUM)
                .font_size(24.0)
                .margin(px(0), px(0), px(0), px(10)),
        );

        Container::new()
            .gap(px(10), px(10))
            .flex_grow(1.0)
            .flex_shrink(1.0)
            .min_width(px(320))
            .flex_basis(pct(50))
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .push(icon_title)
            .push(Text::new(text).font_size(18.0).color(sub_title_color))
    }

    Container::new()
        .background_color(rgb(247, 247, 247))
        .width(pct(100))
        .push(
            wrapper()
                .padding(Unit::Px(100.0), WRAPPER_PADDING_LEFT, Unit::Px(100.0), WRAPPER_PADDING_RIGHT)
                .display(Display::Flex)
                .wrap(FlexWrap::Wrap)
                .gap(px(0), px(50))
                .push(Text::new("Features").width(pct(100)).font_size(36.0).font_weight(FontWeight::SEMIBOLD))
                .push(
                    hero_item(
                        "Reactive",
                        "When your data changes, we automatically re-run your view function.",
                        ResourceId::Bytes(include_bytes!("../assets/electric_bolt_24dp_000000_FILL0_wght400_GRAD0_opsz24.tvg"))
                    )
                )
                .push(
                    hero_item(
                        "Components",
                        "Components are reusable blocks that manage their own state and define both how they are rendered and how they respond to updates.",
                        ResourceId::Bytes(include_bytes!("../assets/view_comfy_24dp_000000_FILL0_wght400_GRAD0_opsz24.tvg"))
                    )
                )
                .push(
                    hero_item(
                        "Pure Rust without macros",
                        "No macros.",
                        ResourceId::Bytes(include_bytes!("../assets/code_24dp_000000_FILL0_wght400_GRAD0_opsz24.tvg"))
                    )
                )
                .push(
                    hero_item(
                        "Web-like styling",
                        "We use Taffy, an implementation of the CSS flexbox, block, and grid layout algorithms, for simple and familiar styling.",
                        ResourceId::Bytes(include_bytes!("../assets/brush_24dp_000000_FILL0_wght400_GRAD0_opsz24.tvg"))
                    )
                )
                .push(
                    hero_item(
                        "Cross Platform",
                        "Currently we support Windows, macOS, Linux, Web, and Android.",
                        ResourceId::Bytes(include_bytes!("../assets/devices_24dp_000000_FILL0_wght400_GRAD0_opsz24.tvg"))
                    )
                )

        )
}

pub(crate) fn index_page(navigate_fn: NavigateFn) -> Container {
    Container::new()
        .width(pct(100))
        .overflow(Overflow::Visible, Overflow::Scroll)
        .push(
            Container::new()
                .display(Display::Flex)
                .width(pct(100))
                .margin(Unit::Px(0.0), Unit::Auto, Unit::Px(0.0), Unit::Auto)
                .flex_direction(FlexDirection::Column)
                .flex_grow(1.0)
                .push(hero_intro(navigate_fn))
                .push(hero_features()),
        )
}
