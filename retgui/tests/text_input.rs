use retgui::drivers::headless::run;
use retgui::elements::{Container, Element, TextInput, Window};
use retgui::geometry::{Point, Size};
use retgui::style::{AlignItems, JustifyContent, Overflow};
use retgui::{RendererType, pct, px};
use std::cell::Cell;
use std::rc::Rc;
use winit::event::{ElementState, Ime, KeyEvent};
use winit::keyboard::{Key, KeyCode, KeyLocation, NativeKey, PhysicalKey};

#[cfg(test)]
mod test_utils;

#[test]
fn type_hello() {
    run("text_input_hello_test", |test| {
        let text_input = TextInput::new("").font_size(32.0).width(px(300));
        let window = Window::new_with_renderer("Text input", RendererType::VelloCPU)
            .justify_content(JustifyContent::Center)
            .align_items(AlignItems::Center)
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

fn key_event(text: &str) -> KeyEvent {
    let key = Key::Character(text.into());
    KeyEvent {
        physical_key: PhysicalKey::Code(KeyCode::KeyX),
        logical_key: key.clone(),
        text: Some(text.into()),
        location: KeyLocation::Standard,
        state: ElementState::Pressed,
        repeat: false,
        text_with_all_modifiers: Some(text.into()),
        key_without_modifiers: key,
    }
}

fn ime_navigation_key_event(code: KeyCode) -> KeyEvent {
    KeyEvent {
        physical_key: PhysicalKey::Code(code),
        logical_key: Key::Unidentified(NativeKey::Unidentified),
        text: None,
        location: KeyLocation::Standard,
        state: ElementState::Pressed,
        repeat: false,
        text_with_all_modifiers: None,
        key_without_modifiers: Key::Unidentified(NativeKey::Unidentified),
    }
}

#[test]
fn set_cursor_after_ll() {
    run("text_input_set_cursor_after_ll_test", |test| {
        let text_input = TextInput::new("Hello").font_size(32.0).width(px(300));
        let window = Window::new_with_renderer("Text input cursor", RendererType::VelloCPU)
            .justify_content(JustifyContent::Center)
            .align_items(AlignItems::Center)
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
    run("text_input_select_ll_test", |test| {
        let text_input = TextInput::new("Hello").font_size(32.0).width(px(300));
        let window = Window::new_with_renderer("Text input selection", RendererType::VelloCPU)
            .justify_content(JustifyContent::Center)
            .align_items(AlignItems::Center)
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
