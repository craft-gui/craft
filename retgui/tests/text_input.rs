use retgui::drivers::headless::run;
use retgui::elements::{Element, TextInput, Window};
use retgui::geometry::{Point, Size};
use retgui::style::{AlignItems, JustifyContent};
use retgui::{RendererType, pct, px};

#[cfg(test)]
mod test_utils;

#[test]
fn type_hello() {
    run(
        "text_input_hello_test",
        |elements| {
            let text_input = TextInput::new(elements, "")
                .edit(elements)
                .font_size(32.0)
                .width(px(300))
                .finish();
            let window = Window::new_with_renderer(elements, "Text input", RendererType::VelloCPU)
                .edit(elements)
                .justify_content(JustifyContent::Center)
                .align_items(AlignItems::Center)
                .width(pct(100))
                .height(pct(100))
                .push(text_input)
                .finish();
            (text_input, window)
        },
        |test, (text_input, window)| {
            test.open(&window, Size::new(800.0, 600.0));
            test.click(&text_input);
            test.type_text(&window, "Hello");

            assert_eq!(text_input.text(test.elements()), "Hello");
            test_utils::check_snapshot(test_utils::screenshot_rgb(test, &window), "text_input_hello.png");
        },
    );
}

#[test]
fn set_cursor_after_ll() {
    run(
        "text_input_set_cursor_after_ll_test",
        |elements| {
            let text_input = TextInput::new(elements, "Hello")
                .edit(elements)
                .font_size(32.0)
                .width(px(300))
                .finish();
            let window = Window::new_with_renderer(elements, "Text input cursor", RendererType::VelloCPU)
                .edit(elements)
                .justify_content(JustifyContent::Center)
                .align_items(AlignItems::Center)
                .width(pct(100))
                .height(pct(100))
                .push(text_input)
                .finish();
            (text_input, window)
        },
        |test, (text_input, window)| {
            test.open(&window, Size::new(800.0, 600.0));
            let content_box = text_input
                .computed_box_transformed(test.elements_mut())
                .content_rectangle();
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
        },
    );
}

#[test]
fn select_ll() {
    run(
        "text_input_select_ll_test",
        |elements| {
            let text_input = TextInput::new(elements, "Hello")
                .edit(elements)
                .font_size(32.0)
                .width(px(300))
                .finish();
            let window = Window::new_with_renderer(elements, "Text input selection", RendererType::VelloCPU)
                .edit(elements)
                .justify_content(JustifyContent::Center)
                .align_items(AlignItems::Center)
                .width(pct(100))
                .height(pct(100))
                .push(text_input)
                .finish();
            (text_input, window)
        },
        |test, (text_input, window)| {
            test.open(&window, Size::new(800.0, 600.0));
            let content_box = text_input
                .computed_box_transformed(test.elements_mut())
                .content_rectangle();
            let text_y = (content_box.y + content_box.height / 2.0) as f64;
            test.pointer_move(&window, Point::new((content_box.x + 43.0) as f64, text_y));
            test.pointer_down(&window);
            test.pointer_move(&window, Point::new((content_box.x + 58.0) as f64, text_y));
            test.pointer_up(&window);

            test_utils::check_snapshot(test_utils::screenshot_rgb(test, &window), "text_input_select_ll.png");
        },
    );
}
