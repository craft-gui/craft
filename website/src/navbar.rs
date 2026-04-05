use crate::link::Link;
use crate::router::NavigateFn;
use crate::theme::{NAVBAR_BACKGROUND_COLOR, NAVBAR_TEXT_COLOR, wrapper};
use craft_retained::elements::{Container, Element, Text};
use craft_retained::style::{AlignItems, Display, FontWeight, JustifyContent, Unit};
use craft_retained::{pct, px, rgb};

pub const NAVBAR_HEIGHT: f32 = 60.0;

fn create_link(navigate_fn: NavigateFn, label: &str, route: &str) -> Container {
    let route_owned = route.to_string();
    let nav = navigate_fn.clone();
    Link(move || {
        nav(&route_owned);
    })
    .push(
        Text::new(label)
            .id(format!("route_{route}").as_str())
            .margin(px(0), px(12), px(0), px(0))
            .font_size(16.0)
            .selectable(false)
            .color(NAVBAR_TEXT_COLOR),
    )
    /*.hovered()
    .color(NAVBAR_TEXT_HOVERED_COLOR)
    .underline(1.0, Color::BLACK, None)
    .margin(px(0), "12px", px(0), px(0))
    .font_size(16.5)
    .disable_selection()
    .normal()*/
}

pub fn navbar(navigate_fn: NavigateFn) -> Container {
    let border_color = rgb(240, 240, 240);
    let container = Container::new()
        .width(pct(100))
        .height(Unit::Px(NAVBAR_HEIGHT))
        .min_height(Unit::Px(NAVBAR_HEIGHT))
        .max_height(Unit::Px(NAVBAR_HEIGHT))
        .border_width(px(0), px(0), px(2), px(0))
        .border_color(border_color, border_color, border_color, border_color)
        .background_color(NAVBAR_BACKGROUND_COLOR);

    let wrapper = wrapper()
        .display(Display::Flex)
        .justify_content(Some(JustifyContent::SpaceBetween))
        .align_items(Some(AlignItems::Center))
        // Left
        .push(
            Container::new()
                .display(Display::Flex)
                .justify_content(Some(JustifyContent::Center))
                .align_items(Some(AlignItems::Center))
                .push(
                    create_link(navigate_fn.clone(), "Craft", "/")
                        .font_size(32.0)
                        .font_weight(FontWeight::BOLD)
                        .margin(px(0), px(24), px(0), px(0)), /*.hovered()
                                                              .font_size(32.0)
                                                              .font_weight(Weight::BOLD)
                                                              .margin(px(0), "24px", px(0), px(0)),*/
                )
                .push(create_link(navigate_fn.clone(), "Home", "/").margin(px(0), px(12), px(0), px(0)))
                .push(create_link(navigate_fn.clone(), "Docs", "/docs").margin(px(0), px(12), px(0), px(0)))
                .push(create_link(navigate_fn.clone(), "Examples", "/examples").margin(px(0), px(12), px(0), px(0))),
        );

    container.push(wrapper)
}
