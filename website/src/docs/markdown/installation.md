# Installation

This tutorial currently targets the main branch. Create a new Rust project using Cargo:

```bash
cargo new --bin hello_craft
```
Next, add the `craft` crate to your `Cargo.toml` file:

```toml
[dependencies.craft]
git = "https://github.com/craft-gui/craft.git"
branch = "main"
features = ["vello_renderer", "devtools", "accesskit", "system_fonts"]
package = "craft_gui"
```

***

## Next Steps
1. Create a counter app.
2. Learn the Elm architecture.
    * View
    * Update
    * Async Updates
3. Learn about the craft.
    * Widgets
    * Layout
    * Styling

[Google.com](https://www.google.com/search?q=craft+gui+rust) is a good place to start.

![A mushroom-head robot drinking bubble tea](https://raw.githubusercontent.com/Codecademy/docs/main/media/codey.jpg 'Codey, the Codecademy mascot, drinking bubble tea')