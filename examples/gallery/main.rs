#[cfg(feature = "audio")]
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

#[cfg(feature = "audio")]
use retgui::elements::Audio;
use retgui::elements::{Button, Calendar, Checkbox, CheckboxGroup, Container, Dropdown, DynElement, Element, Image, Radio, RadioGroup, Slider, SliderDirection, State, Text, TextInput, TinyVg, Window};
use retgui::events::Event;
use retgui::geometry::Point;
use retgui::style::{AlignItems, Animation, BoxShadow, Display, FlexDirection, FontFamily, FontStyle, FontWeight, JustifyContent, KeyFrame, Overflow, Position, Repeat, StyleVariant, TextAlign, TimingFunction};
use retgui::{App, Brush, Color, ColorStop, Gradient, ResourceId, ResourceType, RetGuiOptions, auto, pct, px, retgui_main, rgb, rgba};

use serde::Deserialize;

use util::setup_logging;

pub fn title(app: &mut App, value: &str) -> Text {
    Text::new(app, value)
        .edit(app)
        .font_weight(FontWeight::BOLD)
        .font_size(20.0)
        .margin(px(0.0), px(0.0), px(5.0), px(0.0))
        .finish()
}

pub fn animations(app: &mut App) -> Text {
    let gameboy_gradient = |start, end| {
        Gradient::new_linear(Point::new(start, 0.0), Point::new(end, 0.0)).color_stops(&[
            ColorStop::new(0.0, Color::from_rgb8(50, 50, 252)),
            ColorStop::new(0.2, Color::from_rgb8(133, 227, 103)),
            ColorStop::new(0.4, Color::from_rgb8(255, 82, 232)),
            ColorStop::new(0.6, Color::from_rgb8(255, 1, 81)),
            ColorStop::new(0.8, Color::from_rgb8(249, 229, 46)),
            ColorStop::new(1.0, Color::from_rgb8(240, 240, 240)),
        ])
    };
    let start = gameboy_gradient(-0.5, 1.0);
    let end = gameboy_gradient(0.0, 1.5);
    let animation = Animation::new(Duration::from_secs(3), Repeat::Forever, TimingFunction::EaseInOut)
        .push(KeyFrame::new(0.0).push(StyleVariant::TextBrush(Brush::Gradient(start.clone()))))
        .push(KeyFrame::new(50.0).push(StyleVariant::TextBrush(Brush::Gradient(end))))
        .push(KeyFrame::new(100.0).push(StyleVariant::TextBrush(Brush::Gradient(start))));
    Text::new(app, "Animations")
        .edit(app)
        .font_size(64.0)
        .font_weight(FontWeight::BOLD)
        .animations(vec![animation])
        .finish()
}

pub fn text_input(app: &mut App) -> Container {
    let input = TextInput::new(app, "An element for text input")
        .edit(app)
        .width(px(200.0))
        .height(px(200.0))
        .finish();
    let heading = title(app, "Text Input");
    Container::new(app)
        .edit(app)
        .display(Display::Block)
        .push(heading)
        .push(input)
        .finish()
}

pub fn dropdown(app: &mut App) -> Container {
    let cat = Text::new(app, "Cat");
    let dog = Text::new(app, "Dog");
    let dropdown = Dropdown::new(app)
        .edit(app)
        .width(px(100.0))
        .push(cat)
        .push(dog)
        .selected_item(0)
        .finish();
    let heading = title(app, "Dropdown");
    Container::new(app)
        .edit(app)
        .min_width(px(200.0))
        .display(Display::Block)
        .push(heading)
        .push(dropdown)
        .finish()
}

pub fn text(app: &mut App) -> Container {
    let normal = Text::new(app, "Normal Text with a Color")
        .edit(app)
        .color(Color::from_rgb8(0, 0, 255))
        .finish();
    let bold = Text::new(app, "Bold Text")
        .edit(app)
        .font_weight(FontWeight::BOLD)
        .finish();
    let italic = Text::new(app, "Italic Text")
        .edit(app)
        .font_style(FontStyle::Italic)
        .finish();
    let bold_italic = Text::new(app, "Bold & Italic Text")
        .edit(app)
        .font_weight(FontWeight::BOLD)
        .font_style(FontStyle::Italic)
        .finish();
    let underlined = Text::new(app, "Underlined Text")
        .edit(app)
        .underline(Some(2.0), Color::from_rgb8(0, 255, 0), None)
        .finish();
    let left = Text::new(app, "Left").edit(app).text_align(TextAlign::Left).finish();
    let center = Text::new(app, "Center")
        .edit(app)
        .text_align(TextAlign::Center)
        .finish();
    let right = Text::new(app, "Right").edit(app).text_align(TextAlign::Right).finish();
    let heading = title(app, "Text");
    Container::new(app)
        .edit(app)
        .display(Display::Block)
        .push(heading)
        .push(normal)
        .push(bold)
        .push(italic)
        .push(bold_italic)
        .push(underlined)
        .push(left)
        .push(center)
        .push(right)
        .finish()
}

