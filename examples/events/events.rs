#[path = "../util.rs"]
mod util;

use crate::util::setup_logging;
use craft::components::{Component, ComponentSpecification, UpdateResult};
use craft::craft_main_with_options;
use craft::elements::Container;
use craft::elements::ElementStyles;
use craft::events::Event;
use craft::palette;
use craft::style::Position;
use craft::style::Unit;
use craft::CraftOptions;
use craft::RendererType;
use craft_core::components::ComponentId;

#[derive(Default, Copy, Clone)]
pub struct EventsExample {}

impl Component<()> for EventsExample {
    type Props = ();

    fn view(
        _state: &Self,
        _global_state: &(),
        _props: &Self::Props,
        _children: Vec<ComponentSpecification>,
        _id: ComponentId,
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

    fn update(_state: &mut Self, _global_state: &mut (), _props: &Self::Props, event: Event) -> UpdateResult {
        if event.message.clicked() {
            println!("Target: {:?}, Current Target: {:?}", event.target, event.current_target);
        }

        UpdateResult::new()
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
        }),
    );
}
