#[allow(dead_code)]
#[path = "../../examples/counter_retained/main.rs"]
pub mod counter_retained;

#[allow(dead_code)]
#[path = "../../examples/text/main.rs"]
mod text;

#[allow(dead_code)]
#[path = "../../examples/pointer_events/main.rs"]
mod pointer_events;

use std::cell::RefCell;
use std::rc::Rc;

use craft_retained::elements::{Container, Element, Text};
use craft_retained::events::ui_events::pointer::PointerButton;
use craft_retained::style::Display::Flex;
use craft_retained::style::{FlexDirection, FontWeight, Overflow, Unit};
use craft_retained::{palette, pct, px};

use crate::WebsiteGlobalState;
use crate::examples::counter_retained::counter;
use crate::examples::pointer_events::pointer_events;
use crate::examples::text::text;
use crate::router::NavigateFn;
use crate::theme::{ACTIVE_LINK_COLOR, DEFAULT_LINK_COLOR, WRAPPER_PADDING_LEFT, WRAPPER_PADDING_RIGHT, wrapper};

const COUNTER_EXAMPLE_LINK: &str = "/examples/counter";
const POINTER_EVENTS_EXAMPLE_LINK: &str = "/examples/pointer-events";
const TEXT_EXAMPLE_LINK: &str = "/examples/text";

fn create_examples_link(
    label: &str,
    example_link: &str,
    example_to_show: &str,
    navigate_fn: NavigateFn,
    example_container: Container,
    examples: Vec<Container>,
) -> Text {
    let example_link_captured = example_link.to_string();
    let mut text = Text::new(label)
        .color(DEFAULT_LINK_COLOR)
        .on_pointer_button_up(Rc::new(move |_event, pointer_button_event| {
            if pointer_button_event.button == Some(PointerButton::Primary) {
                update_active_example(example_link_captured.as_str(), example_container.clone(), &examples);
                navigate_fn(example_link_captured.as_str());
            }
        }))
        .id(example_link)
        .selectable(false);
    if example_to_show == example_link {
        text = text.color(ACTIVE_LINK_COLOR);
    }
    text
}

fn examples_sidebar(
    example_to_show: &str,
    navigate_fn: NavigateFn,
    example_container: Container,
    examples: Vec<Container>,
) -> Container {
    let mut links = vec![
        create_examples_link(
            "Counter",
            COUNTER_EXAMPLE_LINK,
            example_to_show,
            navigate_fn.clone(),
            example_container.clone(),
            examples.clone(),
        ),
        create_examples_link(
            "Pointer Events",
            POINTER_EVENTS_EXAMPLE_LINK,
            example_to_show,
            navigate_fn.clone(),
            example_container.clone(),
            examples.clone(),
        ),
        create_examples_link(
            "Text",
            TEXT_EXAMPLE_LINK,
            example_to_show,
            navigate_fn.clone(),
            example_container.clone(),
            examples.clone(),
        ),
    ];

    if true
    /*window.window_width() <= MOBILE_MEDIA_QUERY_WIDTH*/
    {
        let container = Container::new()
            .display(Flex)
            .flex_direction(FlexDirection::Column)
            .gap(px(12), px(12))
            .push(
                Text::new("Examples")
                    .selectable(false)
                    .font_weight(FontWeight::MEDIUM)
                    .font_size(18.0),
            );

        let mut dropdown = Container::new()
            .display(Flex)
            .flex_direction(FlexDirection::Column)
            .min_width(px(200))
            .width(px(200))
            .max_width(px(300));

        for link in links.drain(..) {
            let is_selected = link.get_id().map(|id| id == example_to_show).unwrap_or(false);
            if is_selected {
                //dropdown = dropdown.selected_item(index);
            }
            dropdown = dropdown.push(link);
        }

        container.push(dropdown)
    } else {
        let mut container = Container::new()
            .display(Flex)
            .flex_direction(FlexDirection::Column)
            .gap(px(15), px(15))
            .min_width(px(200))
            .padding(px(0), px(20), px(20), px(0))
            .height(pct(100))
            .push(Text::new("Examples").font_weight(FontWeight::MEDIUM).font_size(24.0));

        for link in links.drain(..) {
            container = container.push(link);
        }

        container
    }
}

pub fn examples(context: Rc<RefCell<WebsiteGlobalState>>, navigate_fn: NavigateFn) -> Container {
    let route = context.borrow().get_route();

    let examples = vec![
        counter().id(COUNTER_EXAMPLE_LINK),
        pointer_events().id(POINTER_EVENTS_EXAMPLE_LINK),
        text().id(TEXT_EXAMPLE_LINK),
    ];

    let container_height = 600.0; //(context.window().window_height() - NAVBAR_HEIGHT - vertical_padding * 2.0).max(0.0);

    let current_index = examples
        .iter()
        .position(|ex| ex.get_id().as_deref() == Some(route.as_str()))
        .unwrap_or(0);

    let example_container = Container::new()
        .width(pct(100))
        .height(px(container_height))
        .background_color(palette::css::WHITE)
        .push(examples[current_index].clone());

    let vertical_padding = 50.0;
    let wrapper = wrapper()
        .padding(
            Unit::Px(vertical_padding),
            WRAPPER_PADDING_RIGHT,
            Unit::Px(vertical_padding),
            WRAPPER_PADDING_LEFT,
        )
        .push(examples_sidebar(
            &route,
            navigate_fn.clone(),
            example_container.clone(),
            examples.clone(),
        ));

    /*if context.window().window_width() <= MOBILE_MEDIA_QUERY_WIDTH {
        wrapper = wrapper.flex_direction(FlexDirection::Column);
        wrapper = wrapper.gap(px(20), px(20));
    }*/

    let wrapper = wrapper.push(example_container.clone());

    Container::new()
        .overflow(Overflow::Visible, Overflow::Scroll)
        .push(wrapper)
}

fn update_active_example(active: &str, example_container: Container, examples: &[Container]) {
    for example in examples {
        if example.get_id().unwrap() == active {
            example_container.remove_all_children();
            example_container.push(example.clone());
            return;
        }
    }
}
