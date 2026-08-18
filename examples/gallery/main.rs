use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[cfg(feature = "audio")]
use retgui::elements::Audio;
use retgui::elements::{Button, Calendar, Checkbox, CheckboxGroup, Container, Dropdown, Element, Image, Radio, RadioGroup, Slider, SliderDirection, Text, TextInput, TinyVg, Window};
use retgui::geometry::Point;
use retgui::style::{AlignItems, BoxShadow, Display, FlexDirection, FlexWrap, FontStyle, FontWeight, JustifyContent, Overflow, Position, TextAlign};
use retgui::{Color, ColorStop, Gradient, ResourceId, RetGuiOptions, RetGuiRuntime, auto, pct, px, retgui_main, rgb, rgba};

use serde::Deserialize;

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

    let underlined_text = Text::new("Underlined Text").underline(Some(2.0), Color::from_rgb8(0, 255, 0), None);

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

#[derive(Deserialize)]
struct WeatherResponse {
    current: CurrentWeather,
}

#[derive(Deserialize)]
struct CurrentWeather {
    time: String,
    temperature_2m: f64,
    apparent_temperature: f64,
    relative_humidity_2m: u8,
    weather_code: u8,
    wind_speed_10m: f64,
}

async fn fetch_amsterdam_weather() -> Result<CurrentWeather, String> {
    let response = reqwest::get(
        "https://api.open-meteo.com/v1/forecast?latitude=52.374&longitude=4.8897&current=temperature_2m,apparent_temperature,relative_humidity_2m,weather_code,wind_speed_10m&timezone=Europe%2FAmsterdam",
    )
    .await
    .map_err(|error| error.to_string())?
    .error_for_status()
    .map_err(|error| error.to_string())?;

    let weather = response
        .json::<WeatherResponse>()
        .await
        .map_err(|error| error.to_string())?;

    Ok(weather.current)
}

fn weather_description(code: u8) -> &'static str {
    match code {
        0 => "Clear sky",
        1..=3 => "Partly cloudy",
        45 | 48 => "Fog",
        51..=57 => "Drizzle",
        61..=67 => "Rain",
        71..=77 => "Snow",
        80..=82 => "Rain showers",
        85 | 86 => "Snow showers",
        95..=99 => "Thunderstorm",
        _ => "Unknown conditions",
    }
}

pub fn async_weather() -> Container {
    let status = Text::new("Click the button for the current conditions.")
        .width(px(280.0))
        .font_size(14.0);

    let status_for_handler = status.clone();
    let button = Button::new()
        .padding(px(5.0), px(15.0), px(5.0), px(15.0))
        .border_radius_all((4.0, 4.0))
        .background_color(Color::from_rgb8(35, 127, 183))
        .push(Text::new("Refresh Weather").color(Color::WHITE).selectable(false))
        .on_click(move |event| {
            status_for_handler.clone().text("Loading...");

            let status = status_for_handler.clone();
            RetGuiRuntime::spawn_local(async move {
                match fetch_amsterdam_weather().await {
                    Ok(weather) => {
                        status.text(&format!(
                            "{}\n{:.1} °C (feels like {:.1} °C)\nHumidity: {}%\nWind: {:.1} km/h\nUpdated: {}",
                            weather_description(weather.weather_code),
                            weather.temperature_2m,
                            weather.apparent_temperature,
                            weather.relative_humidity_2m,
                            weather.wind_speed_10m,
                            weather.time,
                        ));
                    }
                    Err(error) => {
                        status.text(&format!("Request failed: {error}"));
                    }
                }
            });

            event.prevent_propagate();
        });

    Container::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .row_gap(px(8.0))
        .push(title("Amsterdam Weather"))
        .push(button)
        .push(status)
        .push(Text::new("Weather data by Open-Meteo").font_size(12.0))
}

