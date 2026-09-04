use retgui::elements::{Container, Element, Elements, State, Text, Window};
use retgui::events::Event;
use retgui::style::{AlignItems, BoxShadow, FlexDirection, JustifyContent};
use retgui::{Color, RetGuiOptions, pct, px, retgui_main, rgb, rgba};

use util::setup_logging;

fn create_button(
    elements: &mut Elements,
    label: &str,
    base_color: Color,
    delta: i64,
    state: State<i64>,
    count_text: Text,
) -> Container {
    let border_color = rgb(0, 0, 0);
    let label = Text::new(elements, label)
        .edit(elements)
        .font_size(24.0)
        .color(Color::WHITE)
        .selectable(false)
        .finish();
    Container::new(elements)
        .edit(elements)
        .box_shadows(vec![
            BoxShadow::new(false, 0.0, 5.0, 5.0, 0.0, rgba(0, 0, 0, 200)),
            BoxShadow::new(false, 0.0, 25.0, 35.0, 0.0, rgba(0, 0, 0, 150)),
            BoxShadow::new(true, 0.0, 4.0, 4.0, 0.0, rgba(255, 255, 255, 120)),
        ])
        .border_width_all(px(0))
        .border_color_all(border_color)
        .border_radius_all((8.0, 8.0))
        .padding(px(15), px(30), px(15), px(30))
        .justify_content(JustifyContent::Center)
        .background_color(base_color)
        .add_click_listener(move |event, elements| {
            let count = state.update(elements, |count| {
                *count += delta;
                *count
            });
            count_text.edit(elements).text(&format!("Count: {count}")).finish();
            event.stop_propagation();
        })
        .push(label)
        .finish()
}

pub fn counter(elements: &mut Elements) -> Container {
    let count = elements.insert_state(0_i64);
    let count_text = Text::new(elements, "Count: 0");
    let subtract = create_button(elements, "-", rgb(244, 63, 94), -1, count, count_text);
    let add = create_button(elements, "+", rgb(16, 185, 129), 1, count, count_text);
    let buttons = Container::new(elements)
        .edit(elements)
        .column_gap(px(20))
        .push(subtract)
        .push(add)
        .finish();

    Container::new(elements)
        .edit(elements)
        .flex_direction(FlexDirection::Column)
        .justify_content(JustifyContent::Center)
        .align_items(AlignItems::Center)
        .width(pct(100))
        .height(pct(100))
        .row_gap(px(20))
        .push(count_text)
        .push(buttons)
        .finish()
}

pub fn main() {
    setup_logging();
    let mut elements = Elements::new();
    let content = counter(&mut elements);
    Window::new(&mut elements, "Counter")
        .edit(&mut elements)
        .width(pct(100))
        .height(pct(100))
        .push(content)
        .finish();
    retgui_main(elements, RetGuiOptions::basic("Counter"));
}
