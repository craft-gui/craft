use retgui::elements::{Element, TextInput, Window};
use retgui::geometry::{Point, Size};
use retgui::style::{AlignItems, JustifyContent};
use retgui::{RendererType, headless, pct, px};

#[cfg(test)]
mod test_utils;

#[test]
fn type_hello() {
    headless::run("text_input_hello_test", |test| {
        let text_input = TextInput::new("").font_size(32.0).width(px(300));
        let window = Window::new_with_renderer("Text input", RendererType::VelloCPU)
            .justify_content(Some(JustifyContent::Center))
            .align_items(Some(AlignItems::Center))
            .width(pct(100))
            .height(pct(100))
            .push(text_input.clone());

        test.open(&window, Size::new(800.0, 600.0));
        test.click(&text_input);
        test.type_text(&window, "Hello");

        assert_eq!(text_input.get_text(), "Hello");
        test_utils::check_snapshot(test_utils::screenshot_rgb(test, &window), "text_input_hello.png");
    });
}

#[test]
fn set_cursor_after_ll() {
    headless::run("text_input_set_cursor_after_ll_test", |test| {
        let text_input = TextInput::new("Hello").font_size(32.0).width(px(300));
        let window = Window::new_with_renderer("Text input cursor", RendererType::VelloCPU)
            .justify_content(Some(JustifyContent::Center))
            .align_items(Some(AlignItems::Center))
            .width(pct(100))
            .height(pct(100))
            .push(text_input.clone());

        test.open(&window, Size::new(800.0, 600.0));
        let content_box = text_input.get_computed_box_transformed().content_rectangle();
        test.pointer_move(
            &window,
            Point::new(
                (content_box.x + 58.0) as f64,
                (content_box.y + content_box.height / 2.0) as f64,
            ),
        );
        test.pointer_down(&window);
        test.pointer_up(&window);

        test_utils::check_snapshot(
            test_utils::screenshot_rgb(test, &window),
            "text_input_cursor_after_ll.png",
        );
    });
}

#[test]
fn select_ll() {
    headless::run("text_input_select_ll_test", |test| {
        let text_input = TextInput::new("Hello").font_size(32.0).width(px(300));
        let window = Window::new_with_renderer("Text input selection", RendererType::VelloCPU)
            .justify_content(Some(JustifyContent::Center))
            .align_items(Some(AlignItems::Center))
            .width(pct(100))
            .height(pct(100))
            .push(text_input.clone());

        test.open(&window, Size::new(800.0, 600.0));
        let content_box = text_input.get_computed_box_transformed().content_rectangle();
        let text_y = (content_box.y + content_box.height / 2.0) as f64;
        test.pointer_move(&window, Point::new((content_box.x + 43.0) as f64, text_y));
        test.pointer_down(&window);
        test.pointer_move(&window, Point::new((content_box.x + 58.0) as f64, text_y));
        test.pointer_up(&window);

        test_utils::check_snapshot(test_utils::screenshot_rgb(test, &window), "text_input_select_ll.png");
    });
}
