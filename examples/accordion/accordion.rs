use oku::components::ComponentSpecification;
use oku::elements::{Container, Text};

use oku::{oku_main_with_options, OkuOptions};

use oku::components::{Component, UpdateResult};
use oku::elements::ElementStyles;
use oku::events::Event;
use oku::style::FlexDirection;
use oku::style::Unit;
use oku::events::clicked;
use oku::RendererType;

#[derive(Default, Copy, Clone)]
pub struct Accordion {
    show_content: bool,
}

impl Component for Accordion {
    type Props = ();

    fn view(state: &Self, _props: &Self::Props, _children: Vec<ComponentSpecification>) -> ComponentSpecification {
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

    fn update(state: &mut Self, _props: &Self::Props, event: Event) -> UpdateResult {
        println!("target: {:?}", event.target);
        if event.target.as_deref() != Some("accordion_header") {
            return UpdateResult::default();
        }

        if clicked(&event.message) {
            state.show_content = !state.show_content
        }

        UpdateResult::new().prevent_propagate()
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO) // Set the maximum log level you want to capture
        .init();

    oku_main_with_options(
        Container::new().push_children(vec![Accordion::component(), Accordion::component()]).component(),
        Some(OkuOptions {
            renderer: RendererType::default(),
            window_title: "accordion".to_string(),
        }),
    );
}
