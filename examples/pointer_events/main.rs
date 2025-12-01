use craft_retained::elements::Element;
use craft_retained::events::ui_events::pointer::PointerId;
use craft_retained::style::{Display, FlexDirection, Overflow, Unit};
use craft_retained::{
    elements::{Container, Text}
    , Color,
};
use std::cell::RefCell;
use std::rc::Rc;

fn event_log() -> (Rc<RefCell<Container>>, Rc<dyn Fn(String)>) {
    let event_log = Container::new();
    let border_color = Color::from_rgb8(99, 99, 99);
    event_log.borrow_mut()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .overflow(Overflow::Scroll, Overflow::Scroll)
        .width(Unit::Px(300.0))
        .height(Unit::Px(400.0))
        .max_width(Unit::Px(300.0))
        .max_height(Unit::Px(400.0))
        .border_width(Unit::Px(1.0), Unit::Px(1.0), Unit::Px(1.0), Unit::Px(1.0))
        .border_color(border_color, border_color, border_color, border_color)
    ;

    let event_log_copy = event_log.clone();
    let push_text = Rc::new(move |string: String| {
        event_log_copy.borrow_mut().push(Text::new(&string));
    });

    (event_log, push_text)
}

fn pointer_capture_example() -> Rc<RefCell<Container>>{
    let container = Container::new();
    container.borrow_mut()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column);

    let draggable_container = Text::new("Draggable");

    draggable_container.clone().borrow_mut().on_pointer_button_down(Rc::new(|e, _pb_event| {
        e.target.borrow_mut().set_pointer_capture(PointerId::new(1).unwrap());
    }));

    let (event_log, push_text) = event_log();
    let push_text_clone = push_text.clone();
    draggable_container.borrow_mut().on_got_pointer_capture(Rc::new(move |e| {
        push_text_clone("Got Pointer Capture".to_string());
    }));

    let push_text_clone = push_text.clone();
    draggable_container.borrow_mut().on_lost_pointer_capture(Rc::new(move |e| {
        push_text_clone("Lost Pointer Capture".to_string());
    }));

    container.borrow_mut().push(draggable_container);
    container.borrow_mut().push(event_log);

    container
}

#[allow(unused)]
#[cfg(not(target_os = "android"))]
fn main() {
    let pointer_capture_container = Container::new();
    let pointer_capture_event_log = Text::new("");
    let pointer_capture_draggable_container = Text::new("Draggable");

    let root = Container::new();
    root.borrow_mut().push(pointer_capture_example());

    use craft_retained::CraftOptions;
    //util::setup_logging();
    craft_retained::craft_main(root, CraftOptions::basic("Pointer Events"));
}
