use craft::components::{Component, ComponentId, ComponentSpecification, Event};
use craft::elements::ElementStyles;
use craft::elements::{Container, Text};
use craft::events::{Message};
use craft::style::Display;
use craft::style::{AlignItems, FlexDirection, JustifyContent};
use craft::Color;
use craft::CraftOptions;
use craft::RendererType;
use craft::{craft_main_with_options, palette};
use util::setup_logging;

#[derive(Default, Clone)]
pub struct OverlayExample {
    hovered_element_id: Option<String>,
}

impl Component for OverlayExample {
    type Props = ();
    type Message = ();
    type GlobalState = ();

    fn view(
        &self,
        _global_state: &Self::GlobalState,
        _props: &Self::Props,
        _children: Vec<ComponentSpecification>,
        _id: ComponentId,
        _window: &WindowContext,
    ) -> ComponentSpecification {
        Container::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .justify_content(JustifyContent::Center)
            .align_items(AlignItems::Center)
            .width("100%")
            .height("100%")
            .background(Color::from_rgb8(250, 250, 250))
            .push(Text::new(format!("Hovered Element: {:?}", self.hovered_element_id).as_str()))
            .push(
                Container::new()
                    .background(palette::css::RED)
                    .width(Unit::Px(400.0))
                    .height(Unit::Px(400.0))
                    .id("red")
                    .push(
                        Overlay::new()
                            .background(palette::css::GREEN)
                            .inset(Unit::Px(50.0), Unit::Px(50.0), Unit::Px(50.0), Unit::Px(50.0))
                            .position(Position::Absolute)
                            .width(Unit::Px(200.0))
                            .height(Unit::Px(200.0))
                            .id("green")
                            .push(
                                Overlay::new()
                                    .background(palette::css::YELLOW)
                                    .inset(Unit::Px(50.0), Unit::Px(50.0), Unit::Px(50.0), Unit::Px(50.0))
                                    .position(Position::Absolute)
                                    .width(Unit::Px(100.0))
                                    .height(Unit::Px(100.0))
                                    .id("yellow"),
                            )
                            .push(
                                Container::new()
                                    .background(palette::css::PINK)
                                    .inset(Unit::Px(25.0), Unit::Px(25.0), Unit::Px(25.0), Unit::Px(25.0))
                                    .position(Position::Absolute)
                                    .width(Unit::Px(50.0))
                                    .height(Unit::Px(50.0))
                                    .id("pink"),
                            ),
                    )
                    .push(
                        Container::new()
                            .background(palette::css::BLUE)
                            .inset(Unit::Px(0.0), Unit::Px(0.0), Unit::Px(0.0), Unit::Px(0.0))
                            .position(Position::Absolute)
                            .width(Unit::Px(100.0))
                            .height(Unit::Px(100.0))
                            .id("blue"),
                    ),
            )
            .component()
    }

    fn update(
        &mut self,
        _global_state: &mut Self::GlobalState,
        _props: &Self::Props,
        event: &mut Event,
        _message: &Message,
    ) {
        println!("{:?}", event.window);
        self.hovered_element_id = event.target.clone();

        if self.hovered_element_id.is_some() {
            event.prevent_propagate();
        }
    }
}

#[allow(dead_code)]
#[cfg(not(target_os = "android"))]
fn main() {
    setup_logging();

    craft_main_with_options(
        OverlayExample::component(),
        Box::new(()),
        Some(CraftOptions {
            renderer: RendererType::default(),
            window_title: "Overlay".to_string(),
            ..Default::default()
        }),
    );
}

use craft::elements::Overlay;
use craft::style::{Position, Unit};
#[cfg(target_os = "android")]
use craft::AndroidApp;
use craft::WindowContext;

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    setup_logging();

    craft_main_with_options(
        OverlayExample::component(),
        Box::new(()),
        Some(CraftOptions {
            renderer: RendererType::default(),
            window_title: "Overlay".to_string(),
            ..Default::default()
        }),
        app,
    );
}
