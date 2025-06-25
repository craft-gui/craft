# 📜 Craft
[![CI](https://github.com/craft-gui/craft/actions/workflows/ci.yml/badge.svg)](https://github.com/craft-gui/craft/actions/workflows/ci.yml)
[![License: Unlicense](https://img.shields.io/badge/license-Unlicense-blue.svg)](./LICENSE)
[![Discord](https://img.shields.io/discord/1382383100562243746?logo=discord&logoColor=%23ffffff&labelColor=%236A7EC2&color=%237389D8)](https://discord.gg/Atb8nuAub2)

Craft is a reactive GUI. Views are created using Components and Elements.
Updates are performed by handling messages from a Component.

<p align="center">
  <img src="./images/ani_list.png" alt="The ani list example." width="45%">
  <img src="./images/svg_to_tvg.png" alt="The text editor example" width="45%">
</p>
<p align="center">
  <img src="./images/tour.png" alt="The inputs example." width="45%">
  <img src="./images/counter.png" alt="The counter example." width="45%">
</p>

### Counter

```rust
use craft::components::{Context, Component, ComponentSpecification};
use craft::elements::{Container, ElementStyles, Text};
use craft::events::ui_events::pointer::PointerButtonUpdate;
use craft::style::FlexDirection;

#[derive(Default)]
pub struct Counter {
    count: i64,
}

impl Component for Counter {
    type GlobalState = ();
    type Props = ();
    type Message = ();

    fn view(context: &mut Context<Self>) -> ComponentSpecification {
        fn button(label: &str, delta: i64) -> Container {
            Container::new()
                .on_pointer_up(move |c: &mut Context<Counter>, _: &PointerButtonUpdate| {
                    c.state_mut().count += delta;
                })
                .push(Text::new(label).disable_selection().padding(10, 10, 10, 10))
        }

        Container::new()
            .push(Text::new(&format!("Count: {}", context.state().count)).disable_selection())
            .flex_direction(FlexDirection::Column)
            .push(
                Container::new()
                    .flex_direction(FlexDirection::Row)
                    .push(button("-", -1))
                    .push(button("+", 1))
            ).component()
    }
}

fn main() {
    craft::craft_main(Counter::component(), (), craft::CraftOptions::basic("Counter"));
}

```

## Goals

* Reactive
* Components
* Pure Rust without procedural macros
* Web-like styling
* Cross Platform

# Features

* ✅ Reactive Components
* ✅ Async Updates
* ✅ Text Rendering
* ✅ Windows/Linux
* ✅ Android(basic)
* ✅ Web(basic)
* ✅ Image Support
* ✅ DPI Scaling Support
* ⬜️ Transform (Rotation, Skew, Scale) Support
* ✅ Mac
* ⬜️ iOS
* ✅ Text Input (Basic)
* ✅ IME Support (Basic)
* ⬜️ Animations
* ✅ Scrollables (Basic)
* ⬜️ Documentation
* ⬜️ Tests
* ⬜ Videos
* ⬜ SVGs
* ✅ Accessibility (Very Basic)

## Run Examples:

```shell
cargo run --package counter
```
