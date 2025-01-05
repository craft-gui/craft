use oku::elements::ElementStyles;

use oku::components::{Component, ComponentSpecification, UpdateResult};
use oku::elements::Container;
use oku::oku_main_with_options;
use oku::style::Position;
use oku::style::Unit;
use oku::OkuOptions;
use oku::events::Event;
use oku::events::clicked;
use oku::renderer::color::Color;
use oku::RendererType;

#[derive(Default, Copy, Clone)]
pub struct EventsExample {
}

impl Component for EventsExample {
    type Props = ();

    fn view(
        _state: &Self,
        _props: &Self::Props,
        _children: Vec<ComponentSpecification>,
    ) -> ComponentSpecification {
        Container::new()
            .background(Color::RED)
            .width(Unit::Px(400.0))
            .height(Unit::Px(400.0))
            .id("red")
            .push(
                Container::new()
                    .background(Color::GREEN)
                    .inset(Unit::Px(50.0), Unit::Px(50.0), Unit::Px(50.0), Unit::Px(50.0))
                    .position(Position::Absolute)
                    .width(Unit::Px(200.0))
                    .height(Unit::Px(200.0))
                    .id("green")
                    .push(
                        Container::new()
                            .background(Color::BLUE)
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
        _props: &Self::Props,
        event: Event,
    ) -> UpdateResult {
        
        if clicked(event.message) {
            println!("Target: {:?}, Current Target: {:?}", event.target, event.current_target);
        }

        UpdateResult::new()
    }
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    oku_main_with_options(
        EventsExample::component(),
        Some(OkuOptions {
            renderer: RendererType::default(),
            window_title: "events".to_string(),
        }),
    );
}
