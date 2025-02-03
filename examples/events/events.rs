#[path = "../util.rs"]
mod util;

use oku::elements::ElementStyles;
use oku::components::{Component, ComponentSpecification, UpdateResult};
use oku::elements::Container;
use oku_core::events::{ButtonSource, ElementState, Message, MouseButton};
use oku::oku_main_with_options;
use oku::OkuOptions;
use oku::events::{clicked, Event};
use oku::events::OkuMessage::PointerButtonEvent;
use oku::renderer::color::palette;
use oku::style::Unit;
use oku::style::Position;
use oku::RendererType;
use oku_core::GlobalState;
use crate::util::setup_logging;

#[derive(Default, Copy, Clone)]
pub struct EventsExample {
}

impl Component<()> for EventsExample {
    type Props = ();

    fn view(
        _state: &Self,
        _global_state: &(),
        _props: &Self::Props,
        _children: Vec<ComponentSpecification>,
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
            ).component()
    }

    fn update(
        _state: &mut Self,
        _global_state: &mut (),
        _props: &Self::Props,
        event: Event,
    ) -> UpdateResult {
        
        if clicked(&event.message) {
            println!("Target: {:?}, Current Target: {:?}", event.target, event.current_target);
        }

        UpdateResult::new()
    }
}

fn main() {
    setup_logging();

    oku_main_with_options(
        EventsExample::component(),
        Box::new(()),
        Some(OkuOptions {
            renderer: RendererType::default(),
            window_title: "events".to_string(),
        }),
    );
}