pub fn variable_fonts(app: &mut App) -> Container {
    let font = include_bytes!("../../assets/fonts/Roboto-VariableFont_wdth,wght.ttf");
    app.upload_resource(ResourceId::StaticBytes(font), ResourceType::Font, font.as_slice())
        .expect("gallery font must load");
    let heading = title(app, "Variable Fonts");
    let description = Text::new(app, "Roboto: drag the slider to explore font weights from 100 to 900.");
    let preview = Text::new(app, "The quick brown fox jumps over the lazy dog.\nABCDEFGHIJKLMNOPQRSTUVWXYZ\nabcdefghijklmnopqrstuvwxyz 0123456789")
        .edit(app)
        .font_family(FontFamily::new("Roboto"))
        .font_size(36.0)
        .font_weight(FontWeight::NORMAL)
        .finish();
    let weight_label = Text::new(app, "Weight: 400");
    let weight = Slider::new(app, 20.0)
        .edit(app)
        .min(100.0)
        .max(900.0)
        .step(1.0)
        .value(400.0)
        .width(px(300.0))
        .height(px(10.0))
        .margin_vertical(px(10.0))
        .add_slider_value_changed_listener(move |event, app| {
            let weight = event.value.round() as u16;
            preview.edit(app).font_weight(FontWeight(weight)).finish();
            weight_label.edit(app).text(&format!("Weight: {weight}")).finish();
        })
        .finish();

    Container::new(app)
        .edit(app)
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .row_gap(px(12.0))
        .push(heading)
        .push(description)
        .push(weight_label)
        .push(weight)
        .push(preview)
        .finish()
}

pub fn tinyvg(app: &mut App) -> Container {
    let tiger = include_bytes!("tiger.tvg");
    app.upload_resource(ResourceId::StaticBytes(tiger), ResourceType::TinyVg, tiger.as_slice())
        .expect("gallery image must load");
    let image = TinyVg::new(app, ResourceId::StaticBytes(include_bytes!("tiger.tvg")))
        .edit(app)
        .width(px(250.0))
        .height(px(250.0))
        .finish();
    let heading = title(app, "TinyVG");
    Container::new(app)
        .edit(app)
        .display(Display::Block)
        .push(heading)
        .push(image)
        .finish()
}

