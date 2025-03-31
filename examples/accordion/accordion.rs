#[path = "../util.rs"]
mod util;

use crate::util::setup_logging;
use oku::components::ComponentSpecification;
use oku::components::{Component, UpdateResult};
use oku::elements::ElementStyles;
use oku::elements::{Container, Text};
use oku::events::Event;
use oku::style::FlexDirection;
use oku::style::Unit;
use oku::RendererType;
use oku::{oku_main_with_options, OkuOptions};
use oku_core::components::ComponentId;

#[derive(Default, Copy, Clone)]
pub struct Accordion {
    show_content: bool,
}

impl Component<()> for Accordion {
    type Props = ();

    fn view(
        state: &Self,
        _global_state: &(),
        _props: &Self::Props,
        _children: Vec<ComponentSpecification>,
        _id: ComponentId,
    ) -> ComponentSpecification {
        let accordion_content =
            if state.show_content { Text::new("My content!").component() } else { Container::new().component() };

        Container::new()
            .margin(Unit::Px(14.0), Unit::Px(0.0), Unit::Px(0.0), Unit::Px(14.0))
            .flex_direction(FlexDirection::Column)
            .component()
            .push(
                Container::new()
                    .id("accordion_header")
                    .component()
                    .push(Text::new("Accordion Example").id("accordion_header").component()),
            )
            .push(accordion_content)
    }

    fn update(state: &mut Self, _global_state: &mut (), _props: &Self::Props, event: Event) -> UpdateResult {
        println!("target: {:?}", event.target);
        if event.target.as_deref() != Some("accordion_header") {
            return UpdateResult::default();
        }

        if event.message.clicked() {
            state.show_content = !state.show_content
        }

        UpdateResult::new().prevent_propagate()
    }
}

fn main() {
    setup_logging();

    oku_main_with_options(
        Container::new().push_children(vec![Accordion::component(), Accordion::component()]).component(),
        Box::new(()),
        Some(OkuOptions {
            renderer: RendererType::default(),
            window_title: "accordion".to_string(),
        }),
    );
}
