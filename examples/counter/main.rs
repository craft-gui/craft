use craft::components::{Component, ComponentId, ComponentSpecification, Event};
use craft::elements::ElementStyles;
use craft::elements::{Container, Text};
use craft::events::PointerButton;
use craft::style::Display;
use craft::style::{AlignItems, FlexDirection, JustifyContent};
use craft::Color;
use craft::CraftOptions;
use craft::RendererType;
use craft::{craft_main_with_options, WindowContext};
use util::setup_logging;

#[derive(Default, Copy, Clone)]
pub struct Counter {
    count: i64,
}

pub enum CounterMessage {}

impl Component for Counter {
    type GlobalState = ();
    type Props = ();
    type Message = CounterMessage;

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
            .gap("20px")
            .push(
                Text::new(format!("Count:  {}", self.count).as_str())
                    .font_size(72.0)
                    .color(Color::from_rgb8(50, 50, 50)),
            )
            .push(
                Container::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Row)
                    .gap("20px")
                    .push(
                        create_button("-", "decrement", Color::from_rgb8(244, 67, 54), Color::from_rgb8(211, 47, 47))
                            .on_click(|state: &mut Self, event: &mut Event, pointer_button: &PointerButton| {
                                if pointer_button.clicked() {
                                    state.count -= 1;
                                    event.prevent_propagate();
                                }
                            }),
                    )
                    .push(
                        create_button("+", "increment", Color::from_rgb8(76, 175, 80), Color::from_rgb8(67, 160, 71))
                            .on_click(|state: &mut Self, event: &mut Event, pointer_button: &PointerButton| {
                                if pointer_button.clicked() {
                                    state.count += 1;
                                    event.prevent_propagate();
                                }
                            }),
                    ),
            )
            .component()
    }
}

fn create_button(label: &str, id: &str, color: Color, hover_color: Color) -> Container {
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
        .push(
            Text::new(label)
                .id(id)
                .font_size(24.0)
                .color(Color::WHITE)
                .width("100%")
                .height("100%")
                .disable_selection(),
        )
        .id(id)
}

#[allow(dead_code)]
#[cfg(not(target_os = "android"))]
fn main() {
    setup_logging();

    craft_main_with_options(
        Counter::component(),
        Box::new(()),
        Some(CraftOptions {
            renderer: RendererType::default(),
            window_title: "Counter".to_string(),
            ..Default::default()
        }),
    );
}

#[cfg(target_os = "android")]
use craft::AndroidApp;

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    setup_logging();

    craft_main_with_options(
        Counter::component(),
        Box::new(()),
        Some(CraftOptions {
            renderer: RendererType::default(),
            window_title: "Counter".to_string(),
            ..Default::default()
        }),
        app,
    );
}
