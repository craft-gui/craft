use retgui::elements::{Container, Element, Elements, State, Text, Window};
use retgui::events::Event;
use retgui::style::{AlignItems, Display, FlexDirection, JustifyContent, Unit};
use retgui::{Color, rgb};

#[derive(Default, Clone, Copy)]
pub struct Counter {
    count: i64,
}

impl Counter {
    fn change(&mut self, delta: i64) -> bool {
        self.count += delta;
        self.count >= 10
    }

    fn count(&self) -> i64 {
        self.count
    }
}

fn create_button(
    label: &str,
    base_color: Color,
    delta: i64,
    elements: &mut Elements,
    state: State<Counter>,
    count_text: Text,
) -> Container {
    let label = Text::new(elements, label)
        .edit(elements)
        .font_size(24.0)
        .color(Color::WHITE)
        .selectable(false)
        .finish();
    Container::new(elements)
        .edit(elements)
        .border_width(Unit::Px(1.0), Unit::Px(2.0), Unit::Px(3.0), Unit::Px(4.0))
        .border_color_all(rgb(0, 0, 0))
        .border_radius_all((10.0, 10.0))
        .padding(Unit::Px(15.0), Unit::Px(30.0), Unit::Px(15.0), Unit::Px(30.0))
        .display(Display::Flex)
        .justify_content(JustifyContent::Center)
        .align_items(AlignItems::Center)
        .background_color(base_color)
        .on_click(move |event, elements| {
            let (create_window, count) = state.update(elements, |state| {
                let create_window = state.change(delta);
                (create_window, state.count())
            });
            count_text.edit(elements).text(&format!("Count: {count}")).finish();
            if create_window {
                counter(elements);
            }
            event.stop_propagation();
        })
        .push(label)
        .finish()
}

pub fn counter(elements: &mut Elements) -> Window {
    let count = elements.insert_state(Counter::default());
    let count_text = Text::new(elements, "Count: 0");
    let subtract = create_button("-", rgb(244, 67, 54), -1, elements, count, count_text);
    let add = create_button("+", rgb(76, 175, 80), 1, elements, count, count_text);
    let button = Container::new(elements)
        .edit(elements)
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .column_gap(Unit::Px(20.0))
        .push(subtract)
        .push(add)
        .finish();

    Window::new(elements, "MultiWindow")
        .edit(elements)
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .justify_content(JustifyContent::Center)
        .align_items(AlignItems::Center)
        .width(Unit::Percentage(100.0))
        .height(Unit::Percentage(100.0))
        .row_gap(Unit::Px(20.0))
        .push(count_text)
        .font_size(72.0)
        .color(rgb(50, 50, 50))
        .push(button)
        .finish()
}

fn main() {
    let mut elements = Elements::new();
    let _counter1 = counter(&mut elements);
    use retgui::RetGuiOptions;
    util::setup_logging();
    retgui::retgui_main(elements, RetGuiOptions::basic("Counter"));
}
