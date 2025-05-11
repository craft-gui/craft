use util::setup_logging;
use craft::components::{Component, ComponentSpecification, Event};
use craft::craft_main_with_options;
use craft::elements::Container;
use craft::elements::ElementStyles;
use craft::events::{Message};
use craft::palette;
use craft::style::Position;
use craft::style::Unit;
use craft::CraftOptions;
use craft::RendererType;
use craft::components::ComponentId;
use craft::WindowContext;

#[derive(Default, Copy, Clone)]
pub struct EventsExample {}

impl Component for EventsExample {
    type GlobalState = ();

    type Props = ();

    type Message = ();

    fn view(
        &self,
        _global_state: &Self::GlobalState,
        _props: &Self::Props,
        _children: Vec<ComponentSpecification>,
        _id: ComponentId,
        _window: &WindowContext
    ) -> ComponentSpecification {
        Container::new()
            .background(palette::css::RED)
            .width(Unit::Px(400.0))
            .height(Unit::Px(400.0))
            .id("red")
            .push(
                Container::new()
                    .background(palette::css::GREEN)
                    .inset(Unit::Px(50.0), Unit::Px(50.0), Unit::Px(50.0), Unit::Px(50.0))
                    .position(Position::Absolute)
                    .width(Unit::Px(200.0))
                    .height(Unit::Px(200.0))
                    .id("green")
                    .push(
                        Container::new()
                            .background(palette::css::BLUE)
                            .inset(Unit::Px(50.0), Unit::Px(50.0), Unit::Px(50.0), Unit::Px(50.0))
                            .position(Position::Absolute)
                            .width(Unit::Px(100.0))
                            .height(Unit::Px(100.0))
                            .id("blue"),
                    ),
            )
            .component()
    }
    fn update(&mut self, _global_state: &mut Self::GlobalState, _props: &Self::Props, event: &mut Event, message: &Message) {
        if message.clicked() {
            println!("Target: {:?}, Current Target: {:?}", event.target, event.current_target);
        }
    }
}

fn main() {
    setup_logging();

    craft_main_with_options(
        EventsExample::component(),
        Box::new(()),
        Some(CraftOptions {
            renderer: RendererType::default(),
            window_title: "events".to_string(),
            ..Default::default()
        }),
    );
}
