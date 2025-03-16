use crate::components::ComponentSpecification;
use crate::devtools::dev_tools_colors::{BORDER_COLOR, FIELD_NAME_COLOR, FIELD_VALUE_COLOR, ROW_BACKGROUND_COLOR};
use crate::elements::element::Element;
use crate::elements::{Container, ElementStyles, Span, Text};
use crate::style::style_flags::StyleFlags;
use crate::style::Display::Flex;
use crate::style::{FlexDirection, Unit};
use crate::Color;
use taffy::Overflow;

fn format_option<T: std::fmt::Debug>(option: Option<T>) -> String {
    option.map_or("None".to_string(), |value| format!("{:?}", value))
}

fn field_row(
    field_name: &str,
    field_name_color: Color,
    field_value: &str,
    field_value_color: Color,
) -> ComponentSpecification {
    Container::new()
        .push(
            Text::new(field_name.to_lowercase().as_str())
                .color(field_name_color)
                .push_span(Span::new(field_value.to_lowercase().as_str()).color(field_value_color)),
        )
        .padding("0px", "10px", "0px", "10px")
        .component()
}

pub(crate) fn styles_window_view(selected_element: Option<&dyn Element>) -> ComponentSpecification {
    let mut styles_window = Container::new()
        .width(Unit::Percentage(100.0))
        .display(Flex)
        .flex_direction(FlexDirection::Column)
        .height("50%")
        .max_height("50%")
        .overflow(Overflow::Scroll)
        .background(ROW_BACKGROUND_COLOR)
        .push(Container::new().border_width("2px", "0px", "2px", "0px").border_color(BORDER_COLOR).push(
            Text::new("Styles Window").color(Color::from_rgb8(230, 230, 230)).padding("10px", "0px", "10px", "10px"),
        ))
        .component();

    if let Some(selected_element) = selected_element {
        let style = selected_element.style();

        // Font Family
        if style.dirty_flags.contains(StyleFlags::FONT_FAMILY) && style.font_family().is_some() {
            styles_window = styles_window.push(field_row(
                "Font Family: ",
                FIELD_NAME_COLOR,
                style.font_family().unwrap(),
                FIELD_VALUE_COLOR,
            ));
        }

        // Box Sizing
        if style.dirty_flags.contains(StyleFlags::BOX_SIZING) {
            styles_window = styles_window.push(field_row(
                "Box Sizing: ",
                FIELD_NAME_COLOR,
                format!("{:?}", style.box_sizing()).as_str(),
                FIELD_VALUE_COLOR,
            ));
        }

        // Scrollbar Width
        if style.dirty_flags.contains(StyleFlags::SCROLLBAR_WIDTH) {
            styles_window = styles_window.push(field_row(
                "Scrollbar Width: ",
                FIELD_NAME_COLOR,
                style.scrollbar_width().to_string().as_str(),
                FIELD_VALUE_COLOR,
            ));
        }

        // Position
        if style.dirty_flags.contains(StyleFlags::POSITION) {
            styles_window = styles_window.push(field_row(
                "Position: ",
                FIELD_NAME_COLOR,
                format!("{:?}", style.position()).as_str(),
                FIELD_VALUE_COLOR,
            ));
        }

        // Margin
        if style.dirty_flags.contains(StyleFlags::MARGIN) {
            styles_window = styles_window.push(field_row(
                "Margin Top: ",
                FIELD_NAME_COLOR,
                style.margin()[0].to_string().as_str(),
                FIELD_VALUE_COLOR,
            ));
            styles_window = styles_window.push(field_row(
                "Margin Right: ",
                FIELD_NAME_COLOR,
                style.margin()[1].to_string().as_str(),
                FIELD_VALUE_COLOR,
            ));
            styles_window = styles_window.push(field_row(
                "Margin Bottom: ",
                FIELD_NAME_COLOR,
                style.margin()[2].to_string().as_str(),
                FIELD_VALUE_COLOR,
            ));
            styles_window = styles_window.push(field_row(
                "Margin Left: ",
                FIELD_NAME_COLOR,
                style.margin()[3].to_string().as_str(),
                FIELD_VALUE_COLOR,
            ));
        }

        // Padding
        if style.dirty_flags.contains(StyleFlags::PADDING) {
            styles_window = styles_window.push(field_row(
                "Padding Top: ",
                FIELD_NAME_COLOR,
                style.padding()[0].to_string().as_str(),
                FIELD_VALUE_COLOR,
            ));
            styles_window = styles_window.push(field_row(
                "Padding Right: ",
                FIELD_NAME_COLOR,
                style.padding()[1].to_string().as_str(),
                FIELD_VALUE_COLOR,
            ));
            styles_window = styles_window.push(field_row(
                "Padding Bottom: ",
                FIELD_NAME_COLOR,
                style.padding()[2].to_string().as_str(),
                FIELD_VALUE_COLOR,
            ));
            styles_window = styles_window.push(field_row(
                "Padding Left: ",
                FIELD_NAME_COLOR,
                style.padding()[3].to_string().as_str(),
                FIELD_VALUE_COLOR,
            ));
        }

        // Gap
        if style.dirty_flags.contains(StyleFlags::GAP) {
            styles_window = styles_window.push(field_row(
                "Row Gap: ",
                FIELD_NAME_COLOR,
                style.gap()[0].to_string().as_str(),
                FIELD_VALUE_COLOR,
            ));
            styles_window = styles_window.push(field_row(
                "Column Gap: ",
                FIELD_NAME_COLOR,
                style.gap()[1].to_string().as_str(),
                FIELD_VALUE_COLOR,
            ));
        }

        // Inset
        if style.dirty_flags.contains(StyleFlags::INSET) {
            styles_window = styles_window.push(field_row(
                "Inset Top: ",
                FIELD_NAME_COLOR,
                style.inset()[0].to_string().as_str(),
                FIELD_VALUE_COLOR,
            ));
            styles_window = styles_window.push(field_row(
                "Inset Right: ",
                FIELD_NAME_COLOR,
                style.inset()[1].to_string().as_str(),
                FIELD_VALUE_COLOR,
            ));
            styles_window = styles_window.push(field_row(
                "Inset Bottom: ",
                FIELD_NAME_COLOR,
                style.inset()[2].to_string().as_str(),
                FIELD_VALUE_COLOR,
            ));
            styles_window = styles_window.push(field_row(
                "Inset Left: ",
                FIELD_NAME_COLOR,
                style.inset()[3].to_string().as_str(),
                FIELD_VALUE_COLOR,
            ));
        }

        // Width
        if style.dirty_flags.contains(StyleFlags::WIDTH) {
            styles_window = styles_window.push(field_row(
                "Width: ",
                FIELD_NAME_COLOR,
                style.width().to_string().as_str(),
                FIELD_VALUE_COLOR,
            ));
        }

        // Height
        if style.dirty_flags.contains(StyleFlags::HEIGHT) {
            styles_window = styles_window.push(field_row(
                "Height: ",
                FIELD_NAME_COLOR,
                style.height().to_string().as_str(),
                FIELD_VALUE_COLOR,
            ));
        }

        // Max Width
        if style.dirty_flags.contains(StyleFlags::MAX_WIDTH) {
            styles_window = styles_window.push(field_row(
                "Max Width: ",
                FIELD_NAME_COLOR,
                style.max_width().to_string().as_str(),
                FIELD_VALUE_COLOR,
            ));
        }

        // Max Height
        if style.dirty_flags.contains(StyleFlags::MAX_HEIGHT) {
            styles_window = styles_window.push(field_row(
                "Max Height: ",
                FIELD_NAME_COLOR,
                style.max_height().to_string().as_str(),
                FIELD_VALUE_COLOR,
            ));
        }

        // Min Width
        if style.dirty_flags.contains(StyleFlags::MIN_WIDTH) {
            styles_window = styles_window.push(field_row(
                "Min Width: ",
                FIELD_NAME_COLOR,
                style.min_width().to_string().as_str(),
                FIELD_VALUE_COLOR,
            ));
        }

        // Min Height
        if style.dirty_flags.contains(StyleFlags::MIN_HEIGHT) {
            styles_window = styles_window.push(field_row(
                "Min Height: ",
                FIELD_NAME_COLOR,
                style.min_height().to_string().as_str(),
                FIELD_VALUE_COLOR,
            ));
        }

        // X
        if style.dirty_flags.contains(StyleFlags::X) {
            styles_window = styles_window.push(field_row(
                "X: ",
                FIELD_NAME_COLOR,
                style.x().to_string().as_str(),
                FIELD_VALUE_COLOR,
            ));
        }

        // Y
        if style.dirty_flags.contains(StyleFlags::Y) {
            styles_window = styles_window.push(field_row(
                "Y: ",
                FIELD_NAME_COLOR,
                style.y().to_string().as_str(),
                FIELD_VALUE_COLOR,
            ));
        }

        // Display
        if style.dirty_flags.contains(StyleFlags::DISPLAY) {
            styles_window = styles_window.push(field_row(
                "Display: ",
                FIELD_NAME_COLOR,
                format!("{:?}", style.display()).as_str(),
                FIELD_VALUE_COLOR,
            ));
        }

        // Wrap
        if style.dirty_flags.contains(StyleFlags::WRAP) {
            styles_window = styles_window.push(field_row(
                "Wrap: ",
                FIELD_NAME_COLOR,
                format!("{:?}", style.wrap()).as_str(),
                FIELD_VALUE_COLOR,
            ));
        }

        // Align Items
        if style.dirty_flags.contains(StyleFlags::ALIGN_ITEMS) {
            styles_window = styles_window.push(field_row(
                "Align Items: ",
                FIELD_NAME_COLOR,
                format_option(style.align_items()).as_str(),
                FIELD_VALUE_COLOR,
            ));
        }

        // Justify Content
        if style.dirty_flags.contains(StyleFlags::JUSTIFY_CONTENT) {
            styles_window = styles_window.push(field_row(
                "Justify Content: ",
                FIELD_NAME_COLOR,
                format_option(style.justify_content()).as_str(),
                FIELD_VALUE_COLOR,
            ));
        }

        // Flex Direction
        if style.dirty_flags.contains(StyleFlags::FLEX_DIRECTION) {
            styles_window = styles_window.push(field_row(
                "Flex Direction: ",
                FIELD_NAME_COLOR,
                format!("{:?}", style.flex_direction()).as_str(),
                FIELD_VALUE_COLOR,
            ));
        }

        // Flex Grow
        if style.dirty_flags.contains(StyleFlags::FLEX_GROW) {
            styles_window = styles_window.push(field_row(
                "Flex Grow: ",
                FIELD_NAME_COLOR,
                style.flex_grow().to_string().as_str(),
                FIELD_VALUE_COLOR,
            ));
        }

        // Flex Shrink
        if style.dirty_flags.contains(StyleFlags::FLEX_SHRINK) {
            styles_window = styles_window.push(field_row(
                "Flex Shrink: ",
                FIELD_NAME_COLOR,
                style.flex_shrink().to_string().as_str(),
                FIELD_VALUE_COLOR,
            ));
        }

        // Flex Basis
        if style.dirty_flags.contains(StyleFlags::FLEX_BASIS) {
            styles_window = styles_window.push(field_row(
                "Flex Basis: ",
                FIELD_NAME_COLOR,
                style.flex_basis().to_string().as_str(),
                FIELD_VALUE_COLOR,
            ));
        }

        // Color
        if style.dirty_flags.contains(StyleFlags::COLOR) {
            styles_window = styles_window.push(field_row(
                "Color: ",
                FIELD_NAME_COLOR,
                format!("{:?}", style.color()).as_str(),
                FIELD_VALUE_COLOR,
            ))
        }

        // Background Color
        if style.dirty_flags.contains(StyleFlags::BACKGROUND) {
            styles_window = styles_window.push(field_row(
                "Background: ",
                FIELD_NAME_COLOR,
                format!("{:?}", style.background()).as_str(),
                FIELD_VALUE_COLOR,
            ))
        }

        // Font Size
        if style.dirty_flags.contains(StyleFlags::FONT_SIZE) {
            styles_window = styles_window.push(field_row(
                "Font Size: ",
                FIELD_NAME_COLOR,
                style.font_size().to_string().as_str(),
                FIELD_VALUE_COLOR,
            ));
        }

        // Font Weight
        if style.dirty_flags.contains(StyleFlags::FONT_WEIGHT) {
            styles_window = styles_window.push(field_row(
                "Font Weight: ",
                FIELD_NAME_COLOR,
                format!("{:?}", style.font_weight()).as_str(),
                FIELD_VALUE_COLOR,
            ))
        }

        // Font Style
        if style.dirty_flags.contains(StyleFlags::FONT_STYLE) {
            styles_window = styles_window.push(field_row(
                "Font Style: ",
                FIELD_NAME_COLOR,
                format!("{:?}", style.font_style()).as_str(),
                FIELD_VALUE_COLOR,
            ))
        }

        // Overflow
        if style.dirty_flags.contains(StyleFlags::OVERFLOW) {
            styles_window = styles_window.push(field_row(
                "Overflow: ",
                FIELD_NAME_COLOR,
                format!("{:?}", style.overflow()).as_str(),
                FIELD_VALUE_COLOR,
            ))
        }

        // Border Color
        if style.dirty_flags.contains(StyleFlags::BORDER_COLOR) {
            styles_window = styles_window.push(field_row(
                "Border Color: ",
                FIELD_NAME_COLOR,
                format!("{:?}", style.border_color()).as_str(),
                FIELD_VALUE_COLOR,
            ))
        }

        // Border Width
        if style.dirty_flags.contains(StyleFlags::BORDER_WIDTH) {
            styles_window = styles_window.push(field_row(
                "Border Width: ",
                FIELD_NAME_COLOR,
                style.border_width().map(|bw| bw.to_string()).join(", ").as_str(),
                FIELD_VALUE_COLOR,
            ));
        }

        // Border Radius
        if style.dirty_flags.contains(StyleFlags::BORDER_RADIUS) {
            styles_window = styles_window.push(field_row(
                "Border Radius: ",
                FIELD_NAME_COLOR,
                format!("{:?}", style.border_radius()).as_str(),
                FIELD_VALUE_COLOR,
            ));
        }
    }

    styles_window
}
