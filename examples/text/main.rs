use craft_retained::elements::{Element, TextInput, Window};
use craft_retained::style::{AlignItems, Display, FlexDirection, JustifyContent, Overflow, Unit};
use craft_retained::{
    elements::Container,
    rgb,
};

fn main() {
    let container = Container::new();

    let text = TextInput::new(include_str!("../counter_retained/main.rs"));
    text.borrow_mut()
        .overflow(Overflow::Visible, Overflow::Scroll)
        .width(Unit::Px(600.0))
        .height(Unit::Px(600.0))
        .display(Display::Block);

    container
        .borrow_mut()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .justify_content(Some(JustifyContent::Center))
        .align_items(Some(AlignItems::Center))
        .width(Unit::Percentage(100.0))
        .height(Unit::Percentage(100.0))
        .gap(Unit::Px(20.0), Unit::Px(20.0))
        .font_size(72.0)
        .color(rgb(50, 50, 50))
        .push(text);

    let root = Window::new();
    root.borrow_mut().push(container);

    use craft_retained::CraftOptions;
    util::setup_logging();
    craft_retained::craft_main(CraftOptions::basic("text"));
}