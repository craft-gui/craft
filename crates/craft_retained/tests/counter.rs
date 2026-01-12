#[macro_use]
extern crate libtest_mimic_collect;

use std::cell::RefCell;
use std::rc::Rc;

use craft_retained::elements::{Container, Element, Text, Window};
use craft_retained::events::ui_events::pointer::PointerButton;
use craft_retained::style::{AlignItems, FlexDirection, JustifyContent};
use craft_retained::{Color, CraftCallback, CraftOptions, pct, px, rgb};
use image::RgbImage;
use libtest_mimic_collect::TestCollection;
use libtest_mimic_collect::libtest_mimic::Arguments;

fn create_button(label: &str, base_color: Color, delta: i64, state: Rc<RefCell<i64>>, count_text: Text) -> Container {
    let border_color = rgb(0, 0, 0);
    Container::new()
        .border_width(px(1), px(2), px(3), px(4))
        .border_color(border_color, border_color, border_color, border_color)
        .border_radius((10.0, 10.0), (10.0, 10.0), (10.0, 10.0), (10.0, 10.0))
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

#[cfg(test)]
mod test_utils;

#[test]
fn counter() {
    let count = Rc::new(RefCell::new(0));
    let count_text = Text::new(&format!("Count: {}", count.borrow()));

    let add_button = create_button("+", rgb(76, 175, 80), 1, count.clone(), count_text.clone());

    let window = Window::new()
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
                    rgb(244, 67, 54),
                    -1,
                    count.clone(),
                    count_text.clone(),
                ))
                .push(add_button.clone())
        });

    let result_image: Rc<RefCell<Option<RgbImage>>> = Rc::new(RefCell::new(None));
    let result_image_clone = result_image.clone();
    let cb = CraftCallback(Box::new(move || {
        let add_button = add_button.clone();
        let window = window.clone();
        let result_image = result_image_clone.clone();
        async move {
            for _ in 0..3 {
                craft_retained::craft_runtime::time::sleep(craft_runtime::time::Duration::from_millis(1000)).await;
                add_button.click().await;
            }

            craft_retained::craft_runtime::time::sleep(craft_runtime::time::Duration::from_millis(30000)).await;

            let screenshot = window.clone().screenshot();
            let img_buffer = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(
                screenshot.width as u32,
                screenshot.height as u32,
                screenshot.pixels,
            )
            .unwrap();
            let dynamic_img = image::DynamicImage::ImageRgba8(img_buffer);
            *result_image.borrow_mut() = Some(dynamic_img.to_rgb8());
            window.close();
        }
    }));
    craft_retained::craft_test(CraftOptions::test("counter_test", cb));
    test_utils::check_snapshot(result_image.take().unwrap(), "counter.png");
}

pub fn main() {
    let mut args = Arguments::from_args();
    args.test_threads = Some(1);
    TestCollection::run_with_args(args);
}