pub fn gradient() -> Container {
    let container = Container::new();

    let linear = Gradient::new_linear(Point::new(0.0, 0.0), Point::new(1.0, 0.0)).color_stops(&[
        ColorStop::new(0.0, Color::from_rgb8(120, 0, 200)),
        ColorStop::new(0.45, Color::from_rgb8(35, 127, 183)),
        ColorStop::new(1.0, Color::from_rgb8(255, 0, 0)),
    ]);

    let radial = Gradient::new_radial(Point::new(0.5, 0.5), 0.0, Point::new(0.5, 0.5), 0.75).color_stops(&[
        ColorStop::new(0.0, Color::from_rgb8(255, 245, 157)),
        ColorStop::new(0.55, Color::from_rgb8(255, 112, 67)),
        ColorStop::new(1.0, Color::from_rgb8(74, 20, 140)),
    ]);

    let sweep = Gradient::new_sweep(Point::new(0.5, 0.5), 0.0, std::f32::consts::TAU).color_stops(&[
        ColorStop::new(0.0, Color::from_rgb8(244, 67, 54)),
        ColorStop::new(0.33, Color::from_rgb8(76, 175, 80)),
        ColorStop::new(0.66, Color::from_rgb8(33, 150, 243)),
        ColorStop::new(1.0, Color::from_rgb8(244, 67, 54)),
    ]);

    let linear_box = Container::new()
        .width(px(140.0))
        .height(px(90.0))
        .border_radius_all((8.0, 8.0))
        .background_gradient(linear.clone());

    let radial_box = Container::new()
        .width(px(140.0))
        .height(px(90.0))
        .border_radius_all((8.0, 8.0))
        .background_gradient(radial);

    let sweep_box = Container::new()
        .width(px(140.0))
        .height(px(90.0))
        .border_radius_all((8.0, 8.0))
        .background_gradient(sweep);

    let gradient_text = Text::new("Gradient Text")
        .font_weight(FontWeight::BOLD)
        .text_gradient(linear.clone());

    let underline_text = Text::new("Gradient Underline").underline_gradient(Some(3.0), linear, None);

    container
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .row_gap(px(10.0))
        .push(title("Gradients"))
        .push(gradient_text)
        .push(underline_text)
        .push(
            Container::new()
                .display(Display::Flex)
                .gap(px(10.0), px(10.0))
                .push(linear_box)
                .push(radial_box)
                .push(sweep_box),
        )
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
        .justify_content(JustifyContent::Center)
        .background_color(Color::from_rgb8(255, 0, 0));

    container
        .display(Display::Block)
        .push(title("Box Shadows"))
        .push(dropshadow_box)
}

pub fn overlay() -> Container {
    let status = Text::new("Click where the cards overlap");

    let overlay_status = status.clone();
    let floating_card = Container::new()
        .overlay(true)
        .position(Position::Absolute)
        .inset(px(20.0), auto(), auto(), px(20.0))
        .width(px(150.0))
        .height(px(100.0))
        .padding_all(px(10.0))
        .background_color(Color::from_rgb8(76, 175, 80))
        .push(Text::new("Overlay").color(Color::WHITE).selectable(false))
        .on_click(move |event| {
            overlay_status.clone().text("The overlay received the click");
            event.prevent_propagate();
        });

    let normal_status = status.clone();
    let normal_card = Container::new()
        .position(Position::Absolute)
        .inset(px(65.0), auto(), auto(), px(90.0))
        .width(px(120.0))
        .height(px(70.0))
        .padding_all(px(10.0))
        .background_color(Color::from_rgb8(33, 150, 243))
        .push(Text::new("Normal sibling").color(Color::WHITE).selectable(false))
        .on_click(move |event| {
            normal_status.clone().text("The normal sibling received the click");
            event.prevent_propagate();
        });

    Container::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .row_gap(px(8.0))
        .width(px(280.0))
        .min_width(auto())
        .push(title("Overlay"))
        .margin_horizontal(auto())
        .push(
            Container::new()
                .position(Position::Relative)
                .width(px(230.0))
                .height(px(155.0))
                .background_color(Color::from_rgb8(238, 238, 238))
                .push(floating_card)
                .push(normal_card),
        )
        .push(status)
}

