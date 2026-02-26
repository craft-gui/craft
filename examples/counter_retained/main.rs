use std::cell::RefCell;
use std::rc::Rc;

use craft_retained::elements::{Container, Element, Text, Window};
use craft_retained::events::ui_events::pointer::PointerButton;
use craft_retained::style::{AlignItems, BoxShadow, FlexDirection, JustifyContent};
use craft_retained::{Color, CraftOptions, pct, px, rgb, rgba};

fn create_button(label: &str, base_color: Color, delta: i64, state: Rc<RefCell<i64>>, count_text: Text) -> Container {
    let border_color = rgb(0, 0, 0);
    Container::new()
        .box_shadows(vec![
            BoxShadow::new(false, 0.0, 5.0, 5.0, 0.0, rgba(0, 0, 0, 200)),

            // 2. The Big Drop (Heavy and visible)
            // Offset is 25.0 - this will be very obvious.
            BoxShadow::new(false, 0.0, 25.0, 35.0, 0.0, rgba(0, 0, 0, 150)),

            // 3. Inner "Plastic" Highlight
            BoxShadow::new(true, 0.0, 4.0, 4.0, 0.0, rgba(255, 255, 255, 120)),
        ])
        .border_width(px(0), px(0), px(0), px(0))
        .border_color(border_color, border_color, border_color, border_color)
        .border_radius((8.0, 8.0), (8.0, 8.0), (8.0, 8.0), (8.0, 8.0))
        .padding(px(15), px(30), px(15), px(30))
        .justify_content(Some(JustifyContent::Center))
        .background_color(base_color)
        .on_pointer_button_up(Rc::new(move |event, pointer_button_event| {
            if pointer_button_event.button == Some(PointerButton::Primary) {
                *state.borrow_mut() += delta;
                count_text.clone().text(&format!("Count: {}", state.borrow()));
                event.prevent_propagate();
            }
        }))
        .push(Text::new(label).font_size(24.0).color(Color::WHITE).selectable(false))
}

fn main() {
    let count = Rc::new(RefCell::new(0));
    let count_text = Text::new(&format!("Count: {}", count.borrow()));

    Window::new()
        .flex_direction(FlexDirection::Column)
        .justify_content(Some(JustifyContent::Center))
        .align_items(Some(AlignItems::Center))
        .width(pct(100))
        .height(pct(100))
        .gap(px(20), px(20))
        .push(count_text.clone())
        .push({
            Container::new()
                .gap(px(20), px(20))
                .push(create_button(
                    "-",
                    rgb(244, 63, 94),
                    -1,
                    count.clone(),
                    count_text.clone(),
                ))
                .push(create_button(
                    "+",
                    rgb(16, 185, 129),
                    1,
                    count.clone(),
                    count_text.clone(),
                ))
        });

    craft_retained::craft_main(CraftOptions::basic("Counter"));
}
