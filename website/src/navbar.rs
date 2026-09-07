use retgui::elements::{Container, Element, Text};
use retgui::style::{AlignItems, Display, FontWeight, JustifyContent, Unit};
use retgui::{App, pct, px, rgb};

use crate::link::Link;
use crate::router::NavigateFn;
use crate::theme::{NAVBAR_BACKGROUND_COLOR, NAVBAR_TEXT_COLOR, wrapper};

pub const NAVBAR_HEIGHT: f32 = 60.0;

fn create_link(app: &mut App, navigate: NavigateFn, label: &str, route: &str) -> Container {
    let route_owned = route.to_string();
    let text = Text::new(app, label)
        .edit(app)
        .id(&format!("route_{route}"))
        .margin(px(0), px(12), px(0), px(0))
        .font_size(16.0)
        .selectable(false)
        .color(NAVBAR_TEXT_COLOR)
        .finish();
    Link(app, move |app| navigate(&route_owned, app))
        .edit(app)
        .push(text)
        .finish()
}

pub fn navbar(app: &mut App, navigate: NavigateFn) -> Container {
    let brand = create_link(app, navigate.clone(), "RetGui", "/")
        .edit(app)
        .font_size(32.0)
        .font_weight(FontWeight::BOLD)
        .margin(px(0), px(24), px(0), px(0))
        .finish();
    let home = create_link(app, navigate.clone(), "Home", "/");
    let docs = create_link(app, navigate.clone(), "Docs", "/docs");
    let examples = create_link(app, navigate, "Examples", "/examples");
    let links = Container::new(app)
        .edit(app)
        .display(Display::Flex)
        .justify_content(JustifyContent::Center)
        .align_items(AlignItems::Center)
        .push(brand)
        .push(home)
        .push(docs)
        .push(examples)
        .finish();
    let inner = wrapper(app)
        .edit(app)
        .display(Display::Flex)
        .justify_content(JustifyContent::SpaceBetween)
        .align_items(AlignItems::Center)
        .push(links)
        .finish();
    let border = rgb(240, 240, 240);
    Container::new(app)
        .edit(app)
        .width(pct(100))
        .height(Unit::Px(NAVBAR_HEIGHT))
        .min_height(Unit::Px(NAVBAR_HEIGHT))
        .max_height(Unit::Px(NAVBAR_HEIGHT))
        .border_width(px(0), px(0), px(2), px(0))
        .border_color(border, border, border, border)
        .background_color(NAVBAR_BACKGROUND_COLOR)
        .push(inner)
        .finish()
}
