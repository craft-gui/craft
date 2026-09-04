#[cfg(feature = "audio")]
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

#[cfg(feature = "audio")]
use retgui::elements::Audio;
use retgui::elements::{Button, Calendar, Checkbox, CheckboxGroup, Container, Dropdown, DynElement, Element, Elements, Image, Radio, RadioGroup, Slider, SliderDirection, State, Text, TextInput, TinyVg, Window};
use retgui::events::Event;
use retgui::geometry::Point;
use retgui::style::{AlignItems, Animation, BoxShadow, Display, FlexDirection, FontStyle, FontWeight, JustifyContent, KeyFrame, Overflow, Position, Repeat, StyleVariant, TextAlign, TimingFunction};
use retgui::{Brush, Color, ColorStop, Gradient, ResourceId, RetGuiOptions, auto, pct, px, retgui_main, rgb, rgba};
use serde::Deserialize;
use util::setup_logging;

pub fn title(elements: &mut Elements, value: &str) -> Text {
    Text::new(elements, value)
        .edit(elements)
        .font_weight(FontWeight::BOLD)
        .font_size(20.0)
        .margin(px(0.0), px(0.0), px(5.0), px(0.0))
        .finish()
}

pub fn animations(elements: &mut Elements) -> Text {
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
    Text::new(elements, "Animations")
        .edit(elements)
        .font_size(64.0)
        .font_weight(FontWeight::BOLD)
        .animations(vec![animation])
        .finish()
}

pub fn text_input(elements: &mut Elements) -> Container {
    let input = TextInput::new(elements, "An element for text input")
        .edit(elements)
        .width(px(200.0))
        .height(px(200.0))
        .finish();
    let heading = title(elements, "Text Input");
    Container::new(elements)
        .edit(elements)
        .display(Display::Block)
        .push(heading)
        .push(input)
        .finish()
}

pub fn dropdown(elements: &mut Elements) -> Container {
    let cat = Text::new(elements, "Cat");
    let dog = Text::new(elements, "Dog");
    let dropdown = Dropdown::new(elements)
        .edit(elements)
        .width(px(100.0))
        .push(cat)
        .push(dog)
        .selected_item(0)
        .finish();
    let heading = title(elements, "Dropdown");
    Container::new(elements)
        .edit(elements)
        .min_width(px(200.0))
        .display(Display::Block)
        .push(heading)
        .push(dropdown)
        .finish()
}

