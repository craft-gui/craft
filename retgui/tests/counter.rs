use retgui::drivers::headless::run;
use retgui::elements::{Container, Element, State, Text, Window};
use retgui::events::{Event, PointerButton};
use retgui::geometry::Size;
use retgui::style::{AlignItems, FlexDirection, JustifyContent};
use retgui::{App, Color, RendererType, pct, px, rgb};

fn create_button(
    app: &mut App,
    label: &str,
    base_color: Color,
    delta: i64,
    state: State<i64>,
    count_text: Text,
) -> Container {
    let border_color = rgb(0, 0, 0);
    let label = Text::new(app, label)
        .edit(app)
        .font_size(24.0)
        .color(Color::WHITE)
        .selectable(false)
        .finish();
    Container::new(app)
        .edit(app)
        .border_width(px(1), px(2), px(3), px(4))
        .border_color(border_color, border_color, border_color, border_color)
        .border_radius((10.0, 10.0), (10.0, 10.0), (10.0, 10.0), (10.0, 10.0))
        .padding(px(15), px(30), px(15), px(30))
        .justify_content(JustifyContent::Center)
        .background_color(base_color)
        .add_pointer_button_up_listener(move |event, app| {
            if event.button == Some(PointerButton::Left) {
                let count = state.update(app, |count| {
                    *count += delta;
                    *count
                });
                count_text.edit(app).text(&format!("Count: {count}")).finish();
                event.stop_propagation();
            }
        })
        .push(label)
        .finish()
}

#[cfg(test)]
mod test_utils;

#[test]
fn counter() {
    run(
        "counter_test",
        |app| {
            let count = app.insert_state(0_i64);
            let count_text = Text::new(app, "Count: 0");
            let subtract = create_button(app, "-", rgb(244, 67, 54), -1, count, count_text);
            let add_button = create_button(app, "+", rgb(76, 175, 80), 1, count, count_text);
            let buttons = Container::new(app)
                .edit(app)
                .gap(px(20), px(20))
                .push(subtract)
                .push(add_button)
                .finish();
            let window = Window::new_with_renderer(app, "Counter", RendererType::VelloCPU)
                .edit(app)
                .flex_direction(FlexDirection::Column)
                .justify_content(JustifyContent::Center)
                .align_items(AlignItems::Center)
                .width(pct(100))
                .height(pct(100))
                .gap(px(20), px(20))
                .push(count_text)
                .push(buttons)
                .finish();
            (count, add_button, window)
        },
        |test, (count, add_button, window)| {
            test.open(&window, Size::new(800.0, 600.0));
            for _ in 0..3 {
                test.click(&add_button);
            }

            assert_eq!(*test.app().state(count), 3);
            test_utils::check_snapshot(test_utils::screenshot_rgb(test, &window), "counter.png");
        },
    );
}
