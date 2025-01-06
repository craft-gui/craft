use oku::elements::Container;
use oku::elements::ElementStyles;
use oku::elements::Text;
use oku::style::{Display, JustifyContent};
use oku::style::FlexDirection;
use oku::style::Overflow;
use oku::style::Unit;
use oku::{oku_main_with_options, OkuOptions, RendererType};
use oku::components::ComponentSpecification;
use oku::renderer::color::Color;

fn main() {
    oku_main_with_options(
        create_layout(),
        Some(OkuOptions {
            renderer: RendererType::default(),
            ..Default::default()
        }),
    )
}

fn create_layout() -> ComponentSpecification {
    Container::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .width(Unit::Percentage(100.0))
        .height(Unit::Percentage(100.0)) // Occupy full height of the viewport
        .background(Color::from_rgb8(240, 240, 240)) // Light background for the layout
        .push(create_sidebar())
        .push(create_main_content())
        .component()
}

fn create_sidebar() -> ComponentSpecification {
    Container::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .width(Unit::Px(300.0)) // Fixed width for the sidebar
        .min_width(Unit::Px(250.0)) // Minimum width for the sidebar
        .height(Unit::Percentage(100.0)) // Ensure sidebar matches parent height
        .background(Color::from_rgb8(50, 50, 50)) // Dark background for the sidebar
        .padding(Unit::Px(10.0), Unit::Px(10.0), Unit::Px(10.0), Unit::Px(10.0))
        .push(
            Text::new("Sidebar")
                .font_size(18.0)
                .color(Color::from_rgb8(200, 200, 200)), // Softer white text for the title
        )
        .push(create_navigation_links())
        .component()
}

fn create_navigation_links() -> ComponentSpecification {
    Container::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column) // Stack links in a vertical column
        .justify_content(JustifyContent::Start) // Align links to the start
        .gap(Unit::Px(10.0)) // Add spacing between links
        .push(
            create_link("Home"),
        )
        .push(
            create_link("About"),
        )
        .push(
            create_link("Contact"),
        )
        .component()
}

fn create_link(label: &str) -> ComponentSpecification {
    Container::new()
        .padding(Unit::Px(10.0), Unit::Px(10.0), Unit::Px(10.0), Unit::Px(10.0)) // Add padding to each link
        .background(Color::from_rgb8(70, 70, 70)) // Slightly lighter dark background for the links
        .push(
            Text::new(label)
                .font_size(14.0)
                .color(Color::from_rgb8(220, 220, 220)), // Light text for readability
        )
        .component()
}

fn create_main_content() -> ComponentSpecification {
    Container::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .flex_grow(1.0) // Take up remaining space
        .height(Unit::Percentage(100.0)) // Match parent height
        .overflow(Overflow::Scroll) // Allow content overflow to scroll
        .background(Color::from_rgb8(255, 255, 255)) // Clean white background for the main content
        .padding(Unit::Px(20.0), Unit::Px(20.0), Unit::Px(0.0), Unit::Px(20.0)) // Avoid bottom padding
        .push(
            Text::new("About Oku")
                .font_size(24.0)
                .color(Color::from_rgb8(50, 50, 50)), // Dark text for contrast
        )
        .push(
            Text::new("Oku is a reactive GUI framework. It allows developers to build interactive graphical user interfaces efficiently and elegantly.")
                .font_size(18.0)
                .color(Color::from_rgb8(70, 70, 70)),
        )
        .push(
            Text::new("Views in Oku are created using Components and Elements. Components define the structure, layout, and behavior of an interface, while Elements represent the visual building blocks.")
                .font_size(18.0)
                .color(Color::from_rgb8(70, 70, 70)),
        )
        .push(
            Text::new("Updates to the interface are performed by handling messages from a Component. This approach makes the UI highly responsive and easy to maintain.")
                .font_size(18.0)
                .color(Color::from_rgb8(70, 70, 70)),
        )
        .component()
}
