use std::path::PathBuf;
use std::str::FromStr;
use craft_retained::elements::{Dropdown, Element, Image, Slider, Text, TextInput, Window};
use craft_retained::style::{AlignItems, BoxShadow, FlexDirection, JustifyContent, Overflow, Unit};
use craft_retained::{craft_main, pct, px, rgba, Color, CraftOptions, ResourceIdentifier};
use util::setup_logging;

fn main() {
    setup_logging();

    let border_color = rgba(0, 0, 0, 255);
    Window::new("Counter")
        .flex_direction(FlexDirection::Column)
        .justify_content(Some(JustifyContent::Center))
        .align_items(Some(AlignItems::Center))
        .background_color(Color::WHITE)
        .width(pct(100))
        .height(pct(100))
        .gap(px(20), px(20))
        .push({
            Dropdown::new()
                .width(Unit::Px(100.0))
                .push(Text::new("Item 1").font_size(20.0).selectable(true))
                .push(Text::new("Item 2").font_size(20.0).selectable(true))
                .push(Text::new("Item 3").font_size(20.0).selectable(true))
                .push(Text::new("Item 4").font_size(20.0).selectable(true))
                .push(Text::new("Item 5").font_size(20.0).selectable(true))
                .push(Text::new("Item 6").font_size(20.0).selectable(true))
                .push(Text::new("Item 7").font_size(20.0).selectable(true))
                .push(Text::new("Item 8").font_size(20.0).selectable(true))
                .push(Text::new("Item 9").font_size(20.0).selectable(true))
                .push(Text::new("Item 10").font_size(20.0).selectable(true))
                .selected_item(2)
        })
        .push(Text::new("Sample text, this is sample text, hello!!!!!!!!!!!!!!!!!"))
        .push(Text::new("Sample text, this is sample text, hello!!!!!!!!!!!!!!!!!"))
        .push(Text::new("Sample text, this is sample text, hello!!!!!!!!!!!!!!!!!"))
        .push(Slider::new(20.0))
    ;

    craft_main(CraftOptions::basic("Counter"));
}
