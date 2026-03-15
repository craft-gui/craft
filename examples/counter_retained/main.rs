use craft_retained::elements::{Dropdown, Element, Text, Window};
use craft_retained::style::{AlignItems, FlexDirection, JustifyContent, Unit};
use craft_retained::{pct, px, Color, CraftOptions};

fn main() {

    Window::new("Counter")
        .flex_direction(FlexDirection::Column)
        .justify_content(Some(JustifyContent::Center))
        .align_items(Some(AlignItems::Center))
        .width(pct(100))
        .height(pct(100))
        .gap(px(20), px(20))
        .push({
            Dropdown::new()
                .background_color(Color::from_rgb8(150, 150, 152))
                .width(Unit::Px(100.0))
                .height(Unit::Px(30.0))
                .push(Text::new("Item 1").selectable(true))
                .push(Text::new("Item 2").selectable(true))
                .push(Text::new("Item 3").selectable(true))
                .push(Text::new("Item 4").selectable(true))
                .push(Text::new("Item 5").selectable(true))
                .push(Text::new("Item 6").selectable(true))
                .push(Text::new("Item 7").selectable(true))
                .push(Text::new("Item 8").selectable(true))
                .push(Text::new("Item 9").selectable(true))
                .push(Text::new("Item 10").selectable(true))
        });

    craft_retained::craft_main(CraftOptions::basic("Dropdown example"));
}
