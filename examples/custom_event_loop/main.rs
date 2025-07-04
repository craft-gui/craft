use crate::custom_event_loop::CraftWinitState;
use craft::components::Context;
use craft::elements::Canvas;
use craft::events::ui_events::pointer::PointerButtonUpdate;
use craft::{
    components::{Component, ComponentSpecification},
    elements::{Container, ElementStyles, Text},
    rgb,
    style::{AlignItems, Display, FlexDirection, JustifyContent},
    Color,
};
use craft::setup_craft;
use winit::event_loop::EventLoop;

mod custom_event_loop;
mod wgpu_triangle;

#[derive(Default)]
pub struct Counter {
    count: i64,
}

impl Component for Counter {
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
            .gap(20)
            .push(Text::new(&format!("Count: {}", context.state().count)).font_size(72).color(rgb(50, 50, 50)))
            .push(
                Container::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Row)
                    .gap(20)
                    .push(create_button("-", rgb(244, 67, 54), rgb(211, 47, 47), -1))
                    .push(create_button("+", rgb(76, 175, 80), rgb(67, 160, 71), 1)),
            )
            .push(Canvas::new().width("300px").height("300px").min_width("300px").min_height("300px"))
            .component()
    }
}

fn create_button(label: &str, base_color: Color, hover_color: Color, delta: i64) -> Container {
    Container::new()
        .border_width(1, 2, 3, 4)
        .border_color(rgb(0, 0, 0))
        .border_radius(10, 10, 10, 10)
        .padding(15, 30, 15, 30)
        .display(Display::Flex)
        .justify_content(JustifyContent::Center)
        .align_items(AlignItems::Center)
        .background(base_color)
        .hovered()
        .background(hover_color)
        .on_pointer_up(
            move |context: &mut Context<Counter>, pointer_button: &PointerButtonUpdate| {
                if pointer_button.is_primary() {
                    context.state_mut().count += delta;
                    context.event_mut().prevent_propagate();
                }
            },
        )
        .push(Text::new(label).font_size(24).color(Color::WHITE).disable_selection())
}

#[allow(unused)]
#[cfg(not(target_os = "android"))]
fn main() {
    use craft::CraftOptions;

    let application = Counter::component();
    let global_state = ();
    let options = CraftOptions::basic("Custom Event Loop");
    
    
    let event_loop = EventLoop::new().expect("Failed to create winit event loop.");
    let craft_state = setup_craft(application, Box::new(global_state), Some(options));
    let mut winit_craft_state = CraftWinitState::new(craft_state);
    
    event_loop.run_app(&mut winit_craft_state).expect("run_app failed");
}
