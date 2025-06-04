use craft::components::ComponentId;
use craft::components::{Component, ComponentSpecification, Event};
use craft::elements::Container;
use craft::elements::ElementStyles;
use craft::events::Message;
use craft::palette;
use craft::style::{BoxSizing, Display, FlexDirection, Overflow};
use craft::CraftOptions;
use craft::WindowContext;
use craft::{craft_main, Color};
use util::setup_logging;

#[derive(Default, Copy, Clone)]
pub struct EventsExample {}

impl Component for EventsExample {
    type GlobalState = ();

    type Props = ();

    type Message = ();

    fn view(
        &self,
        _global_state: &Self::GlobalState,
        _props: &Self::Props,
        _children: Vec<ComponentSpecification>,
        _id: ComponentId,
        _window: &WindowContext,
    ) -> ComponentSpecification {
        let scroll_example = Container::new()
            .display(Display::Block)
            .width("200px")
            .height("100px")
            .overflow_y(Overflow::Scroll)
            .border_width(4, 4, 4, 4)
            .border_color(Color::BLACK)
            .push(
                Container::new()
                    .height("300px")
                    .display(Display::Block)
                    .background(Color::from_rgb8(170, 170, 255))
                    .component(),
            )
            .component();

        let padded_scroll = Container::new()
            .display(Display::Block)
            .width("200px")
            .height("100px")
            .padding("20px", "20px", "20px", "20px")
            .overflow_y(Overflow::Scroll)
            .border_width(10, 10, 10, 10)
            .border_color(Color::BLACK)
            .box_sizing(BoxSizing::BorderBox)
            .push(
                Container::new().height("300px").display(Display::Block).background(palette::css::LAVENDER).component(),
            )
            .component();

        let nested_scroll = Container::new()
            .width("300px")
            .height("150px")
            .display(Display::Block)
            .overflow_y(Overflow::Scroll)
            .padding(20, 20, 20, 20)
            .border_width(2, 2, 2, 2)
            .border_color(palette::css::PURPLE)
            .push(
                Container::new()
                    .display(Display::Block)
                    .width("220px")
                    .height("300px")
                    .overflow_y(Overflow::Scroll)
                    .border_width(4, 4, 4, 4)
                    .border_color(palette::css::GOLD)
                    .padding(10, 10, 10, 10)
                    .background(palette::css::GREEN)
                    .push(
                        Container::new()
                            .display(Display::Block)
                            .width("150px")
                            .height("900px")
                            .background(palette::css::BLUE)
                            .component(),
                    )
                    .component(),
            )
            .component();

        Container::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            // .overflow_y(Overflow::Scroll)
            .height("100%")
            .max_height("100%")
            .gap("50px")
            .push(scroll_example)
            .push(padded_scroll)
            .push(nested_scroll)
            .component()
    }
    fn update(
        &mut self,
        _global_state: &mut Self::GlobalState,
        _props: &Self::Props,
        event: &mut Event,
        message: &Message,
    ) {
        if message.clicked() {
            let target = if let Some(target) = event.target { target.get_id().clone() } else { None };
            let current_target =
                if let Some(current_target) = event.current_target { current_target.get_id().clone() } else { None };
            println!("Target: {:?}, Current Target: {:?}", target, current_target);
        }
    }
}

fn main() {
    setup_logging();
    craft_main(EventsExample::component(), (), CraftOptions::basic("Events"));
}
