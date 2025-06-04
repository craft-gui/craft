#[path = "../../examples/counter/main.rs"]
mod counter;

#[path = "../../examples/text/main.rs"]
mod text;

#[path = "../../examples/request/main.rs"]
mod request;

#[path = "../../examples/tour/main.rs"]
mod tour;

use crate::examples::counter::Counter;
use crate::examples::request::AniList;
use crate::examples::text::TextState;
use crate::examples::tour::Tour;
use crate::navbar::NAVBAR_HEIGHT;
use crate::theme::EXAMPLES_SIDEBAR_BACKGROUND_COLOR;
use crate::WebsiteGlobalState;
use craft::components::{Component, ComponentId, ComponentSpecification, Event};
use craft::elements::{Container, ElementStyles, Text};
use craft::events::Message;
use craft::palette;
use craft::style::Display::Flex;
use craft::style::{FlexDirection, Unit};
use craft::WindowContext;

pub(crate) struct Examples {
    pub(crate) example_to_show: String,
}

impl Default for Examples {
    fn default() -> Self {
        Examples {
            example_to_show: "counter".to_string(),
        }
    }
}

fn create_examples_link(label: &str, example_link: &str) -> ComponentSpecification {
    Text::new(label).id(example_link).disable_selection().component()
}

fn examples_sidebar() -> ComponentSpecification {
    Container::new()
        .background(EXAMPLES_SIDEBAR_BACKGROUND_COLOR)
        .display(Flex)
        .flex_direction(FlexDirection::Column)
        .gap("15px")
        .padding("20px", "20px", "20px", "20px")
        .height("100%")
        .push(Text::new("Examples").font_size(24.0).component())
        .push(create_examples_link("Counter", "example_counter"))
        .push(create_examples_link("Tour", "example_tour"))
        .push(create_examples_link("Request", "example_request"))
        .push(create_examples_link("Text", "example_text_state"))
        .component()
}

impl Component for Examples {
    type GlobalState = WebsiteGlobalState;
    type Props = ();
    type Message = ();

    fn view(
        &self,
        _global_state: &Self::GlobalState,
        _props: &Self::Props,
        _children: Vec<ComponentSpecification>,
        _id: ComponentId,
        window: &WindowContext,
    ) -> ComponentSpecification {
        let wrapper = Container::new().display(Flex).width("100%").height("100%").push(examples_sidebar()).component();

        let content = match self.example_to_show.as_str() {
            "text_state" => TextState::component().key("example_text_state"),
            "tour" => Tour::component().key("example_tour"),
            "request" => AniList::component().key("example_request"),
            _ => Counter::component().key("example_counter"),
        };

        let container_height = (window.window_height() - NAVBAR_HEIGHT).max(0.0);

        wrapper.push(
            Container::new()
                .width("100%")
                .height(Unit::Px(container_height))
                .background(palette::css::WHITE)
                .push(content)
                .component(),
        )
    }

    fn update(
        &mut self,
        _global_state: &mut Self::GlobalState,
        _props: &Self::Props,
        event: &mut Event,
        message: &Message,
    ) {
        if message.clicked() {
            if let Some(target) = event.current_target.as_ref() {
                if let Some(id) = target.get_id() {
                    if id.starts_with("example_") {
                        self.example_to_show = id.trim_start_matches("example_").to_string();
                    }
                }
            }
        }
    }
}
