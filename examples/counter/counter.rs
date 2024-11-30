use oku::RendererType::Wgpu;

use oku::components::{Component, ComponentSpecification, UpdateResult};
use oku::elements::{Container, Text};
use oku::engine::events::{ButtonSource, ElementState, Message, MouseButton};
use oku::oku_main_with_options;
use oku::style::FlexDirection;
use oku::OkuOptions;

#[cfg(target_os = "android")]
use oku::{AndroidApp};
use oku::engine::events::Event;
use oku::engine::events::OkuMessage::PointerButtonEvent;
use oku_core::elements::ElementStyles;
use oku_core::engine::renderer::color::Color;
use oku_core::RendererType::Vello;
use oku_core::style::{Display, JustifyContent, Overflow, Unit, Wrap};

#[derive(Default, Copy, Clone)]
pub struct Counter {
    count: u64,
}

impl Component for Counter {
    type Props = ();

    fn view(
        state: &Self,
        _props: &Self::Props,
        _children: Vec<ComponentSpecification>,
    ) -> ComponentSpecification {
        Container::new()
            .flex_direction(FlexDirection::Column)
            .component()
            .push(Text::new(format!("Counter: {}", state.count).as_str()).component())
            .push(Container::new().component())
            .push(Text::new("increment").id("increment").component())
    }

    fn update(
        state: &mut Self,
        _props: &Self::Props,
        message: Event,
    ) -> UpdateResult {
        if message.target.as_deref() != Some("increment") {
            return UpdateResult::default();
        }

        if let Message::OkuMessage(PointerButtonEvent(pointer_button)) = message.message {
            if pointer_button.button == ButtonSource::Mouse(MouseButton::Left)
                && pointer_button.state == ElementState::Pressed
            {
                state.count += 1
            }
        };

        UpdateResult::new().prevent_propagate()
    }
}

#[cfg(not(target_os = "android"))]
fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    oku_main_with_options(
        Container::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .justify_content(JustifyContent::Start)
            .width(Unit::Px(300.0))
            //.height(Unit::Px(400.0))
            .height(Unit::Px(780.0))
            .background(Color::RED)
            .overflow(Overflow::Scroll)
            .border(Unit::Px(5.0), Unit::Px(5.0), Unit::Px(5.0), Unit::Px(5.0))
            .margin(Unit::Px(5.0), Unit::Px(5.0), Unit::Px(5.0), Unit::Px(5.0))
            .padding(Unit::Px(5.0), Unit::Px(5.0), Unit::Px(5.0), Unit::Px(5.0))
            .component()
            .push(Counter::component())
            .push(
                Container::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column)
                    .width(Unit::Px(200.0))
                    .min_height(Unit::Px(500.0))
                    .max_height(Unit::Px(500.0))
                    .height(Unit::Px(500.0))
                    .background(Color::BLUE)
                    .overflow(Overflow::Scroll)
                    .border(Unit::Px(5.0), Unit::Px(5.0), Unit::Px(5.0), Unit::Px(5.0))
                    .margin(Unit::Px(5.0), Unit::Px(5.0), Unit::Px(5.0), Unit::Px(5.0))
                    .padding(Unit::Px(5.0), Unit::Px(5.0), Unit::Px(5.0), Unit::Px(5.0))
                    .component()
                    .push(Counter::component())
                    .push(
                        Container::new()
                            .width(Unit::Px(150.0))
                            .min_height(Unit::Px(700.0))
                            .height(Unit::Px(700.0))
                            .background(Color::GREEN)
                            .component()
                    )
            )
            .push(
                Container::new()
                    .min_height(Unit::Px(400.0))
                    .width(Unit::Px(200.0))
                    .margin(Unit::Px(400.0), Unit::Px(0.0), Unit::Px(2.0), Unit::Px(0.0))
                    .background(Color::rgba(255, 255, 0, 255))
                    .component()
            )
           ,
        Some(OkuOptions {
            renderer: Wgpu,
            window_title: "counter".to_string(),
        }),
    );
}



#[allow(dead_code)]
#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: AndroidApp) {
    #[cfg(not(target_arch = "wasm32"))]
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    oku_main_with_options(
        Counter::component(),
        Some(OkuOptions {
            renderer: Wgpu,
            window_title: "counter".to_string(),
        }),
        app
    );
}