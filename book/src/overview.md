# Overview

## What is RetGUI?

RetGUI is a retained Rust GUI library.
Elements and events are the low level primitives that make up a RetGUI app.
Elements are copy and are lightweight handles.
Elements are dropped when they are deleted from the UI tree and any dangling handles will ignore future reads/writes.
Styling and layout are roughly based on the web.
Keeping application state and UI tree in sync is left up the developer.
No state management or styling systems are included.
It should be quite easy to add in Elm, reactive diffing, or fine grain reactivity on top.
If you are looking for a simple retained GUI, you should try out RetGUI.

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
use retgui::{Elements, RetGuiOptions, retgui_main};

fn main() {
    let mut elements = Elements::new();
    Window::new(&mut elements, "Hello World App")
        .edit(&mut elements)
        .push_with(|elements| Text::new(elements, "Hello World!"))
        .finish();
    retgui_main(elements, RetGuiOptions::basic("hello_world_app"));
}
```