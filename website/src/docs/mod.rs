use craft::elements::{Container, ElementStyles};
use craft::style::{Display, FlexDirection};

pub(crate) mod docs_component;
pub(crate) mod installation;
pub(crate) mod hello_world;
pub(crate) mod state_management;
pub(crate) mod styling;
pub(crate) mod how_to_contribute;
mod markdown_viewer;

pub(crate) fn docs_template() -> Container {
    Container::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
}