pub fn multiple_windows() -> Container {
    let container = Container::new();
    let border_radius = (1.0, 1.0);
    let border_color = Color::BLACK;
    let border_width = px(1.0);

    let open_new_window_btn = Button::new()
        .push(Text::new("Open a new window"))
        .padding(px(5.0), px(15.0), px(5.0), px(15.0))
        .border_radius(border_radius, border_radius, border_radius, border_radius)
        .border_color(border_color, border_color, border_color, border_color)
        .border_width(border_width, border_width, border_width, border_width);

    open_new_window_btn.clone().on_click(|_e| {
        Window::new("A new window!").push(Text::new("Hi!").font_size(32.0).font_weight(FontWeight::BOLD));
    });
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
            Button::new()
                .width(px(120.0))
                .background_color(Color::from_rgb8(35, 127, 183))
                .on_click(move |_e| {
                    scrollable_container.clone().scroll_to_top();
                })
                .push(
                    Text::new("Scroll to the top")
                        .color(Color::WHITE)
                        .font_size(14.0)
                        .padding(px(3.0), px(5.0), px(3.0), px(5.0)),
                ),
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
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .push(title("Radio Button"))
        .push(
            RadioGroup::new("Pick a color")
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .justify_content(JustifyContent::Center)
                .push(Radio::new("red", "red", active_color.clone()).push(Text::new("red")))
                .push(
                    Radio::new("green", "green", active_color.clone())
                        .push(green.clone())
                        .hide_radio(),
                )
                .push(Radio::new("blue", "blue", active_color.clone()).push(Text::new("blue")))
                .on_radio_value_changed(move |_event, new_value| {
                    if new_value.borrow().as_str() == "green" {
                        green.clone().border_color_all(rgb(0, 100, 255));
                    } else {
                        green.clone().border_color_all(rgba(0, 0, 0, 0));
                    }
                }),
        )
}

pub fn checkbox() -> Container {
    Container::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .push(title("Checkbox"))
        .push(
            CheckboxGroup::new("Select your favorite foods")
                .on_checkbox_toggled(move |_event, checkbox_toggled| {
                    println!(
                        "checkbox toggled: {} - {}",
                        checkbox_toggled.label, checkbox_toggled.status
                    );
                })
                .flex_direction(FlexDirection::Column)
                .gap(px(15.0), px(15.0))
                .push(Checkbox::new("coffee", true).push(Text::new("Coffee").selectable(false)))
                .push(Checkbox::new("tea", false).push(Text::new("Tea").selectable(false)))
                .push(Checkbox::new("红烧肉", false).push(Text::new("红烧肉").selectable(false)))
                .push(Checkbox::new("カツカレー", false).push(Text::new("カツカレー").selectable(false))),
        )
}

#[cfg(feature = "audio")]
pub fn audio() -> Audio {
    let mut asset_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    asset_path.push("assets");
    asset_path.push("1-11. Mice on Venus.mp3");
    Audio::new(Path::new(asset_path.as_path()))
}

#[cfg(not(feature = "audio"))]
pub fn audio() -> Container {
    Container::new()
}

pub fn main() {
    setup_logging();

    let window = Window::new("Gallery")
        .display(Display::Flex)
        .justify_content(JustifyContent::Center)
        .align_items(AlignItems::Center)
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
        .push(audio())
        .push(Calendar::new().start_year(1950))
        .push(text_input())
        .push(dropdown())
        .push(text())
        .push(tinyvg())
        .push(images())
        .push(gradient())
        .push(box_shadows())
        .push(async_weather())
        .push(overlay())
        .push(sliders())
        .push(radio_buttons())
        .push(checkbox())
        .push(scrollable())
        .push(multiple_windows());

    window.push(wrapper);

    retgui_main(RetGuiOptions::basic("Gallery"));
}
