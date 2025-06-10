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
use crate::theme::{wrapper, ACTIVE_LINK_COLOR, DEFAULT_LINK_COLOR, WRAPPER_PADDING_LEFT, WRAPPER_PADDING_RIGHT};
use crate::WebsiteGlobalState;
use craft::components::{Component, ComponentId, ComponentSpecification, Event, Props};
use craft::elements::{Container, ElementStyles, Text};
use craft::events::ui_events::pointer::PointerButtonUpdate;
use craft::palette;
use craft::style::Display::Flex;
use craft::style::{FlexDirection, Overflow, Unit, Weight};
use craft::WindowContext;
use util::ExampleProps;

const COUNTER_EXAMPLE_LINK: &str = "/examples/counter";
const TOUR_EXAMPLE_LINK: &str = "/examples/tour";
const REQUEST_EXAMPLE_LINK: &str = "/examples/request";
const TEXT_EXAMPLE_LINK: &str = "/examples/text";

#[derive(Default)]
pub(crate) struct Examples {
}

fn create_examples_link(label: &str, example_link: &str, example_to_show: &String) -> ComponentSpecification {
    let example_link_captured = example_link.to_string();
    let mut text = Text::new(label)
        .color(DEFAULT_LINK_COLOR)
        .on_pointer_button_up(
            move |_state: &mut Examples, global_state: &mut WebsiteGlobalState, event: &mut Event, pointer_button: &PointerButtonUpdate| {
                if pointer_button.is_primary() {
                    global_state.set_route(example_link_captured.as_str());
                    event.prevent_propagate();
                }
            },
        )
        .id(example_link)
        .disable_selection();
    if example_to_show == example_link {
        text = text.color(ACTIVE_LINK_COLOR);
    }
    text.component()
}

fn examples_sidebar(example_to_show: &String) -> ComponentSpecification {
    
    Container::new()
        .display(Flex)
        .flex_direction(FlexDirection::Column)
        .gap("15px")
        .min_width("200px")
        .padding("0px", "20px", "20px", "0px")
        .height("100%")
        .push(Text::new("Examples").font_weight(Weight::MEDIUM).font_size(24.0).component())
        .push(create_examples_link("Counter", COUNTER_EXAMPLE_LINK, example_to_show))
        .push(create_examples_link("Tour", TOUR_EXAMPLE_LINK, example_to_show))
        .push(create_examples_link("Request", REQUEST_EXAMPLE_LINK, example_to_show))
        .push(create_examples_link("Text", TEXT_EXAMPLE_LINK, example_to_show))
        .component()
}

impl Component for Examples {
    type GlobalState = WebsiteGlobalState;
    type Props = ();
    type Message = ();

    fn view(
        &self,
        global_state: &Self::GlobalState,
        _props: &Self::Props,
        _children: Vec<ComponentSpecification>,
        _id: ComponentId,
        window: &WindowContext,
    ) -> ComponentSpecification {
        let route = global_state.get_route();
        let example_to_show: String = if route == "/examples" { COUNTER_EXAMPLE_LINK.to_string() } else { route };
        
        let vertical_padding = 50.0;
        let wrapper = wrapper()
            .padding(Unit::Px(vertical_padding), WRAPPER_PADDING_RIGHT, Unit::Px(vertical_padding), WRAPPER_PADDING_LEFT)
            .push(examples_sidebar(&example_to_show)).component();

        let example_props = ExampleProps {
            show_scrollbar: false,
        };
        let content = match example_to_show.as_str() {
            TEXT_EXAMPLE_LINK => TextState::component().key(TEXT_EXAMPLE_LINK),
            TOUR_EXAMPLE_LINK => Tour::component().key(TOUR_EXAMPLE_LINK).props(Props::new(example_props)),
            REQUEST_EXAMPLE_LINK => AniList::component().key(REQUEST_EXAMPLE_LINK).props(Props::new(example_props)),
            _ => Counter::component().key(COUNTER_EXAMPLE_LINK),
        };

        let container_height = (window.window_height() - NAVBAR_HEIGHT - vertical_padding * 2.0).max(0.0);

        let wrapper = wrapper.push(
            Container::new()
                .width("100%")
                .height(Unit::Px(container_height))
                .background(palette::css::WHITE)
                .push(content)
                .component(),
        );
        
        Container::new() 
            .overflow_y(Overflow::Scroll)
            .push(wrapper).component()
    }
}
