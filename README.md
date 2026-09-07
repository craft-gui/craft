<p align="center"><img src="./images/retgui_logo.svg" alt="The RetGui logo" width="40%"></p>

<p align="center">
  <a href="./LICENSE"><img src="https://img.shields.io/badge/license-Unlicense-blue.svg" alt="License: Unlicense"></a>
  <a href="https://discord.gg/Atb8nuAub2"><img src="https://img.shields.io/discord/1382383100562243746?logo=discord&logoColor=%23ffffff&labelColor=%236A7EC2&color=%237389D8" alt="Discord"></a>
</p>

## Introduction
RetGui is a Rust library for creating desktop user interfaces.

## Installation
Add the following to your `Cargo.toml`:
```toml
[dependencies.retgui]
git = "https://github.com/RetGui/RetGui"
default-features = false
features = ["system_fonts", "vello_hybrid_renderer"]
```

## Example

```rust
use retgui::elements::{Container, Element, State, Text, Window};
use retgui::events::Event;
use retgui::style::{AlignItems, FlexDirection, JustifyContent};
use retgui::{App, Color, RetGuiOptions, pct, px, rgb};

fn create_button(
    app: &mut App,
    label: &str,
    base_color: Color,
    delta: i64,
    count: State<i64>,
    count_text: Text,
) -> Container {
    let label = Text::new(app, label)
        .edit(app)
        .font_size(24.0)
        .color(Color::WHITE)
        .selectable(false)
        .finish();

    Container::new(app)
        .edit(app)
        .border_width(px(1), px(2), px(3), px(4))
        .border_color_all(rgb(0, 0, 0))
        .border_radius_all((10.0, 10.0))
        .padding(px(15), px(30), px(15), px(30))
        .justify_content(JustifyContent::Center)
        .background_color(base_color)
        .on_click(move |event, app| {
            let count = count.update(app, |count| {
                *count += delta;
                *count
            });
            count_text
                .edit(app)
                .text(&format!("Count: {count}"))
                .finish();
            event.stop_propagation();
        })
        .push(label)
        .finish()
}

fn main() {
    let mut app = App::new();
    let count = app.insert_state(0_i64);
    let count_text = Text::new(&mut app, "Count: 0");
    let subtract = create_button(
        &mut app, "-", rgb(244, 67, 54), -1, count, count_text,
    );
    let add = create_button(
        &mut app, "+", rgb(76, 175, 80), 1, count, count_text,
    );
    let buttons = Container::new(&mut app)
        .edit(&mut app)
        .gap(px(20), px(20))
        .push(subtract)
        .push(add)
        .finish();

    Window::new(&mut app, "Counter")
        .edit(&mut app)
        .flex_direction(FlexDirection::Column)
        .justify_content(JustifyContent::Center)
        .align_items(AlignItems::Center)
        .width(pct(100))
        .height(pct(100))
        .gap(px(20), px(20))
        .push(count_text)
        .push(buttons)
        .finish();

    retgui::retgui_main(app, RetGuiOptions::basic("Counter"));
}
```

## Supported Platforms
| Platform | Status                             | Accessibility Status |
|----------|------------------------------------|----------------------|
| Windows  | Officially supported               | In Progress          |
| macOS    | Officially supported               | Planned              |
| Linux    | Officially supported               | Planned              |
| Web      | Runs, but not officially supported | TBD                  |
| Android  | Runs, but not officially supported | TBD                  |
| iOS      | Runs, but not officially supported | TBD                  |

## Features
| Feature               | Description                                                                 | Platforms Not Supported |
|-----------------------|-----------------------------------------------------------------------------|-------------------------|
| audio                 | Enables playing audio via MiniAudio.                                        | Web, Android, and iOS   |
| clipboard             | Enables clipboard support in text elements.                                 |                         |
| vello_cpu_renderer    | Enables the Vello CPU renderer.                                             |                         |
| vello_hybrid_renderer | Enables the Vello Hybrid renderer.                                          |                         |
| http_client           | Enables the HTTP client, which allows loading resources from URLs and more. |                         |
| system_fonts          | Tells the font engine to load system fonts automatically.                   |                         |
| png                   | Enables decoding PNG images.                                                |                         |
| jpeg                  | Enables decoding JPEG images                                                |                         |
| markdown              | Provides the ability to render markdown via an element.                     |                         |
| link                  | Allows opening links with your default browser.                             |                         |

## Showcase
<p>
  <img src="./images/gallery.png" alt="The RetGui gallery example." width="40%">
  <img src="./images/counter.png" alt="The RetGui counter example." width="40%">
</p>

## FAQ
### 1. Are Android, iOS, and the Web Supported? 
RetGui can run on those platforms, but they are not officially supported. We would like to support those platforms, but it requires a lot of platform integration. Please use SwiftUI, Jetpack Compose, Flutter, and etc.

### 2. Why do mutations need `App`?
App owns all RetGUI data and an element is just a handle to some data in app.

## License
Distributed under the Unlicense License. See the [LICENSE](./LICENSE) for more information.
