use std::cell::RefCell;
use std::rc::Rc;

use retgui::drivers::headless::run;
use retgui::elements::{Element, Radio, RadioGroup, Text, Window};
use retgui::geometry::Size;
use retgui::style::FlexDirection;
use retgui::{RendererType, px};

#[cfg(test)]
mod test_utils;

#[test]
fn switches_from_red_to_green() {
    run("radio_switches_from_red_to_green", |test| {
        let active_color = Rc::new(RefCell::new("red".to_string()));
        let red = Radio::new("red", "red", active_color.clone()).push(Text::new("Red"));
        let green = Radio::new("green", "green", active_color.clone()).push(Text::new("Green"));
        let group = RadioGroup::new("Color")
            .flex_direction(FlexDirection::Column)
            .gap(px(8), px(8))
            .push(red)
            .push(green.clone());
        let window = Window::new_with_renderer("Radio buttons", RendererType::VelloCPU)
            .width(px(240))
            .height(px(120))
            .push(group);

        test.open(&window, Size::new(240.0, 120.0));
        assert_eq!(active_color.borrow().as_str(), "red");

        test.click(&green);

        assert_eq!(active_color.borrow().as_str(), "green");
        test_utils::check_snapshot(
            test_utils::screenshot_rgb(test, &window),
            "switches_from_red_to_green.png",
        );
    });
}
