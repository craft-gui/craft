use craft_retained::elements::core::ElementData;
use craft_retained::elements::Element;
use craft_retained::events::ui_events::pointer::PointerId;
use craft_retained::events::Event;
use craft_retained::style::{AlignItems, Display, FlexDirection, JustifyContent, Overflow, Position, Unit};
use craft_retained::{
    elements::{Container, Text},
    Color,
};
use std::cell::RefCell;
use std::rc::Rc;

fn title(txt: &str) -> Rc<RefCell<Text>> {
    let title = Text::new(txt);
    title.borrow_mut().font_size(24.0).padding(Unit::Px(0.0), Unit::Px(0.0), Unit::Px(25.0), Unit::Px(0.0));

    title
}

fn event_log() -> (Rc<RefCell<Container>>, Rc<dyn Fn(String)>) {
    let container = Container::new();
    container
        .borrow_mut()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .gap(Unit::Px(20.0), Unit::Px(20.0));

    let event_log = Container::new();
    let border_color = Color::from_rgb8(99, 99, 99);
    event_log
        .borrow_mut()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .overflow(Overflow::Visible, Overflow::Scroll)
        .width(Unit::Px(300.0))
        .height(Unit::Px(200.0))
        .max_width(Unit::Px(300.0))
        .max_height(Unit::Px(200.0))
        .border_width(Unit::Px(1.0), Unit::Px(1.0), Unit::Px(1.0), Unit::Px(1.0))
        .margin(Unit::Px(25.0), Unit::Px(0.0), Unit::Px(0.0), Unit::Px(0.0))
        .border_color(border_color, border_color, border_color, border_color);

    let event_log_copy = event_log.clone();
    let push_text = Rc::new(move |string: String| {
        event_log_copy.borrow_mut().push(Text::new(&string));
    });

    let event_log_copy = event_log.clone();
    let clear_log = Text::new("Clear");
    clear_log
        .borrow_mut()
        .background_color(Color::from_rgb8(210, 210, 215))
        .border_width(Unit::Px(1.0), Unit::Px(1.0), Unit::Px(1.0), Unit::Px(1.0))
        .border_radius((6.0, 6.0), (6.0, 6.0), (6.0, 6.0), (6.0, 6.0))
        .padding(Unit::Px(10.0), Unit::Px(25.0), Unit::Px(10.0), Unit::Px(25.0))
        .width(Unit::Px(90.0))
        .on_pointer_button_down(Rc::new(move |_e, _pb_event| {
            let to_remove = event_log_copy.borrow().children().to_vec();
            for child in to_remove {
                event_log_copy.borrow_mut().remove_child(child).expect("Failed to remove child!");
            }
        }));

    container.borrow_mut().push(event_log);

    container.borrow_mut().push(clear_log);

    (container, push_text)
}

fn pointer_capture_example() -> Rc<RefCell<Container>> {
    let container = Container::new();
    let container_padding = 20.0;
    container.borrow_mut().display(Display::Flex).flex_direction(FlexDirection::Column).padding(
        Unit::Px(container_padding),
        Unit::Px(container_padding),
        Unit::Px(container_padding),
        Unit::Px(container_padding),
    );

    let draggable_text = Text::new("Draggable");

    draggable_text
        .clone()
        .borrow_mut()
        .display(Display::Flex)
        .width(Unit::Px(100.0))
        .color(Color::WHITE)
        .background_color(Color::from_rgba8(40, 40, 255, 100))
        .on_pointer_button_down(Rc::new(|e, _pb_event| {
            e.target.borrow_mut().set_pointer_capture(PointerId::new(1).unwrap());
        }));

    let (event_log, push_text) = event_log();
    let push_text_clone = push_text.clone();

    let draggable_text_clone = draggable_text.clone();
    draggable_text.borrow_mut().on_pointer_moved(Rc::new(move |e, pointer_moved_event| {
        let mouse_y = pointer_moved_event.current.logical_position().x as f32;
        let half_size = draggable_text_clone.borrow_mut().computed_box_transformed().size.width / 2.0;
        if draggable_text_clone.borrow_mut().has_pointer_capture(PointerId::new(1).unwrap()) {
            draggable_text_clone.borrow_mut().position(Position::Relative).inset(
                Unit::Px(0.0),
                Unit::Px(0.0),
                Unit::Px(0.0),
                Unit::Px(mouse_y - half_size - container_padding),
            );
        }
        e.prevent_defaults();
    }));

    draggable_text.borrow_mut().on_got_pointer_capture(Rc::new(move |_e| {
        push_text_clone("Got Pointer Capture".to_string());
    }));

    let push_text_clone = push_text.clone();
    draggable_text.borrow_mut().on_lost_pointer_capture(Rc::new(move |_e| {
        push_text_clone("Lost Pointer Capture".to_string());
    }));

    container.borrow_mut().push(title("Pointer Capture"));
    container.borrow_mut().push(draggable_text);
    container.borrow_mut().push(event_log);

    container
}

fn pointer_enter_leave_example() -> Rc<RefCell<Container>> {
    let container = Container::new();
    container.borrow_mut().display(Display::Flex).flex_direction(FlexDirection::Column).padding(
        Unit::Px(20.0),
        Unit::Px(20.0),
        Unit::Px(20.0),
        Unit::Px(20.0),
    );

    let (event_log, push_text) = event_log();

    let parent = Container::new();
    let pointer_enter_leave_log = move |is_enter: bool, node_name: &'static str| {
        let push_text_clone_2 = push_text.clone();
        let pointer_event_name = if is_enter { "Pointer Enter" } else { "Pointer Leave" };
        return Rc::new(move |_event: &mut Event| {
            push_text_clone_2(format!("{}: {}", pointer_event_name, node_name));
        });
    };

    parent
        .borrow_mut()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .align_items(Some(AlignItems::Center))
        .justify_content(Some(JustifyContent::Center))
        .width(Unit::Px(250.0))
        .height(Unit::Px(250.0))
        .background_color(Color::from_rgba8(10, 10, 255, 150))
        .on_pointer_enter(pointer_enter_leave_log(true, "Parent").clone())
        .on_pointer_leave(pointer_enter_leave_log(false, "Parent").clone());

    let child_container = Container::new();
    child_container
        .borrow_mut()
        .width(Unit::Px(125.0))
        .height(Unit::Px(125.0))
        .background_color(Color::from_rgba8(255, 10, 10, 150))
        .on_pointer_enter(pointer_enter_leave_log(true, "Child").clone())
        .on_pointer_leave(pointer_enter_leave_log(false, "Child").clone());

    parent.borrow_mut().push(child_container);

    container.borrow_mut().push(title("Pointer Enter + Leave"));
    container.borrow_mut().push(parent);
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
    root.borrow_mut()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .overflow(Overflow::Visible, Overflow::Scroll)
        .max_height(Unit::Percentage(100.0))
        .gap(Unit::Px(50.0), Unit::Px(50.0));
    root.borrow_mut().push(pointer_capture_example());
    root.borrow_mut().push(pointer_enter_leave_example());

    use craft_retained::CraftOptions;
    //util::setup_logging();
    craft_retained::craft_main(root, CraftOptions::basic("Pointer Events"));
}
