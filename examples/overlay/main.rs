use craft::components::{Component, ComponentSpecification, Context};
use craft::elements::ElementStyles;
use craft::elements::{Container, Text};
use craft::style::Display;
use craft::style::{AlignItems, FlexDirection, JustifyContent};
use craft::CraftOptions;
use craft::{craft_main, palette};
use craft::Color;
use util::setup_logging;

#[derive(Default, Clone)]
pub struct OverlayExample {
    hovered_element_id: Option<String>,
}

impl Component for OverlayExample {
    type GlobalState = ();
    type Props = ();
    type Message = ();

    fn view(context: &mut Context<Self>) -> ComponentSpecification {
        Container::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .justify_content(JustifyContent::Center)
            .align_items(AlignItems::Center)
            .width("100%")
            .height("100%")
            .background(Color::from_rgb8(250, 250, 250))
            .push(Text::new(format!("Hovered Element: {:?}", context.state().hovered_element_id).as_str()))
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

    fn update(context: &mut Context<Self>) {
        println!("{:?}", context.window());

        let target = context.target().map(|target| target.get_id()).cloned();
        if let Some(target) = target {
            context.state_mut().hovered_element_id = target.clone().map(|s| s.into());
            if let Some(_id) = target {
                context.event_mut().prevent_propagate();
            }
        } else {
            context.state_mut().hovered_element_id = None;
        }
    }
}

#[allow(dead_code)]
#[cfg(not(target_os = "android"))]
fn main() {
    setup_logging();
    craft_main(OverlayExample::component(), (), CraftOptions::basic("Overlay"));
}

use craft::elements::Overlay;
use craft::style::{Position, Unit};
#[cfg(target_os = "android")]
use craft::AndroidApp;

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    setup_logging();

    craft_main(
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