pub fn images(app: &mut App) -> Container {
    let image = Image::new(app, ResourceId::Url("https://picsum.photos/300/200".to_string()))
        .edit(app)
        .width(px(300.0))
        .height(px(200.0))
        .finish();
    let heading = title(app, "Image");
    Container::new(app)
        .edit(app)
        .display(Display::Block)
        .push(heading)
        .push(image)
        .finish()
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

pub fn async_weather(app: &mut App) -> Container {
    let status = Text::new(app, "Click the button for the current conditions.")
        .edit(app)
        .width(px(280.0))
        .font_size(14.0)
        .finish();
    let label = Text::new(app, "Refresh Weather")
        .edit(app)
        .color(Color::WHITE)
        .selectable(false)
        .finish();
    let button = Button::new(app)
        .edit(app)
        .padding(px(5.0), px(15.0), px(5.0), px(15.0))
        .border_radius_all((4.0, 4.0))
        .background_color(Color::from_rgb8(35, 127, 183))
        .push(label)
        .add_click_listener(move |event, app| {
            status.edit(app).text("Loading...").finish();
            app.spawn_local(fetch_amsterdam_weather(), move |weather, app| {
                let message = match weather {
                    Ok(weather) => format!(
                        "{}\n{:.1} °C (feels like {:.1} °C)\nHumidity: {}%\nWind: {:.1} km/h\nUpdated: {}",
                        weather_description(weather.weather_code),
                        weather.temperature_2m,
                        weather.apparent_temperature,
                        weather.relative_humidity_2m,
                        weather.wind_speed_10m,
                        weather.time,
                    ),
                    Err(error) => format!("Request failed: {error}"),
                };
                status.edit(app).text(&message).finish();
            });
            event.stop_propagation();
        })
        .finish();
    let heading = title(app, "Amsterdam Weather");
    let attribution = Text::new(app, "Weather data by Open-Meteo")
        .edit(app)
        .font_size(12.0)
        .finish();
    Container::new(app)
        .edit(app)
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .row_gap(px(8.0))
        .push(heading)
        .push(button)
        .push(status)
        .push(attribution)
        .finish()
}

pub fn gradient(app: &mut App) -> Container {
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
    let linear_box = Container::new(app)
        .edit(app)
        .width(px(140.0))
        .height(px(90.0))
        .border_radius_all((8.0, 8.0))
        .background_gradient(linear.clone())
        .finish();
    let radial_box = Container::new(app)
        .edit(app)
        .width(px(140.0))
        .height(px(90.0))
        .border_radius_all((8.0, 8.0))
        .background_gradient(radial)
        .finish();
    let sweep_box = Container::new(app)
        .edit(app)
        .width(px(140.0))
        .height(px(90.0))
        .border_radius_all((8.0, 8.0))
        .background_gradient(sweep)
        .finish();
    let gradient_text = Text::new(app, "Gradient Text")
        .edit(app)
        .font_weight(FontWeight::BOLD)
        .text_gradient(linear.clone())
        .finish();
    let underline = Text::new(app, "Gradient Underline")
        .edit(app)
        .underline_gradient(Some(3.0), linear, None)
        .finish();
    let boxes = Container::new(app)
        .edit(app)
        .display(Display::Flex)
        .gap(px(10.0), px(10.0))
        .push(linear_box)
        .push(radial_box)
        .push(sweep_box)
        .finish();
    let heading = title(app, "Gradients");
    Container::new(app)
        .edit(app)
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .row_gap(px(10.0))
        .push(heading)
        .push(gradient_text)
        .push(underline)
        .push(boxes)
        .finish()
}

pub fn box_shadows(app: &mut App) -> Container {
    let border_color = rgb(0, 0, 0);
    let shadow = Container::new(app)
        .edit(app)
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
        .background_color(Color::from_rgb8(255, 0, 0))
        .finish();
    let heading = title(app, "Box Shadows");
    Container::new(app)
        .edit(app)
        .display(Display::Block)
        .push(heading)
        .push(shadow)
        .finish()
}

pub fn overlay(app: &mut App) -> Container {
    let status = Text::new(app, "Click where the cards overlap");
    let overlay_label = Text::new(app, "Overlay")
        .edit(app)
        .color(Color::WHITE)
        .selectable(false)
        .finish();
    let floating = Container::new(app)
        .edit(app)
        .overlay(true)
        .position(Position::Absolute)
        .inset(px(20.0), auto(), auto(), px(20.0))
        .width(px(150.0))
        .height(px(100.0))
        .padding_all(px(10.0))
        .background_color(Color::from_rgb8(76, 175, 80))
        .push(overlay_label)
        .add_click_listener(move |event, app| {
            status.edit(app).text("The overlay received the click").finish();
            event.stop_propagation();
        })
        .finish();
    let normal_label = Text::new(app, "Normal sibling")
        .edit(app)
        .color(Color::WHITE)
        .selectable(false)
        .finish();
    let normal = Container::new(app)
        .edit(app)
        .position(Position::Absolute)
        .inset(px(65.0), auto(), auto(), px(90.0))
        .width(px(120.0))
        .height(px(70.0))
        .padding_all(px(10.0))
        .background_color(Color::from_rgb8(33, 150, 243))
        .push(normal_label)
        .add_click_listener(move |event, app| {
            status.edit(app).text("The normal sibling received the click").finish();
            event.stop_propagation();
        })
        .finish();
    let cards = Container::new(app)
        .edit(app)
        .position(Position::Relative)
        .width(px(230.0))
        .height(px(155.0))
        .background_color(Color::from_rgb8(238, 238, 238))
        .push(floating)
        .push(normal)
        .finish();
    let heading = title(app, "Overlay");
    Container::new(app)
        .edit(app)
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .row_gap(px(8.0))
        .width(px(280.0))
        .min_width(auto())
        .push(heading)
        .margin_horizontal(auto())
        .push(cards)
        .push(status)
        .finish()
}

pub fn multiple_windows(app: &mut App) -> Container {
    let radius = (1.0, 1.0);
    let border = Color::BLACK;
    let width = px(1.0);
    let label = Text::new(app, "Open a new window");
    let button = Button::new(app)
        .edit(app)
        .push(label)
        .padding(px(5.0), px(15.0), px(5.0), px(15.0))
        .border_radius(radius, radius, radius, radius)
        .border_color(border, border, border, border)
        .border_width(width, width, width, width)
        .add_click_listener(|_event, app| {
            let greeting = Text::new(app, "Hi!")
                .edit(app)
                .font_size(32.0)
                .font_weight(FontWeight::BOLD)
                .finish();
            Window::new(app, "A new window!").edit(app).push(greeting).finish();
        })
        .finish();
    let heading = title(app, "Multiple Windows");
    Container::new(app)
        .edit(app)
        .display(Display::Block)
        .push(heading)
        .push(button)
        .finish()
}

pub fn sliders(app: &mut App) -> Container {
    let first = Slider::new(app, 20.0)
        .edit(app)
        .value(70.0)
        .width(px(100.0))
        .height(px(10.0))
        .finish();
    let br = (0.0, 0.0);
    let second = Slider::new(app, 14.0)
        .edit(app)
        .value(20.0)
        .width(px(100.0))
        .height(px(10.0))
        .track_color(Color::from_rgb8(120, 150, 0))
        .border_radius(br, br, br, br)
        .thumb_border_radius(br, br, br, br)
        .finish();
    let third = Slider::new(app, 20.0)
        .edit(app)
        .value(70.0)
        .width(px(10.0))
        .height(px(100.0))
        .direction(SliderDirection::Vertical)
        .finish();
    let heading = title(app, "Sliders");
    Container::new(app)
        .edit(app)
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .row_gap(px(15.0))
        .push(heading)
        .push(first)
        .push(second)
        .push(third)
        .finish()
}

pub fn scrollable(app: &mut App) -> Container {
    let start = Text::new(app, "The Start");
    let middle = Text::new(app, "The Middle")
        .edit(app)
        .margin(px(50.0), px(0.0), px(250.0), px(0.0))
        .finish();
    let end = Text::new(app, "The End")
        .edit(app)
        .padding(px(0.0), px(0.0), px(10.0), px(0.0))
        .finish();
    let scrollable = Container::new(app)
        .edit(app)
        .display(Display::Block)
        .overflow_y(Overflow::Scroll)
        .width(px(200.0))
        .max_height(px(150.0))
        .padding(px(5.0), px(15.0), px(5.0), px(15.0))
        .border_radius_all((1.0, 1.0))
        .border_color_all(Color::BLACK)
        .border_width_all(px(1.0))
        .push(start)
        .push(middle)
        .push(end)
        .finish();
    let label = Text::new(app, "Scroll to the top")
        .edit(app)
        .color(Color::WHITE)
        .font_size(14.0)
        .padding(px(3.0), px(5.0), px(3.0), px(5.0))
        .finish();
    let button = Button::new(app)
        .edit(app)
        .width(px(120.0))
        .background_color(Color::from_rgb8(35, 127, 183))
        .add_click_listener(move |_event, app| {
            scrollable.scroll_to_top(app);
        })
        .push(label)
        .finish();
    let heading = title(app, "Scrollable");
    Container::new(app)
        .edit(app)
        .display(Display::Block)
        .push(heading)
        .push(scrollable)
        .push(button)
        .finish()
}

pub fn radio_buttons(app: &mut App) -> Container {
    let active = app.insert_state("red".to_string());
    let green = Image::new(
        app,
        ResourceId::Url("https://www.iconsdb.com/icons/preview/green/square-xxl.png".to_string()),
    )
    .edit(app)
    .border_width_all(px(1))
    .border_color_all(rgba(0, 0, 0, 0))
    .finish();
    let red_label = Text::new(app, "red");
    let red = Radio::new(app, "red", "red", active).edit(app).push(red_label).finish();
    let green_radio = Radio::new(app, "green", "green", active)
        .edit(app)
        .push(green)
        .hide_radio()
        .finish();
    let blue_label = Text::new(app, "blue");
    let blue = Radio::new(app, "blue", "blue", active)
        .edit(app)
        .push(blue_label)
        .finish();
    let group = RadioGroup::new(app, "Pick a color")
        .edit(app)
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .justify_content(JustifyContent::Center)
        .push(red)
        .push(green_radio)
        .push(blue)
        .add_radio_value_changed_listener(move |event, app| {
            green
                .edit(app)
                .border_color_all(if event.value.as_str() == "green" {
                    rgb(0, 100, 255)
                } else {
                    rgba(0, 0, 0, 0)
                })
                .finish();
        })
        .finish();
    let heading = title(app, "Radio Button");
    Container::new(app)
        .edit(app)
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .push(heading)
        .push(group)
        .finish()
}

pub fn checkbox(app: &mut App) -> Container {
    let coffee_label = Text::new(app, "Coffee").edit(app).selectable(false).finish();
    let coffee = Checkbox::new(app, "coffee", true).edit(app).push(coffee_label).finish();
    let tea_label = Text::new(app, "Tea").edit(app).selectable(false).finish();
    let tea = Checkbox::new(app, "tea", false).edit(app).push(tea_label).finish();
    let pork_label = Text::new(app, "红烧肉").edit(app).selectable(false).finish();
    let pork = Checkbox::new(app, "红烧肉", false).edit(app).push(pork_label).finish();
    let curry_label = Text::new(app, "カツカレー").edit(app).selectable(false).finish();
    let curry = Checkbox::new(app, "カツカレー", false)
        .edit(app)
        .push(curry_label)
        .finish();
    let group = CheckboxGroup::new(app, "Select your favorite foods")
        .edit(app)
        .add_checkbox_toggled_listener(move |event, _app| {
            println!("checkbox toggled: {} - {}", event.label, event.status);
        })
        .flex_direction(FlexDirection::Column)
        .gap(px(15.0), px(15.0))
        .push(coffee)
        .push(tea)
        .push(pork)
        .push(curry)
        .finish();
    let heading = title(app, "Checkbox");
    Container::new(app)
        .edit(app)
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .push(heading)
        .push(group)
        .finish()
}

#[cfg(feature = "audio")]
pub fn audio(app: &mut App) -> Audio {
    let mut asset_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    asset_path.push("assets");
    asset_path.push("1-11. Mice on Venus.mp3");
    Audio::new(app, Path::new(asset_path.as_path()))
}

#[cfg(not(feature = "audio"))]
pub fn audio(app: &mut App) -> Container {
    Container::new(app)
}

struct GalleryExample {
    label: &'static str,
    section: Container,
}

impl GalleryExample {
    fn new(app: &mut App, label: &'static str, child: impl Element) -> Self {
        let section = Container::new(app)
            .edit(app)
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .flex_grow(1.0)
            .width(pct(100))
            .height(pct(100))
            .padding_all(px(32.0))
            .overflow(Overflow::Clip, Overflow::Scroll)
            .push(child)
            .finish();
        Self { label, section }
    }

    fn titled(app: &mut App, label: &'static str, child: impl Element) -> Self {
        let heading = title(app, label);
        let content = Container::new(app)
            .edit(app)
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .row_gap(px(12.0))
            .push(heading)
            .push(child)
            .finish();
        Self::new(app, label, content)
    }
}

#[derive(Clone, Copy)]
struct NavigationSelection {
    active: State<DynElement>,
}

impl NavigationSelection {
    fn new(app: &mut App, active: DynElement) -> Self {
        Self {
            active: app.insert_state(active),
        }
    }

    fn select(&self, app: &mut App, target: DynElement) {
        let previous = self.active.update(app, |active| std::mem::replace(active, target));
        style_navigation_button(app, previous, false);
        style_navigation_button(app, target, true);
    }
}

fn gallery_examples(app: &mut App) -> Vec<GalleryExample> {
    let animations = animations(app);
    let audio = audio(app);
    let calendar = Calendar::new(app).edit(app).start_year(1950).finish();
    let text_input = text_input(app);
    let dropdown = dropdown(app);
    let text = text(app);
    let variable_fonts = variable_fonts(app);
    let tinyvg = tinyvg(app);
    let images = images(app);
    let gradient = gradient(app);
    let shadows = box_shadows(app);
    let weather = async_weather(app);
    let overlay = overlay(app);
    let sliders = sliders(app);
    let radios = radio_buttons(app);
    let checkboxes = checkbox(app);
    let scrollable = scrollable(app);
    let windows = multiple_windows(app);

    vec![
        GalleryExample::new(app, "Animations", animations),
        GalleryExample::titled(app, "Audio", audio),
        GalleryExample::titled(app, "Calendar", calendar),
        GalleryExample::new(app, "Text Input", text_input),
        GalleryExample::new(app, "Dropdown", dropdown),
        GalleryExample::new(app, "Text", text),
        GalleryExample::new(app, "Variable Fonts", variable_fonts),
        GalleryExample::new(app, "TinyVG", tinyvg),
        GalleryExample::new(app, "Image", images),
        GalleryExample::new(app, "Gradients", gradient),
        GalleryExample::new(app, "Box Shadows", shadows),
        GalleryExample::new(app, "Async", weather),
        GalleryExample::new(app, "Overlay", overlay),
        GalleryExample::new(app, "Sliders", sliders),
        GalleryExample::new(app, "Radio Buttons", radios),
        GalleryExample::new(app, "Checkboxes", checkboxes),
        GalleryExample::new(app, "Scrollable", scrollable),
        GalleryExample::new(app, "Multiple Windows", windows),
    ]
}

fn navigation_background(selected: bool) -> Color {
    if selected {
        Color::from_rgb8(214, 232, 250)
    } else {
        Color::from_rgb8(247, 248, 250)
    }
}

fn style_navigation_button(app: &mut App, button: impl Element, selected: bool) {
    button
        .edit(app)
        .background_color(navigation_background(selected))
        .outline_color_all(retgui::palette::css::DODGER_BLUE)
        .outline_width_all(px(if selected { 2.0 } else { 0.0 }))
        .finish();
}

fn navigation_button(app: &mut App, label: &str, selected: bool) -> Button {
    let label = Text::new(app, label)
        .edit(app)
        .font_size(15.0)
        .selectable(false)
        .finish();
    Button::new(app)
        .edit(app)
        .display(Display::Flex)
        .align_items(AlignItems::Center)
        .width(pct(100))
        .min_height(px(38.0))
        .padding_horizontal(px(14.0))
        .border_width_all(px(0.0))
        .border_radius_all((5.0, 5.0))
        .background_color(navigation_background(selected))
        .outline_color_all(retgui::palette::css::DODGER_BLUE)
        .outline_width_all(px(if selected { 2.0 } else { 0.0 }))
        .push(label)
        .finish()
}

fn sidebar(app: &mut App) -> Container {
    Container::new(app)
        .edit(app)
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .flex_shrink(0.0)
        .width(px(220.0))
        .height(pct(100))
        .padding(px(12.0), px(8.0), px(12.0), px(8.0))
        .row_gap(px(3.0))
        .border_width(px(0.0), px(1.0), px(0.0), px(0.0))
        .border_color_all(Color::from_rgb8(210, 214, 220))
        .background_color(navigation_background(false))
        .overflow(Overflow::Clip, Overflow::Scroll)
        .finish()
}

fn content_pane(app: &mut App) -> Container {
    Container::new(app)
        .edit(app)
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .flex_grow(1.0)
        .width(pct(100))
        .height(pct(100))
        .overflow(Overflow::Clip, Overflow::Clip)
        .finish()
}

fn select_example(app: &mut App, examples: &[GalleryExample], selected: usize) {
    for (index, example) in examples.iter().enumerate() {
        example.section.set_display(
            app,
            if index == selected {
                Display::Flex
            } else {
                Display::None
            },
        );
    }
}

fn gallery(app: &mut App) -> Container {
    let examples = Rc::new(gallery_examples(app));
    let sidebar = sidebar(app);
    let content = content_pane(app);
    let buttons = examples
        .iter()
        .enumerate()
        .map(|(index, example)| navigation_button(app, example.label, index == 0))
        .collect::<Vec<_>>();
    let selection = NavigationSelection::new(
        app,
        buttons
            .first()
            .expect("the gallery must contain at least one example")
            .as_dyn_element(),
    );
    select_example(app, &examples, 0);

    for (index, (example, button)) in examples.iter().zip(buttons).enumerate() {
        let examples = examples.clone();
        let button = button
            .edit(app)
            .add_click_listener(move |event, app| {
                select_example(app, &examples, index);
                selection.select(app, event.current_target());
                event.stop_propagation();
            })
            .finish();
        sidebar.push(app, button);
        content.push(app, example.section);
    }

    Container::new(app)
        .edit(app)
        .display(Display::Flex)
        .width(pct(100))
        .height(pct(100))
        .push(sidebar)
        .push(content)
        .finish()
}

pub fn main() {
    setup_logging();
    let mut app = App::new();
    let gallery = gallery(&mut app);
    Window::new(&mut app, "Gallery")
        .edit(&mut app)
        .display(Display::Flex)
        .overflow(Overflow::Clip, Overflow::Clip)
        .width(pct(100))
        .height(pct(100))
        .push(gallery)
        .finish();
    retgui_main(app, RetGuiOptions::basic("Gallery"));
}
