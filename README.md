<p align="center"><img src="./images/retgui_logo.svg" alt="The RetGui logo" width="40%"></p>

<p align="center">
  <a href="./LICENSE"><img src="https://img.shields.io/badge/license-Unlicense-blue.svg" alt="License: Unlicense"></a>
  <a href="https://discord.gg/Atb8nuAub2"><img src="https://img.shields.io/discord/1382383100562243746?logo=discord&logoColor=%23ffffff&labelColor=%236A7EC2&color=%237389D8" alt="Discord"></a>
</p>

## Introduction

RetGui is a library for creating desktops user interfaces. retgui_retained provides platform independent widgets.

## Example

```rust
use std::cell::RefCell;
use std::rc::Rc;

use retgui_retained::elements::{Container, Element, Text, Window};
use retgui_retained::events::ui_events::pointer::PointerButton;
use retgui_retained::style::{AlignItems, FlexDirection, JustifyContent};
use retgui_retained::{Color, RetGuiOptions, pct, px, rgb};

fn create_button(label: &str, base_color: Color, delta: i64, state: Rc<RefCell<i64>>, count_text: Text) -> Container {
    Container::new()
        .border_width(px(1), px(2), px(3), px(4))
        .border_color_all(rgb(0, 0, 0))
        .border_radius_all((10.0, 10.0))
        .padding(px(15), px(30), px(15), px(30))
        .justify_content(Some(JustifyContent::Center))
        .background_color(base_color)
        .on_click(Rc::new(move |event| {
            *state.borrow_mut() += delta;
            count_text.clone().text(&format!("Count: {}", state.borrow()));
            event.prevent_propagate();
        }))
        .push(Text::new(label).font_size(24.0).color(Color::WHITE).selectable(false))
}

fn main() {
    let count = Rc::new(RefCell::new(0));
    let count_text = Text::new(&format!("Count: {}", count.borrow()));

    Window::new()
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
                .push(create_button(
                    "+",
                    rgb(76, 175, 80),
                    1,
                    count.clone(),
                    count_text.clone(),
                ))
        });

    retgui_retained::retgui_main(RetGuiOptions::basic("Counter"));
}
```


## Showcase
<p>
  <img src="./images/gallery.png" alt="The RetGui gallery example." width="40%">
  <img src="./images/counter.png" alt="The RetGui gallery example." width="40%">
</p>
<p align="center">
</p>

## FAQ
### 1. Are Android, iOS, and the Web Supported? 
No. We would like to support those platforms, but it requires a lot of platform integration. Please use SwiftUI, Jetpack Compose, Flutter, and etc.

### 2. Why do I have to clone an Element into a separate variable before using it in a callback?
`Element` is `Clone`, but not Copy, so it requires a clone. There is a Rust project goal for improving the ergonomics around this behavior.
https://github.com/rust-lang/rust-project-goals/issues/107