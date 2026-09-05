use retgui::elements::{Container, Element, Elements, Text, Window};
use retgui::events::{Event, PointerEnterEvent, PointerLeaveEvent};
use retgui::style::{AlignItems, Display, FlexDirection, JustifyContent, Overflow, Position, Unit};
use retgui::{Color, pct};

#[derive(Clone, Copy)]
struct EventLog {
    view: Container,
    entries: Container,
}

impl EventLog {
    fn push(self, elements: &mut Elements, message: impl AsRef<str>) {
        let text = Text::new(elements, message.as_ref());
        self.entries.push(elements, text);
    }
}

fn title(elements: &mut Elements, txt: &str) -> Text {
    Text::new(elements, txt)
        .edit(elements)
        .font_size(24.0)
        .padding(Unit::Px(0.0), Unit::Px(0.0), Unit::Px(25.0), Unit::Px(0.0))
        .finish()
}

fn event_log(elements: &mut Elements) -> EventLog {
    let entries = Container::new(elements)
        .edit(elements)
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .overflow(Overflow::Visible, Overflow::Scroll)
        .width(Unit::Px(300.0))
        .height(Unit::Px(200.0))
        .max_width(Unit::Px(300.0))
        .max_height(Unit::Px(200.0))
        .border_width_all(Unit::Px(1.0))
        .margin(Unit::Px(25.0), Unit::Px(0.0), Unit::Px(0.0), Unit::Px(0.0))
        .border_color_all(Color::from_rgb8(99, 99, 99))
        .finish();

    let clear_log = Text::new(elements, "Clear")
        .edit(elements)
        .background_color(Color::from_rgb8(210, 210, 215))
        .border_width_all(Unit::Px(1.0))
        .border_radius_all((6.0, 6.0))
        .padding(Unit::Px(10.0), Unit::Px(25.0), Unit::Px(10.0), Unit::Px(25.0))
        .width(Unit::Px(90.0))
        .add_click_listener(move |_event, elements| {
            entries.delete_all_children(elements);
        })
        .finish();

    let container = Container::new(elements)
        .edit(elements)
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .row_gap(Unit::Px(20.0))
        .push(entries)
        .push(clear_log)
        .finish();

    EventLog {
        view: container,
        entries,
    }
}

fn pointer_capture_example(elements: &mut Elements) -> Container {
    let container_padding = 20.0;

    let draggable_text = Text::new(elements, "Draggable");
    let event_log = event_log(elements);

    let draggable_text = draggable_text
        .edit(elements)
        .display(Display::Flex)
        .width(Unit::Px(100.0))
        .color(Color::WHITE)
        .background_color(Color::from_rgba8(40, 40, 255, 100))
        .add_pointer_button_down_listener(|event, elements| {
            event
                .target()
                .set_pointer_capture(elements, event.pointer.pointer_id.unwrap());
        })
        .add_pointer_moved_listener(move |event, elements| {
            let mouse_x = event.current.logical_position().x as f32;
            let half_width = draggable_text.computed_box_transformed(elements).size.width / 2.0;
            if draggable_text.has_pointer_capture(elements, event.pointer.pointer_id.unwrap()) {
                draggable_text
                    .edit(elements)
                    .position(Position::Relative)
                    .inset(
                        Unit::Px(0.0),
                        Unit::Px(0.0),
                        Unit::Px(0.0),
                        Unit::Px(mouse_x - half_width - container_padding),
                    )
                    .finish();
            }
            event.prevent_default();
        })
        .add_lost_pointer_capture_listener(move |_event, elements| {
            event_log.push(elements, "Lost Pointer Capture");
        })
        .add_got_pointer_capture_listener(move |_event, elements| {
            event_log.push(elements, "Got Pointer Capture");
        })
        .finish();

    let heading = title(elements, "Pointer Capture");
    Container::new(elements)
        .edit(elements)
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .padding_all(Unit::Px(container_padding))
        .push(heading)
        .push(draggable_text)
        .push(event_log.view)
        .finish()
}

fn pointer_enter_leave_example(elements: &mut Elements) -> Container {
    let event_log = event_log(elements);

    let pointer_enter_log = move |element_name: &'static str| {
        move |_event: &mut PointerEnterEvent, elements: &mut Elements| {
            event_log.push(elements, format!("Pointer Enter: {element_name}"));
        }
    };
    let pointer_leave_log = move |element_name: &'static str| {
        move |_event: &mut PointerLeaveEvent, elements: &mut Elements| {
            event_log.push(elements, format!("Pointer Leave: {element_name}"));
        }
    };

    let parent = Container::new(elements)
        .edit(elements)
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::Center)
        .width(Unit::Px(250.0))
        .height(Unit::Px(250.0))
        .background_color(Color::from_rgba8(10, 10, 255, 150))
        .add_pointer_enter_listener(pointer_enter_log("Parent"))
        .add_pointer_leave_listener(pointer_leave_log("Parent"))
        .finish();

    let child_container = Container::new(elements)
        .edit(elements)
        .width(Unit::Px(125.0))
        .height(Unit::Px(125.0))
        .background_color(Color::from_rgba8(255, 10, 10, 150))
        .add_pointer_enter_listener(pointer_enter_log("Child"))
        .add_pointer_leave_listener(pointer_leave_log("Child"))
        .finish();

    parent.push(elements, child_container);
    let heading = title(elements, "Pointer Enter + Leave");

    Container::new(elements)
        .edit(elements)
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .padding_all(Unit::Px(20.0))
        .push(heading)
        .push(parent)
        .push(event_log.view)
        .finish()
}

pub fn pointer_events(elements: &mut Elements) -> Container {
    let capture = pointer_capture_example(elements);
    let enter_leave = pointer_enter_leave_example(elements);
    Container::new(elements)
        .edit(elements)
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .overflow_y(Overflow::Scroll)
        .max_height(Unit::Percentage(100.0))
        .width(pct(100))
        .height(pct(100))
        .row_gap(Unit::Px(50.0))
        .push(capture)
        .push(enter_leave)
        .finish()
}

#[allow(unused)]
#[cfg(not(target_os = "android"))]
fn main() {
    let mut elements = Elements::new();
    let content = pointer_events(&mut elements);
    Window::new(&mut elements, "Pointer Events")
        .edit(&mut elements)
        .width(pct(100))
        .height(pct(100))
        .push(content)
        .finish();

    use retgui::RetGuiOptions;
    //util::setup_logging();
    retgui::retgui_main(elements, RetGuiOptions::basic("Pointer Events"));
}
