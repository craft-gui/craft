use retgui::drivers::headless::run;
use retgui::elements::{Element, Radio, RadioGroup, Text, Window};
use retgui::geometry::Size;
use retgui::style::FlexDirection;
use retgui::{RendererType, px};

#[cfg(test)]
mod test_utils;

#[test]
fn switches_from_red_to_green() {
    run(
        "radio_switches_from_red_to_green",
        |elements| {
            let active_color = elements.insert_state("red".to_string());
            let red_label = Text::new(elements, "Red");
            let red = Radio::new(elements, "red", "red", active_color)
                .edit(elements)
                .push(red_label)
                .finish();
            let green_label = Text::new(elements, "Green");
            let green = Radio::new(elements, "green", "green", active_color)
                .edit(elements)
                .push(green_label)
                .finish();
            let group = RadioGroup::new(elements, "Color")
                .edit(elements)
                .flex_direction(FlexDirection::Column)
                .gap(px(8), px(8))
                .push(red)
                .push(green)
                .finish();
            let window = Window::new_with_renderer(elements, "Radio buttons", RendererType::VelloCPU)
                .edit(elements)
                .width(px(240))
                .height(px(120))
                .push(group)
                .finish();
            (active_color, green, window)
        },
        |test, (active_color, green, window)| {
            test.open(&window, Size::new(240.0, 120.0));
            assert_eq!(test.elements().state(active_color).as_str(), "red");

            test.click(&green);

            assert_eq!(test.elements().state(active_color).as_str(), "green");
            test_utils::check_snapshot(
                test_utils::screenshot_rgb(test, &window),
                "switches_from_red_to_green.png",
            );
        },
    );
}
