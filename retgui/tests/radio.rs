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
            let red = Radio::new(elements, "red", "red", active_color).push(elements, red_label);
            let green_label = Text::new(elements, "Green");
            let green = Radio::new(elements, "green", "green", active_color).push(elements, green_label);
            let group = RadioGroup::new(elements, "Color")
                .flex_direction(elements, FlexDirection::Column)
                .gap(elements, px(8), px(8))
                .push(elements, red)
                .push(elements, green);
            let window = Window::new_with_renderer(elements, "Radio buttons", RendererType::VelloCPU)
                .width(elements, px(240))
                .height(elements, px(120))
                .push(elements, group);
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
