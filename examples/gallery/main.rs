use std::cell::RefCell;
use std::rc::Rc;

use craft_retained::elements::{Calendar, Checkbox, CheckboxGroup, Container, Dropdown, Element, Image, Radio, RadioGroup, Slider, SliderDirection, Text, TextInput, TinyVg, Window};
use craft_retained::style::{AlignItems, BoxShadow, Display, FlexDirection, FlexWrap, FontStyle, FontWeight, JustifyContent, Overflow, TextAlign, Underline};
use craft_retained::{Color, CraftOptions, ResourceId, craft_main, pct, px, rgb, rgba};

use util::setup_logging;

pub fn title(str: &str) -> Text {
    Text::new(str)
        .font_weight(FontWeight::BOLD)
        .font_size(20.0)
        .margin(px(0.0), px(0.0), px(5.0), px(0.0))
}

pub fn text_input() -> Container {
    let container = Container::new();

    let text_input = TextInput::new("An element for text input")
        .width(px(200.0))
        .height(px(200.0));

    container
        .display(Display::Block)
        .push(title("Text Input"))
        .push(text_input)
}

pub fn dropdown() -> Container {
    let container = Container::new();

    let dropdown = Dropdown::new()
        .width(px(100.0))
        .push(Text::new("Cat"))
        .push(Text::new("Dog"))
        .selected_item(0);

    container
        .min_width(px(200.0))
        .display(Display::Block)
        .push(title("Dropdown"))
        .push(dropdown)
}

pub fn text() -> Container {
    let container = Container::new();

    let normal_text = Text::new("Normal Text with a Color").color(Color::from_rgb8(0, 0, 255));
    let bold_text = Text::new("Bold Text").font_weight(FontWeight::BOLD);
    let italic_text = Text::new("Italic Text").font_style(FontStyle::Italic);
    let bold_and_italic_text = Text::new("Bold & Italic Text")
        .font_weight(FontWeight::BOLD)
        .font_style(FontStyle::Italic);

    let underlined_text = Text::new("Underlined Text").underline(Some(Underline {
        thickness: Some(2.0),
        color: Color::from_rgb8(0, 255, 0),
        offset: None,
    }));

    let left_aligned_text = Text::new("Left").text_align(TextAlign::Left);
    let centered_text = Text::new("Center").text_align(TextAlign::Center);
    let right_aligned_text = Text::new("Right").text_align(TextAlign::Right);

    container
        .display(Display::Block)
        .push(title("Text"))
        .push(normal_text)
        .push(bold_text)
        .push(italic_text)
        .push(bold_and_italic_text)
        .push(underlined_text)
        .push(left_aligned_text)
        .push(centered_text)
        .push(right_aligned_text)
}

pub fn tinyvg() -> Container {
    let container = Container::new();

    let tinyvg = TinyVg::new(ResourceId::StaticBytes(include_bytes!("tiger.tvg")))
        .width(px(250.0))
        .height(px(250.0));

    container.display(Display::Block).push(title("TinyVG")).push(tinyvg)
}

pub fn images() -> Container {
    let container = Container::new();

    let image = Image::new(ResourceId::Url("https://picsum.photos/300/200".to_string()))
        .width(px(300.0))
        .height(px(200.0));

    container.display(Display::Block).push(title("Image")).push(image)
}

pub fn box_shadows() -> Container {
    let container = Container::new();
    let border_color = rgb(0, 0, 0);

    let dropshadow_box = Container::new()
        .box_shadows(vec![
            BoxShadow::new(false, 0.0, 5.0, 5.0, 0.0, rgba(0, 0, 0, 200)),
            BoxShadow::new(false, 0.0, 25.0, 35.0, 0.0, rgba(0, 0, 0, 150)),
            BoxShadow::new(true, 0.0, 4.0, 4.0, 0.0, rgba(255, 255, 255, 120)),
        ])
        .border_width(px(0), px(0), px(0), px(0))
        .border_color(border_color, border_color, border_color, border_color)
        .border_radius((8.0, 8.0), (8.0, 8.0), (8.0, 8.0), (8.0, 8.0))
        .padding(px(15), px(30), px(15), px(30))
        .justify_content(Some(JustifyContent::Center))
        .background_color(Color::from_rgb8(255, 0, 0));

    container
        .display(Display::Block)
        .push(title("Box Shadows"))
        .push(dropshadow_box)
}

pub fn multiple_windows() -> Container {
    let container = Container::new();
    let border_radius = (1.0, 1.0);
    let border_color = Color::BLACK;
    let border_width = px(1.0);

    let open_new_window_btn = Text::new("Open a new window")
        .padding(px(5.0), px(15.0), px(5.0), px(15.0))
        .border_radius(border_radius, border_radius, border_radius, border_radius)
        .border_color(border_color, border_color, border_color, border_color)
        .border_width(border_width, border_width, border_width, border_width);

    open_new_window_btn.clone().on_pointer_button_down(Rc::new(|_e, _pb| {
        Window::new("A new window!").push(Text::new("Hi!").font_size(32.0).font_weight(FontWeight::BOLD));
    }));
    container
        .display(Display::Block)
        .push(title("Multiple Windows"))
        .push(open_new_window_btn)
}

