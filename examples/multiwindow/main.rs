use std::cell::RefCell;
use std::rc::Rc;

use craft_retained::elements::{Container, Element, Text, Window};
use craft_retained::events::ui_events::pointer::PointerButton;
use craft_retained::style::{AlignItems, Display, FlexDirection, JustifyContent, Unit};
use craft_retained::{Color, rgb};

#[derive(Default, Clone, Copy)]
pub struct Counter {
    count: i64,
}

impl Counter {
    fn change(&mut self, delta: i64) {
        self.count += delta;
    }

    fn count(&self) -> i64 {
        self.count
    }
}

fn create_button(
    label: &str,
    base_color: Color,
    delta: i64,
    state: Rc<RefCell<Counter>>,
    count_text: Rc<RefCell<Text>>,
) -> Rc<RefCell<Container>> {
    let border_color = rgb(0, 0, 0);
    let label = Text::new(label);
    label.borrow_mut().font_size(24.0).color(Color::WHITE).selectable(false);
    let container = Container::new();
    container
        .borrow_mut()
        .border_width(Unit::Px(1.0), Unit::Px(2.0), Unit::Px(3.0), Unit::Px(4.0))
        .border_color(border_color, border_color, border_color, border_color)
        .border_radius((10.0, 10.0), (10.0, 10.0), (10.0, 10.0), (10.0, 10.0))
        .padding(Unit::Px(15.0), Unit::Px(30.0), Unit::Px(15.0), Unit::Px(30.0))
        .display(Display::Flex)
        .justify_content(Some(JustifyContent::Center))
        .align_items(Some(AlignItems::Center))
        .background_color(base_color)
        .on_pointer_button_up(Rc::new(move |event, pointer_button_event| {
            if pointer_button_event.button == Some(PointerButton::Primary) {
                state.borrow_mut().change(delta);
                count_text
                    .borrow_mut()
                    .text(&format!("Count: {}", state.borrow().count()));
                event.prevent_propagate();
            }
        }))
        .push(label);
    container
}

pub fn counter() -> Rc<RefCell<dyn Element>> {
    let count = Rc::new(RefCell::new(Counter::default()));

    let window = Window::new();

    let count_text = Text::new(&format!("Count: {}", count.borrow().count()));

    let button = Container::new();
    button
        .borrow_mut()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .gap(Unit::Px(20.0), Unit::Px(20.0))
        .push(create_button(
            "-",
            rgb(244, 67, 54),
            -1,
            count.clone(),
            count_text.clone(),
        ))
        .push(create_button(
            "+",
            rgb(76, 175, 80),
            1,
            count.clone(),
            count_text.clone(),
        ));

    window
        .borrow_mut()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .justify_content(Some(JustifyContent::Center))
        .align_items(Some(AlignItems::Center))
        .width(Unit::Percentage(100.0))
        .height(Unit::Percentage(100.0))
        .gap(Unit::Px(20.0), Unit::Px(20.0))
        .push(count_text)
        .font_size(72.0)
        .color(rgb(50, 50, 50))
        .push(button);

    let root = Container::new();
    root.borrow_mut().push(window);

    root
}

fn main() {
    let _counter1 = counter();
    let _counter2 = counter();

    //let window_2 = Window::new();

    use craft_retained::CraftOptions;
    util::setup_logging();
    craft_retained::craft_main(CraftOptions::basic("Counter"));
}
