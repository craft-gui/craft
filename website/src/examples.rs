use std::rc::Rc;

use retgui::elements::{Container, Element, State, Text};
use retgui::events::PointerButton;
use retgui::style::{Display, FlexDirection, FontWeight, Overflow};
use retgui::{App, palette, pct, px};

use crate::WebsiteGlobalState;
use crate::router::NavigateFn;
use crate::theme::{ACTIVE_LINK_COLOR, DEFAULT_LINK_COLOR, wrapper};

#[allow(dead_code)]
#[path = "../../examples/counter/main.rs"]
pub mod counter;
#[allow(dead_code)]
#[path = "../../examples/pointer_events/main.rs"]
mod pointer_events;
#[allow(dead_code)]
#[path = "../../examples/text/main.rs"]
mod text;

const COUNTER: &str = "/examples/counter";
const POINTER_EVENTS: &str = "/examples/pointer-events";
const TEXT: &str = "/examples/text";

fn show_example(app: &mut App, examples: &[Container], selected: usize) {
    for (index, example) in examples.iter().enumerate() {
        example.set_display(
            app,
            if index == selected {
                Display::Flex
            } else {
                Display::None
            },
        );
    }
}

fn example_link(
    app: &mut App,
    label: &str,
    route: &'static str,
    index: usize,
    selected: State<usize>,
    examples: Rc<Vec<Container>>,
    navigate: NavigateFn,
) -> Text {
    let color = if *selected.read(app) == index {
        ACTIVE_LINK_COLOR
    } else {
        DEFAULT_LINK_COLOR
    };
    Text::new(app, label)
        .edit(app)
        .color(color)
        .selectable(false)
        .add_pointer_button_up_listener(move |event, app| {
            if event.button == Some(PointerButton::Left) {
                *selected.write(app) = index;
                show_example(app, &examples, index);
                navigate(route, app);
            }
        })
        .finish()
}

pub fn examples(app: &mut App, global_state: State<WebsiteGlobalState>, navigate: NavigateFn) -> Container {
    let route = global_state.read(app).get_route();
    let counter = counter::counter(app).edit(app).id(COUNTER).finish();
    let pointer = pointer_events::pointer_events(app)
        .edit(app)
        .id(POINTER_EVENTS)
        .finish();
    let text = text::text(app).edit(app).id(TEXT).finish();
    let examples = Rc::new(vec![counter, pointer, text]);
    let selected_index = [COUNTER, POINTER_EVENTS, TEXT]
        .iter()
        .position(|candidate| *candidate == route)
        .unwrap_or(0);
    let selected = app.insert_state(selected_index);
    show_example(app, &examples, selected_index);

    let heading = Text::new(app, "Examples")
        .edit(app)
        .selectable(false)
        .font_weight(FontWeight::MEDIUM)
        .font_size(20.0)
        .finish();
    let sidebar = Container::new(app)
        .edit(app)
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .gap(px(12), px(12))
        .min_width(px(210))
        .push(heading)
        .finish();
    for (index, (label, route)) in [("Counter", COUNTER), ("Pointer events", POINTER_EVENTS), ("Text", TEXT)]
        .into_iter()
        .enumerate()
    {
        let link = example_link(app, label, route, index, selected, examples.clone(), navigate.clone());
        sidebar.push(app, link);
    }

    let content = Container::new(app)
        .edit(app)
        .width(pct(100))
        .height(px(600))
        .background_color(palette::css::WHITE)
        .finish();
    for example in examples.iter() {
        content.push(app, *example);
    }
    let page = wrapper(app)
        .edit(app)
        .padding_all(px(40))
        .gap(px(24), px(24))
        .push(sidebar)
        .push(content)
        .finish();
    Container::new(app)
        .edit(app)
        .overflow(Overflow::Visible, Overflow::Scroll)
        .push(page)
        .finish()
}
