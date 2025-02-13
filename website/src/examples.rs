use crate::counter::counter::Counter;
use crate::text::text::TextState;
use crate::theme::EXAMPLES_SIDEBAR_BACKGROUND_COLOR;
use crate::WebsiteGlobalState;
use oku::components::{Component, ComponentSpecification, UpdateResult};
use oku::elements::{Container, ElementStyles, Text};
use oku::events::{clicked, Event};
use oku::renderer::color::palette;
use oku::style::Display::Flex;
use oku::style::{Display, FlexDirection};

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

fn create_examples_link(label: &str, example_link: &str) -> Text {
    Text::new(label).id(example_link).color(palette::css::WHITE)
}

fn examples_sidebar() -> ComponentSpecification {
    Container::new()
        .background(EXAMPLES_SIDEBAR_BACKGROUND_COLOR)
        .display(Flex)
        .flex_direction(FlexDirection::Column)
        .gap("15px")
        .width("30%")
        .padding("20px", "20px", "20px", "20px")
        .min_width("300px")
        .max_width("50%")
        .height("100%")
        .push(Text::new("Examples").color(palette::css::WHITE).font_size(24.0).component())
        .push(create_examples_link("Counter", "example_counter"))
        .push(create_examples_link("Text", "example_text_state"))
        .component()
}

impl Component<WebsiteGlobalState> for Examples {
    type Props = ();

    fn view(state: &Self, global_state: &WebsiteGlobalState, props: &Self::Props, children: Vec<ComponentSpecification>) -> ComponentSpecification {
        let wrapper = Container::new()
            .display(Display::Flex)
            .width("100%")
            .height("100%")
            .push(examples_sidebar()).component();

        wrapper.push(
            Container::new()
                .width("100%")
                .height("100%")
                .background(palette::css::WHITE)
                .push(match state.example_to_show.as_str() {
                    "text_state" => TextState::component().key("example_text_state"),
                    "counter" | &_ => Counter::component().key("example_counter"),
                })
        )
    }

    fn update(state: &mut Self, _global_state: &mut WebsiteGlobalState, _props: &Self::Props, event: Event) -> UpdateResult {
        
        if clicked(&event.message) && event.current_target.is_some() {
            let current_target = event.current_target.as_ref().unwrap();
            if current_target.starts_with("example_") {
                state.example_to_show = current_target.replace("example_", "").to_string();
            }
        }
        
        UpdateResult::default()
    }
}