pub fn text(elements: &mut Elements) -> Container {
    let normal = Text::new(elements, "Normal Text with a Color")
        .edit(elements)
        .color(Color::from_rgb8(0, 0, 255))
        .finish();
    let bold = Text::new(elements, "Bold Text")
        .edit(elements)
        .font_weight(FontWeight::BOLD)
        .finish();
    let italic = Text::new(elements, "Italic Text")
        .edit(elements)
        .font_style(FontStyle::Italic)
        .finish();
    let bold_italic = Text::new(elements, "Bold & Italic Text")
        .edit(elements)
        .font_weight(FontWeight::BOLD)
        .font_style(FontStyle::Italic)
        .finish();
    let underlined = Text::new(elements, "Underlined Text")
        .edit(elements)
        .underline(Some(2.0), Color::from_rgb8(0, 255, 0), None)
        .finish();
    let left = Text::new(elements, "Left")
        .edit(elements)
        .text_align(TextAlign::Left)
        .finish();
    let center = Text::new(elements, "Center")
        .edit(elements)
        .text_align(TextAlign::Center)
        .finish();
    let right = Text::new(elements, "Right")
        .edit(elements)
        .text_align(TextAlign::Right)
        .finish();
    let heading = title(elements, "Text");
    Container::new(elements)
        .edit(elements)
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

pub fn tinyvg(elements: &mut Elements) -> Container {
    let image = TinyVg::new(elements, ResourceId::StaticBytes(include_bytes!("tiger.tvg")))
        .edit(elements)
        .width(px(250.0))
        .height(px(250.0))
        .finish();
    let heading = title(elements, "TinyVG");
    Container::new(elements)
        .edit(elements)
        .display(Display::Block)
        .push(heading)
        .push(image)
        .finish()
}

pub fn images(elements: &mut Elements) -> Container {
    let image = Image::new(elements, ResourceId::Url("https://picsum.photos/300/200".to_string()))
        .edit(elements)
        .width(px(300.0))
        .height(px(200.0))
        .finish();
    let heading = title(elements, "Image");
    Container::new(elements)
        .edit(elements)
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

pub fn async_weather(elements: &mut Elements) -> Container {
    let status = Text::new(elements, "Click the button for the current conditions.")
        .edit(elements)
        .width(px(280.0))
        .font_size(14.0)
        .finish();
    let label = Text::new(elements, "Refresh Weather")
        .edit(elements)
        .color(Color::WHITE)
        .selectable(false)
        .finish();
    let button = Button::new(elements)
        .edit(elements)
        .padding(px(5.0), px(15.0), px(5.0), px(15.0))
        .border_radius_all((4.0, 4.0))
        .background_color(Color::from_rgb8(35, 127, 183))
        .push(label)
        .add_click_listener(move |event, elements| {
            status.edit(elements).text("Loading...").finish();
            elements.spawn_local(fetch_amsterdam_weather(), move |weather, elements| {
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
                status.edit(elements).text(&message).finish();
            });
            event.stop_propagation();
        })
        .finish();
    let heading = title(elements, "Amsterdam Weather");
    let attribution = Text::new(elements, "Weather data by Open-Meteo")
        .edit(elements)
        .font_size(12.0)
        .finish();
    Container::new(elements)
        .edit(elements)
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .row_gap(px(8.0))
        .push(heading)
        .push(button)
        .push(status)
        .push(attribution)
        .finish()
}

pub fn gradient(elements: &mut Elements) -> Container {
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
    let linear_box = Container::new(elements)
        .edit(elements)
        .width(px(140.0))
        .height(px(90.0))
        .border_radius_all((8.0, 8.0))
        .background_gradient(linear.clone())
        .finish();
    let radial_box = Container::new(elements)
        .edit(elements)
        .width(px(140.0))
        .height(px(90.0))
        .border_radius_all((8.0, 8.0))
        .background_gradient(radial)
        .finish();
    let sweep_box = Container::new(elements)
        .edit(elements)
        .width(px(140.0))
        .height(px(90.0))
        .border_radius_all((8.0, 8.0))
        .background_gradient(sweep)
        .finish();
    let gradient_text = Text::new(elements, "Gradient Text")
        .edit(elements)
        .font_weight(FontWeight::BOLD)
        .text_gradient(linear.clone())
        .finish();
    let underline = Text::new(elements, "Gradient Underline")
        .edit(elements)
        .underline_gradient(Some(3.0), linear, None)
        .finish();
    let boxes = Container::new(elements)
        .edit(elements)
        .display(Display::Flex)
        .gap(px(10.0), px(10.0))
        .push(linear_box)
        .push(radial_box)
        .push(sweep_box)
        .finish();
    let heading = title(elements, "Gradients");
    Container::new(elements)
        .edit(elements)
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .row_gap(px(10.0))
        .push(heading)
        .push(gradient_text)
        .push(underline)
        .push(boxes)
        .finish()
}

pub fn box_shadows(elements: &mut Elements) -> Container {
    let border_color = rgb(0, 0, 0);
    let shadow = Container::new(elements)
        .edit(elements)
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
    let heading = title(elements, "Box Shadows");
    Container::new(elements)
        .edit(elements)
        .display(Display::Block)
        .push(heading)
        .push(shadow)
        .finish()
}

pub fn overlay(elements: &mut Elements) -> Container {
    let status = Text::new(elements, "Click where the cards overlap");
    let overlay_label = Text::new(elements, "Overlay")
        .edit(elements)
        .color(Color::WHITE)
        .selectable(false)
        .finish();
    let floating = Container::new(elements)
        .edit(elements)
        .overlay(true)
        .position(Position::Absolute)
        .inset(px(20.0), auto(), auto(), px(20.0))
        .width(px(150.0))
        .height(px(100.0))
        .padding_all(px(10.0))
        .background_color(Color::from_rgb8(76, 175, 80))
        .push(overlay_label)
        .add_click_listener(move |event, elements| {
            status.edit(elements).text("The overlay received the click").finish();
            event.stop_propagation();
        })
        .finish();
    let normal_label = Text::new(elements, "Normal sibling")
        .edit(elements)
        .color(Color::WHITE)
        .selectable(false)
        .finish();
    let normal = Container::new(elements)
        .edit(elements)
        .position(Position::Absolute)
        .inset(px(65.0), auto(), auto(), px(90.0))
        .width(px(120.0))
        .height(px(70.0))
        .padding_all(px(10.0))
        .background_color(Color::from_rgb8(33, 150, 243))
        .push(normal_label)
        .add_click_listener(move |event, elements| {
            status
                .edit(elements)
                .text("The normal sibling received the click")
                .finish();
            event.stop_propagation();
        })
        .finish();
    let cards = Container::new(elements)
        .edit(elements)
        .position(Position::Relative)
        .width(px(230.0))
        .height(px(155.0))
        .background_color(Color::from_rgb8(238, 238, 238))
        .push(floating)
        .push(normal)
        .finish();
    let heading = title(elements, "Overlay");
    Container::new(elements)
        .edit(elements)
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

pub fn multiple_windows(elements: &mut Elements) -> Container {
    let radius = (1.0, 1.0);
    let border = Color::BLACK;
    let width = px(1.0);
    let label = Text::new(elements, "Open a new window");
    let button = Button::new(elements)
        .edit(elements)
        .push(label)
        .padding(px(5.0), px(15.0), px(5.0), px(15.0))
        .border_radius(radius, radius, radius, radius)
        .border_color(border, border, border, border)
        .border_width(width, width, width, width)
        .add_click_listener(|_event, elements| {
            let greeting = Text::new(elements, "Hi!")
                .edit(elements)
                .font_size(32.0)
                .font_weight(FontWeight::BOLD)
                .finish();
            Window::new(elements, "A new window!")
                .edit(elements)
                .push(greeting)
                .finish();
        })
        .finish();
    let heading = title(elements, "Multiple Windows");
    Container::new(elements)
        .edit(elements)
        .display(Display::Block)
        .push(heading)
        .push(button)
        .finish()
}

pub fn sliders(elements: &mut Elements) -> Container {
    let first = Slider::new(elements, 20.0)
        .edit(elements)
        .value(70.0)
        .width(px(100.0))
        .height(px(10.0))
        .finish();
    let br = (0.0, 0.0);
    let second = Slider::new(elements, 14.0)
        .edit(elements)
        .value(20.0)
        .width(px(100.0))
        .height(px(10.0))
        .track_color(Color::from_rgb8(120, 150, 0))
        .border_radius(br, br, br, br)
        .thumb_border_radius(br, br, br, br)
        .finish();
    let third = Slider::new(elements, 20.0)
        .edit(elements)
        .value(70.0)
        .width(px(10.0))
        .height(px(100.0))
        .direction(SliderDirection::Vertical)
        .finish();
    let heading = title(elements, "Sliders");
    Container::new(elements)
        .edit(elements)
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .row_gap(px(15.0))
        .push(heading)
        .push(first)
        .push(second)
        .push(third)
        .finish()
}

pub fn scrollable(elements: &mut Elements) -> Container {
    let start = Text::new(elements, "The Start");
    let middle = Text::new(elements, "The Middle")
        .edit(elements)
        .margin(px(50.0), px(0.0), px(250.0), px(0.0))
        .finish();
    let end = Text::new(elements, "The End")
        .edit(elements)
        .padding(px(0.0), px(0.0), px(10.0), px(0.0))
        .finish();
    let scrollable = Container::new(elements)
        .edit(elements)
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
    let label = Text::new(elements, "Scroll to the top")
        .edit(elements)
        .color(Color::WHITE)
        .font_size(14.0)
        .padding(px(3.0), px(5.0), px(3.0), px(5.0))
        .finish();
    let button = Button::new(elements)
        .edit(elements)
        .width(px(120.0))
        .background_color(Color::from_rgb8(35, 127, 183))
        .add_click_listener(move |_event, elements| {
            scrollable.scroll_to_top(elements);
        })
        .push(label)
        .finish();
    let heading = title(elements, "Scrollable");
    Container::new(elements)
        .edit(elements)
        .display(Display::Block)
        .push(heading)
        .push(scrollable)
        .push(button)
        .finish()
}

pub fn radio_buttons(elements: &mut Elements) -> Container {
    let active = elements.insert_state("red".to_string());
    let green = Image::new(
        elements,
        ResourceId::Url("https://www.iconsdb.com/icons/preview/green/square-xxl.png".to_string()),
    )
    .edit(elements)
    .border_width_all(px(1))
    .border_color_all(rgba(0, 0, 0, 0))
    .finish();
    let red_label = Text::new(elements, "red");
    let red = Radio::new(elements, "red", "red", active)
        .edit(elements)
        .push(red_label)
        .finish();
    let green_radio = Radio::new(elements, "green", "green", active)
        .edit(elements)
        .push(green)
        .hide_radio()
        .finish();
    let blue_label = Text::new(elements, "blue");
    let blue = Radio::new(elements, "blue", "blue", active)
        .edit(elements)
        .push(blue_label)
        .finish();
    let group = RadioGroup::new(elements, "Pick a color")
        .edit(elements)
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .justify_content(JustifyContent::Center)
        .push(red)
        .push(green_radio)
        .push(blue)
        .add_radio_value_changed_listener(move |event, elements| {
            green
                .edit(elements)
                .border_color_all(if event.value.as_str() == "green" {
                    rgb(0, 100, 255)
                } else {
                    rgba(0, 0, 0, 0)
                })
                .finish();
        })
        .finish();
    let heading = title(elements, "Radio Button");
    Container::new(elements)
        .edit(elements)
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .push(heading)
        .push(group)
        .finish()
}

pub fn checkbox(elements: &mut Elements) -> Container {
    let coffee_label = Text::new(elements, "Coffee").edit(elements).selectable(false).finish();
    let coffee = Checkbox::new(elements, "coffee", true)
        .edit(elements)
        .push(coffee_label)
        .finish();
    let tea_label = Text::new(elements, "Tea").edit(elements).selectable(false).finish();
    let tea = Checkbox::new(elements, "tea", false)
        .edit(elements)
        .push(tea_label)
        .finish();
    let pork_label = Text::new(elements, "红烧肉").edit(elements).selectable(false).finish();
    let pork = Checkbox::new(elements, "红烧肉", false)
        .edit(elements)
        .push(pork_label)
        .finish();
    let curry_label = Text::new(elements, "カツカレー")
        .edit(elements)
        .selectable(false)
        .finish();
    let curry = Checkbox::new(elements, "カツカレー", false)
        .edit(elements)
        .push(curry_label)
        .finish();
    let group = CheckboxGroup::new(elements, "Select your favorite foods")
        .edit(elements)
        .add_checkbox_toggled_listener(move |event, _elements| {
            println!("checkbox toggled: {} - {}", event.label, event.status);
        })
        .flex_direction(FlexDirection::Column)
        .gap(px(15.0), px(15.0))
        .push(coffee)
        .push(tea)
        .push(pork)
        .push(curry)
        .finish();
    let heading = title(elements, "Checkbox");
    Container::new(elements)
        .edit(elements)
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .push(heading)
        .push(group)
        .finish()
}

#[cfg(feature = "audio")]
pub fn audio(elements: &mut Elements) -> Audio {
    let mut asset_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    asset_path.push("assets");
    asset_path.push("1-11. Mice on Venus.mp3");
    Audio::new(elements, Path::new(asset_path.as_path()))
}

#[cfg(not(feature = "audio"))]
pub fn audio(elements: &mut Elements) -> Container {
    Container::new(elements)
}

struct GalleryExample {
    label: &'static str,
    section: Container,
}

impl GalleryExample {
    fn new(elements: &mut Elements, label: &'static str, child: impl Element) -> Self {
        let section = Container::new(elements)
            .edit(elements)
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

    fn titled(elements: &mut Elements, label: &'static str, child: impl Element) -> Self {
        let heading = title(elements, label);
        let content = Container::new(elements)
            .edit(elements)
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .row_gap(px(12.0))
            .push(heading)
            .push(child)
            .finish();
        Self::new(elements, label, content)
    }
}

#[derive(Clone, Copy)]
struct NavigationSelection {
    active: State<DynElement>,
}

impl NavigationSelection {
    fn new(elements: &mut Elements, active: DynElement) -> Self {
        Self {
            active: elements.insert_state(active),
        }
    }

    fn select(&self, elements: &mut Elements, target: DynElement) {
        let previous = self.active.update(elements, |active| std::mem::replace(active, target));
        style_navigation_button(elements, previous, false);
        style_navigation_button(elements, target, true);
    }
}

fn gallery_examples(elements: &mut Elements) -> Vec<GalleryExample> {
    let animations = animations(elements);
    let audio = audio(elements);
    let calendar = Calendar::new(elements).edit(elements).start_year(1950).finish();
    let text_input = text_input(elements);
    let dropdown = dropdown(elements);
    let text = text(elements);
    let tinyvg = tinyvg(elements);
    let images = images(elements);
    let gradient = gradient(elements);
    let shadows = box_shadows(elements);
    let weather = async_weather(elements);
    let overlay = overlay(elements);
    let sliders = sliders(elements);
    let radios = radio_buttons(elements);
    let checkboxes = checkbox(elements);
    let scrollable = scrollable(elements);
    let windows = multiple_windows(elements);

    vec![
        GalleryExample::new(elements, "Animations", animations),
        GalleryExample::titled(elements, "Audio", audio),
        GalleryExample::titled(elements, "Calendar", calendar),
        GalleryExample::new(elements, "Text Input", text_input),
        GalleryExample::new(elements, "Dropdown", dropdown),
        GalleryExample::new(elements, "Text", text),
        GalleryExample::new(elements, "TinyVG", tinyvg),
        GalleryExample::new(elements, "Image", images),
        GalleryExample::new(elements, "Gradients", gradient),
        GalleryExample::new(elements, "Box Shadows", shadows),
        GalleryExample::new(elements, "Async", weather),
        GalleryExample::new(elements, "Overlay", overlay),
        GalleryExample::new(elements, "Sliders", sliders),
        GalleryExample::new(elements, "Radio Buttons", radios),
        GalleryExample::new(elements, "Checkboxes", checkboxes),
        GalleryExample::new(elements, "Scrollable", scrollable),
        GalleryExample::new(elements, "Multiple Windows", windows),
    ]
}

fn navigation_background(selected: bool) -> Color {
    if selected {
        Color::from_rgb8(214, 232, 250)
    } else {
        Color::from_rgb8(247, 248, 250)
    }
}

fn style_navigation_button(elements: &mut Elements, button: impl Element, selected: bool) {
    button
        .edit(elements)
        .background_color(navigation_background(selected))
        .outline_color_all(retgui::palette::css::DODGER_BLUE)
        .outline_width_all(px(if selected { 2.0 } else { 0.0 }))
        .finish();
}

fn navigation_button(elements: &mut Elements, label: &str, selected: bool) -> Button {
    let label = Text::new(elements, label)
        .edit(elements)
        .font_size(15.0)
        .selectable(false)
        .finish();
    Button::new(elements)
        .edit(elements)
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

fn sidebar(elements: &mut Elements) -> Container {
    Container::new(elements)
        .edit(elements)
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

fn content_pane(elements: &mut Elements) -> Container {
    Container::new(elements)
        .edit(elements)
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .flex_grow(1.0)
        .width(pct(100))
        .height(pct(100))
        .overflow(Overflow::Clip, Overflow::Clip)
        .finish()
}

fn select_example(elements: &mut Elements, examples: &[GalleryExample], selected: usize) {
    for (index, example) in examples.iter().enumerate() {
        example.section.set_display(
            elements,
            if index == selected {
                Display::Flex
            } else {
                Display::None
            },
        );
    }
}

fn gallery(elements: &mut Elements) -> Container {
    let examples = Rc::new(gallery_examples(elements));
    let sidebar = sidebar(elements);
    let content = content_pane(elements);
    let buttons = examples
        .iter()
        .enumerate()
        .map(|(index, example)| navigation_button(elements, example.label, index == 0))
        .collect::<Vec<_>>();
    let selection = NavigationSelection::new(
        elements,
        buttons
            .first()
            .expect("the gallery must contain at least one example")
            .as_dyn_element(),
    );
    select_example(elements, &examples, 0);

    for (index, (example, button)) in examples.iter().zip(buttons).enumerate() {
        let examples = examples.clone();
        let button = button
            .edit(elements)
            .add_click_listener(move |event, elements| {
                select_example(elements, &examples, index);
                selection.select(elements, event.current_target());
                event.stop_propagation();
            })
            .finish();
        sidebar.push(elements, button);
        content.push(elements, example.section);
    }

    Container::new(elements)
        .edit(elements)
        .display(Display::Flex)
        .width(pct(100))
        .height(pct(100))
        .push(sidebar)
        .push(content)
        .finish()
}

pub fn main() {
    setup_logging();
    let mut elements = Elements::new();
    let gallery = gallery(&mut elements);
    Window::new(&mut elements, "Gallery")
        .edit(&mut elements)
        .display(Display::Flex)
        .overflow(Overflow::Clip, Overflow::Clip)
        .width(pct(100))
        .height(pct(100))
        .push(gallery)
        .finish();
    retgui_main(elements, RetGuiOptions::basic("Gallery"));
}
