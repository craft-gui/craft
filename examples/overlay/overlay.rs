#[path = "../util.rs"]
mod util;

use craft::components::{Component, ComponentId, ComponentSpecification, UpdateResult};
use craft::elements::{Container, Text};
use craft::elements::ElementStyles;
use craft::events::Event;
use craft::style::Display;
use craft::style::{AlignItems, FlexDirection, JustifyContent};
use craft::Color;
use craft::CraftOptions;
use craft::RendererType;
use craft::{craft_main_with_options, palette};

#[derive(Default, Clone)]
pub struct OverlayExample {
    hovered_element_id: Option<String>
}

impl Component for OverlayExample {
    type Props = ();

    fn view_with_no_global_state(
        state: &Self,
        _props: &Self::Props,
        _children: Vec<ComponentSpecification>,
        _id: ComponentId,
    ) -> ComponentSpecification {
        Container::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .justify_content(JustifyContent::Center)
            .align_items(AlignItems::Center)
            .width("100%")
            .height("100%")
            .background(Color::from_rgb8(250, 250, 250))

            .push(Text::new(format!("Hovered Element: {:?}", state.hovered_element_id).as_str()))
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
                                    .id("pink")
                                ,
                            )
                        ,
                    )
                    .push(
                        Container::new()
                            .background(palette::css::BLUE)
                            .inset(Unit::Px(0.0), Unit::Px(0.0), Unit::Px(0.0), Unit::Px(0.0))
                            .position(Position::Absolute)
                            .width(Unit::Px(100.0))
                            .height(Unit::Px(100.0))
                            .id("blue")
                        ,
                    )
            )

            .component()
    }

    fn update_with_no_global_state(state: &mut Self, _props: &Self::Props, event: Event) -> UpdateResult { 
        state.hovered_element_id = event.target;
        
        if state.hovered_element_id.is_some() {
            return UpdateResult::new().prevent_propagate();
        }
        
       UpdateResult::default()
    }
}

fn create_button(label: &str, id: &str, color: Color, hover_color: Color) -> ComponentSpecification {
    Container::new()
        .border_width("1px", "2px", "3px", "4px")
        .border_color(Color::from_rgb8(0, 0, 0))
        .border_radius(10.0, 10.0, 10.0, 10.0)
        .padding("15px", "30px", "15px", "30px")
        .background(color)
        .display(Display::Flex)
        .justify_content(JustifyContent::Center)
        .align_items(AlignItems::Center)
        .hovered()
        .background(hover_color)
        .push(Text::new(label).id(id).font_size(24.0).color(Color::WHITE).width("100%").height("100%"))
        .id(id)
        .component()
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
        }),
    );
}

#[cfg(target_os = "android")]
use craft::AndroidApp;
use craft_core::elements::Overlay;
use craft_core::style::{Position, Unit};
use util::setup_logging;

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
        }),
        app,
    );
}