pub fn sliders() -> Container {
    let container = Container::new();

    let slider_1 = Slider::new(20.0).value(70.0).width(px(100.0)).height(px(10.0));

    let br = (0.0, 0.0);
    let slider_2 = Slider::new(14.0)
        .value(20.0)
        .width(px(100.0))
        .height(px(10.0))
        .track_color(Color::from_rgb8(120, 150, 0))
        .border_radius(br, br, br, br)
        .thumb_border_radius(br, br, br, br);

    let slider_3 = Slider::new(20.0)
        .value(70.0)
        .width(px(10.0))
        .height(px(100.0))
        .direction(SliderDirection::Vertical);

    container
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .row_gap(px(15.0))
        .push(title("Sliders"))
        .push(slider_1)
        .push(slider_2)
        .push(slider_3)
}

pub fn scrollable() -> Container {
    let container = Container::new();

    let scrollable_container = Container::new()
        .display(Display::Block)
        .overflow_y(Overflow::Scroll) // Enable vertical scrolling.
        .width(px(200.0))
        .max_height(px(150.0))
        .padding(px(5.0), px(15.0), px(5.0), px(15.0))
        .border_radius_all((1.0, 1.0))
        .border_color_all(Color::BLACK)
        .border_width_all(px(1.0))
        .push(Text::new("The Start"))
        .push(Text::new("The Middle").margin(px(50.0), px(0.0), px(250.0), px(0.0)))
        .push(Text::new("The End").padding(px(0.0), px(0.0), px(10.0), px(0.0)));

    container
        .display(Display::Block)
        .push(title("Scrollable"))
        .push(scrollable_container.clone())
        .push(
            Text::new("Scroll to the top")
                .width(px(120.0))
                .background_color(Color::from_rgb8(35, 127, 183))
                .color(Color::WHITE)
                .font_size(14.0)
                .padding(px(3.0), px(5.0), px(3.0), px(5.0))
                .on_pointer_button_down(Rc::new(move |_e, _pb| {
                    scrollable_container.clone().scroll_to_top();
                })),
        )
}

pub fn radio_buttons() -> Container {
    let active_color = Rc::new(RefCell::new("red".to_string()));

    let green = Image::new(ResourceId::Url(
        "https://www.iconsdb.com/icons/preview/green/square-xxl.png".to_string(),
    ))
    .border_width_all(px(1))
    .border_color_all(rgba(0, 0, 0, 0));
    Container::new()
        .width(pct(100.0))
        .height(pct(100.0))
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .align_items(Some(AlignItems::Center))
        .justify_content(Some(JustifyContent::Center))
        .push(
            RadioGroup::new("Pick a color")
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .justify_content(Some(JustifyContent::Center))
                .push(Radio::new("red", "red", active_color.clone()).push(Text::new("red")))
                .push(
                    Radio::new("green", "green", active_color.clone())
                        .push(green.clone())
                        .hide_radio(),
                )
                .push(Radio::new("blue", "blue", active_color.clone()).push(Text::new("blue")))
                .on_radio_value_changed(Rc::new(move |_event, new_value| {
                    if new_value.borrow().as_str() == "green" {
                        green.clone().border_color_all(rgb(0, 100, 255));
                    } else {
                        green.clone().border_color_all(rgba(0, 0, 0, 0));
                    }
                })),
        )
}

pub fn checkbox() -> Container {
    Container::new().push(
        CheckboxGroup::new("Select your favorite foods")
            .on_checkbox_toggled(Rc::new(move |_event, checkbox_toggled| {
                println!(
                    "checkbox toggled: {} - {}",
                    checkbox_toggled.label, checkbox_toggled.status
                );
            }))
            .flex_direction(FlexDirection::Column)
            .gap(px(15.0), px(15.0))
            .push(Checkbox::new("coffee", true).push(Text::new("Coffee").selectable(false)))
            .push(Checkbox::new("tea", false).push(Text::new("Tea").selectable(false)))
            .push(Checkbox::new("红烧肉", false).push(Text::new("红烧肉").selectable(false)))
            .push(Checkbox::new("カツカレー", false).push(Text::new("カツカレー").selectable(false))),
    )
}

pub fn main() {
    setup_logging();

    let window = Window::new("Gallery")
        .display(Display::Flex)
        .justify_content(Some(JustifyContent::Center))
        .align_items(Some(AlignItems::Center))
        .overflow(Overflow::Clip, Overflow::Scroll)
        .width(pct(100))
        .height(pct(100));

    let wrapper = Container::new()
        .display(Display::Flex)
        .wrap(FlexWrap::Wrap)
        .padding_all(px(10.0))
        .gap(px(40.0), px(50.0))
        .width(pct(100))
        .height(pct(100))
        .max_width(px(1200.0))
        .push(Calendar::new().start_year(1950))
        .push(text_input())
        .push(dropdown())
        .push(text())
        .push(tinyvg())
        .push(images())
        .push(box_shadows())
        .push(multiple_windows())
        .push(sliders())
        .push(scrollable())
        .push(radio_buttons())
        .push(checkbox());

    window.push(wrapper);

    craft_main(CraftOptions::basic("Gallery"));
}
