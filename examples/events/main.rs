use util::setup_logging;
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
use craft::components::ComponentId;
use craft::WindowContext;

#[derive(Default, Copy, Clone)]
pub struct EventsExample {}

impl Component<()> for EventsExample {
    type Props = ();

    fn view_with_no_global_state(
        _state: &Self,
        _props: &Self::Props,
        _children: Vec<ComponentSpecification>,
        _id: ComponentId,
        _window_context: &WindowContext
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

    fn update_with_no_global_state(_state: &mut Self, _props: &Self::Props, event: Event, _window_context: &mut WindowContext) -> UpdateResult {
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
            ..Default::default()
        }),
    );
}
