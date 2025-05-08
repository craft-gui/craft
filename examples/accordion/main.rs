#[path = "../util.rs"]
mod util;

use crate::util::setup_logging;
use craft::components::ComponentSpecification;
use craft::components::{Component, UpdateResult};
use craft::elements::ElementStyles;
use craft::elements::{Container, Text};
use craft::events::Event;
use craft::style::FlexDirection;
use craft::style::Unit;
use craft::RendererType;
use craft::{craft_main_with_options, CraftOptions};
use craft::components::ComponentId;
use craft::WindowContext;

#[derive(Default, Copy, Clone)]
pub struct Accordion {
    show_content: bool,
}

impl Component<()> for Accordion {
    type Props = ();

    fn view_with_no_global_state(
        state: &Self,
        _props: &Self::Props,
        _children: Vec<ComponentSpecification>,
        _id: ComponentId,
        _window_context: &WindowContext
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

    fn update_with_no_global_state(state: &mut Self, _props: &Self::Props, event: Event, _window_context: &mut WindowContext) -> UpdateResult {
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

    craft_main_with_options(
        Container::new().push_children(vec![Accordion::component(), Accordion::component()]).component(),
        Box::new(()),
        Some(CraftOptions {
            renderer: RendererType::default(),
            window_title: "accordion".to_string(),
            ..Default::default()
        }),
    );
}
