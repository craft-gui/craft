use std::rc::Rc;

use craft_retained::elements::{Container, Dropdown, Element, Image, Slider, SliderDirection, Text, TextInput, TinyVg, Window};
use craft_retained::style::{AlignItems, BoxShadow, Display, FlexDirection, FlexWrap, FontStyle, FontWeight, JustifyContent, Overflow, Underline};
use craft_retained::{craft_main, pct, px, rgb, rgba, Color, CraftOptions, ResourceId};
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
        .selected_item(0)
        ;

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

    let underlined_text = Text::new("Underlined Text")
        .underline(Some(Underline {
            thickness: Some(2.0),
            color: Color::from_rgb8(0, 255, 0),
            offset: None,
        }));

    container
        .display(Display::Block)
        .push(title("Text"))
        .push(normal_text)
        .push(bold_text)
        .push(italic_text)
        .push(bold_and_italic_text)
        .push(underlined_text)
}

pub fn tinyvg() -> Container {
    let container = Container::new();


    let tinyvg = TinyVg::new(ResourceId::Bytes(include_bytes!("tiger.tvg")))
        .width(px(250.0))
        .height(px(250.0))
        ;

    container
        .display(Display::Block)
        .push(title("TinyVG"))
        .push(tinyvg)
}

pub fn images() -> Container {
    let container = Container::new();


    let image = Image::new(ResourceId::Url("https://picsum.photos/300/200".to_string()))
        .width(px(300.0))
        .height(px(200.0))
        ;

    container
        .display(Display::Block)
        .push(title("Image"))
        .push(image)
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
        .border_width(border_width, border_width, border_width, border_width)
        ;

    open_new_window_btn.clone().on_pointer_button_down(Rc::new(|_e, _pb| {
        Window::new("A new window!")
            .push(
                Text::new("Hi!")
                    .font_size(32.0)
                    .font_weight(FontWeight::BOLD)
            );
    }));
    container
        .display(Display::Block)
        .push(title("Multiple Windows"))
        .push(open_new_window_btn)
}

pub fn sliders() -> Container {
    let container = Container::new();


    let slider_1 = Slider::new(20.0)
        .value(70.0)
        .width(px(100.0))
        .height(px(10.0))
        ;

    let br = (0.0, 0.0);
    let slider_2 = Slider::new(14.0)
        .value(20.0)
        .width(px(100.0))
        .height(px(10.0))
        .track_color(Color::from_rgb8(120, 150, 0))
        .border_radius(br, br, br, br)
        .thumb_border_radius(br, br, br, br)
        ;

    let slider_3 = Slider::new(20.0)
        .value(70.0)
        .width(px(10.0))
        .height(px(100.0))
        .direction(SliderDirection::Vertical)
        ;

    container
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .gap(px(15.0), px(15.0))
        .push(title("Sliders"))
        .push(slider_1)
        .push(slider_2)
        .push(slider_3)
}

pub fn scrollable() -> Container {
    let container = Container::new();

    let border_radius = (1.0, 1.0);
    let border_color = Color::BLACK;
    let border_width = px(1.0);

    let scrollable_container = Container::new().display(Display::Block)
        .overflow(Overflow::Clip, Overflow::Scroll) // Enable vertical scrolling.
        .width(px(200.0))
        .max_height(px(150.0))
        .padding(px(5.0), px(15.0), px(5.0), px(15.0))
        .border_radius(border_radius, border_radius, border_radius, border_radius)
        .border_color(border_color, border_color, border_color, border_color)
        .border_width(border_width, border_width, border_width, border_width)

        .push(Text::new("The Start"))
        .push(Text::new("The Middle")
            .margin(px(50.0), px(0.0), px(250.0), px(0.0)))
        .push(Text::new("The End")
            .padding(px(0.0), px(0.0), px(10.0), px(0.0))
        );

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
                }))
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
        .padding(px(10.0), px(10.0), px(10.0), px(10.0))
        .gap(px(40.0), px(50.0))
        .width(pct(100))
        .height(pct(100))
        .max_width(px(1200.0))
        .push(text_input())
        .push(dropdown())
        .push(text())
        .push(tinyvg())
        .push(images())
        .push(box_shadows())
        .push(multiple_windows())
        .push(sliders())
        .push(scrollable())
    ;

    window.push(wrapper);

    craft_main(CraftOptions::basic("Gallery"));
}
