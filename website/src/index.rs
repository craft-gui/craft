use retgui::elements::{Container, Element, Elements, Text};
use retgui::style::{Display, FlexDirection, FlexWrap, FontWeight, Overflow, Unit};
use retgui::{Color, palette, pct, px, rgb};

use crate::link::Link;
use crate::router::NavigateFn;
use crate::theme::{WRAPPER_PADDING_LEFT, WRAPPER_PADDING_RIGHT, wrapper};
use crate::web_link::WebLink;

fn hero_intro(elements: &mut Elements, navigate: NavigateFn) -> Container {
    let heading = Text::new(elements, "A Reactive GUI Framework for Rust")
        .edit(elements)
        .color(Color::WHITE)
        .font_size(56.0)
        .line_height(1.0)
        .max_width(px(680))
        .font_weight(FontWeight::BOLD)
        .finish();
    let subtitle = Text::new(elements, "Build your UI with regular Rust code.")
        .edit(elements)
        .line_height(1.0)
        .color(Color::WHITE)
        .font_size(20.0)
        .finish();
    let learn_label = Text::new(elements, "Learn RetGui")
        .edit(elements)
        .selectable(false)
        .color(palette::css::WHITE)
        .finish();
    let learn = Link(elements, move |elements| navigate("/docs", elements))
        .edit(elements)
        .padding(px(8), px(20), px(8), px(20))
        .background_color(rgb(69, 117, 230))
        .push(learn_label)
        .finish();
    let github_label = Text::new(elements, "GitHub")
        .edit(elements)
        .selectable(false)
        .color(palette::css::WHITE)
        .finish();
    let github = WebLink(elements, "https://github.com/RetGui/retgui")
        .edit(elements)
        .padding(px(8), px(20), px(8), px(20))
        .border_width_all(px(1))
        .border_color_all(palette::css::WHITE)
        .push(github_label)
        .finish();
    let buttons = Container::new(elements)
        .edit(elements)
        .display(Display::Flex)
        .wrap(FlexWrap::Wrap)
        .gap(px(17), px(17))
        .push(learn)
        .push(github)
        .finish();
    let inner = wrapper(elements)
        .edit(elements)
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .padding(
            Unit::Px(100.0),
            WRAPPER_PADDING_RIGHT,
            Unit::Px(100.0),
            WRAPPER_PADDING_LEFT,
        )
        .row_gap(px(28))
        .push(heading)
        .push(subtitle)
        .push(buttons)
        .finish();
    Container::new(elements)
        .edit(elements)
        .width(pct(100))
        .background_color(rgb(45, 48, 53))
        .push(inner)
        .finish()
}

fn hero_features(elements: &mut Elements) -> Container {
    let heading = Text::new(elements, "Features")
        .edit(elements)
        .width(pct(100))
        .font_size(36.0)
        .font_weight(FontWeight::SEMIBOLD)
        .finish();
    let features = [
        (
            "Compile-time ownership",
            "Slotmap handles and explicit stores replace runtime borrow checks.",
        ),
        ("Pure Rust", "No UI macros are required."),
        ("Web-like styling", "Flexbox and block layout use familiar concepts."),
        (
            "Cross platform",
            "Windows, macOS, Linux, Web, and Android are supported.",
        ),
    ];
    let content = wrapper(elements)
        .edit(elements)
        .padding(px(80), px(20), px(80), px(20))
        .display(Display::Flex)
        .wrap(FlexWrap::Wrap)
        .gap(px(24), px(32))
        .push(heading)
        .finish();
    for (name, description) in features {
        let name = Text::new(elements, name)
            .edit(elements)
            .font_weight(FontWeight::MEDIUM)
            .font_size(24.0)
            .finish();
        let description = Text::new(elements, description)
            .edit(elements)
            .font_size(18.0)
            .color(rgb(70, 70, 70))
            .finish();
        let item = Container::new(elements)
            .edit(elements)
            .flex_grow(1.0)
            .min_width(px(320))
            .flex_basis(pct(45))
            .flex_direction(FlexDirection::Column)
            .row_gap(px(8))
            .push(name)
            .push(description)
            .finish();
        content.push(elements, item);
    }
    Container::new(elements)
        .edit(elements)
        .background_color(rgb(247, 247, 247))
        .width(pct(100))
        .push(content)
        .finish()
}

pub(crate) fn index_page(elements: &mut Elements, navigate: NavigateFn) -> Container {
    let intro = hero_intro(elements, navigate);
    let features = hero_features(elements);
    let page = Container::new(elements)
        .edit(elements)
        .display(Display::Flex)
        .width(pct(100))
        .flex_direction(FlexDirection::Column)
        .flex_grow(1.0)
        .push(intro)
        .push(features)
        .finish();
    Container::new(elements)
        .edit(elements)
        .width(pct(100))
        .overflow(Overflow::Visible, Overflow::Scroll)
        .push(page)
        .finish()
}
