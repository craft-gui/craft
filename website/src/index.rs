use oku::components::ComponentSpecification;
use oku::elements::{Container, ElementStyles, Text};
use oku::palette;
use oku::style::Wrap::WrapReverse;
use oku::style::{AlignItems, Display, FlexDirection, Overflow, Unit, Weight};

fn hero_intro() -> ComponentSpecification {
    Container::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .align_items(AlignItems::Center)
        .wrap(WrapReverse)
        .width("100%")
        .push(
            Container::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .margin("0%", "5%", "0%", "10%")
                .width("39%")
                .push(Text::new("Oku")
                    .font_size(32.0)
                    .font_weight(Weight::SEMIBOLD)
                    .color(palette::css::WHITE))
                .push(
                    Text::new("A reactive GUI framework that allows developers to build interactive graphical user interfaces efficiently and elegantly.")
                        .font_size(24.0)
                        .color(palette::css::WHITE),
                )
        )
        .push(Container::new().width("40%").height("500px").margin("0%", "5%", "0%", "0%"))
        // .push(Image::new(ResourceIdentifier::File(PathBuf::from("Intro Hero Image.png"))).width("40%").margin("0%", "5%", "0%", "0%"))
        .component()
}

fn hero_features() -> ComponentSpecification {
    fn hero_feature_item(title: &str, text: &str) -> ComponentSpecification {
        Container::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .align_items(AlignItems::FlexStart)
            .width("80%")
            .gap("20px")
            .push(Text::new(title).font_size(24.0).font_weight(Weight::SEMIBOLD).color(palette::css::WHITE))
            .push(Text::new(text).font_size(18.0).color(palette::css::WHITE))
            .component()
    }

    Container::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .align_items(AlignItems::Center)
        .padding("5%", "10%", "0%", "10%")
        .width("100%")
        .gap("50px")
        .push(
            hero_feature_item("Components", "Components encapsulate both a view and an update function, enabling modular and reusable UI elements that dynamically respond to state changes.")
        )
        .push(
            hero_feature_item("Views", "Views in Oku are constructed using Components and Elements, forming the structural and visual hierarchy of an interface. They determine how UI elements are arranged and rendered on the screen.")
        )
        .push(
            hero_feature_item("Messages", "Messages in Oku facilitate communication between Components and Views. They define user interactions and system-triggered events, allowing the UI to respond dynamically to changes.")
        )
        .component()
}

pub(crate) fn index_page() -> ComponentSpecification {
    Container::new()
        .display(Display::Flex)
        .width("100%")
        .flex_direction(FlexDirection::Column)
        .flex_grow(1.0) // Take up remaining space
        .overflow(Overflow::Scroll) // Allow content overflow to scroll
        .padding(Unit::Px(20.0), Unit::Px(20.0), Unit::Percentage(10.0), Unit::Px(20.0))
        .push(hero_intro())
        .push(hero_features())
        .component()
}
