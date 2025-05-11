use craft::components::ComponentId;
use craft::components::ComponentSpecification;
use craft::components::{Component, Event};
use craft::elements::ElementStyles;
use craft::elements::{Container, Text};
use craft::events::{Message};
use craft::style::FlexDirection;
use craft::style::Unit;
use craft::RendererType;
use craft::WindowContext;
use craft::{craft_main_with_options, CraftOptions};
use util::setup_logging;

#[derive(Default, Copy, Clone)]
pub struct Accordion {
    show_content: bool,
}

impl Component for Accordion {
    type Props = ();
    type GlobalState = ();
    type Message = ();

    fn view(
        &self,
        _global_state: &Self::GlobalState,
        _props: &Self::Props,
        _children: Vec<ComponentSpecification>,
        _id: ComponentId,
        _window: &WindowContext,
    ) -> ComponentSpecification {
        let accordion_content =
            if self.show_content { Text::new("My content!").component() } else { Container::new().component() };

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

    fn update(
        &mut self,
        _global_state: &mut Self::GlobalState,
        _props: &Self::Props,
        event: &mut Event,
        message: &Message,
    ) {
        println!("target: {:?}", event.target);
        if event.target.as_deref() != Some("accordion_header") {
            return;
        }

        if message.clicked() {
            self.show_content = !self.show_content
        }

        Event::new().prevent_propagate()
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
