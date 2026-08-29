# Overview

## What is RetGUI?

RetGUI is a retained Rust GUI library.
Elements and events are the low level primitives that make up a RetGUI app.
Elements are cloneable and reference counted at runtime.
While the code remains safe, this is a tradeoff between Rust's focus on compile time correctness and ease of use.
Styling and layout are roughly based on the web.
Keeping application state and UI tree in sync is left up the developer.
No state management or styling systems are included.
It should be quite easy to add in Elm, reactive diffing, or fine grain reactivity on top.
If you are looking for a simple framework without the borrow checker getting in your way you may find RetGUI interesting.

## Hello World
A simple app can be setup as follows.
Note retgui_main is blocking and will run until all windows have closed.

```toml
[package]
name = "hello_world"
version = "0.1.0"
edition = "2024"

[dependencies]

[dependencies.retgui]
git = "https://github.com/RetGui/RetGui.git"
rev = "c608ee19e89f5f14272caef48a571be90cd75509"
features = ["default"]

[profile.dev.package."*"]
opt-level = 1
```

```rust
use retgui::elements::{Element, Text, Window};
use retgui::{RetGuiOptions, retgui_main};

fn main() {
    Window::new("Hello World App").push(Text::new("Hello World!"));
    retgui_main(RetGuiOptions::basic("hello_world_app"));
}
```