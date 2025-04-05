use crate::style::{AlignItems, Display, FlexDirection, JustifyContent, Style, Unit, Wrap};
use taffy::{FlexWrap, Overflow};
use winit::dpi::{LogicalPosition, PhysicalPosition};

fn unit_to_taffy_dimension_with_scale_factor(unit: Unit, scale_factor: f64) -> taffy::Dimension {
    match unit {
        Unit::Px(px) => taffy::Dimension::length(
            PhysicalPosition::from_logical(LogicalPosition::new(px as f64, px as f64), scale_factor).x,
        ),
        Unit::Percentage(percentage) => taffy::Dimension::percent(percentage / 100.0),
        Unit::Auto => taffy::Dimension::auto(),
    }
}

fn unit_to_taffy_lengthpercentageauto_with_scale_factor(unit: Unit, scale_factor: f64) -> taffy::LengthPercentageAuto {
    match unit {
        Unit::Px(px) => taffy::LengthPercentageAuto::length(
            PhysicalPosition::from_logical(LogicalPosition::new(px as f64, px as f64), scale_factor).x,
        ),
        Unit::Percentage(percentage) => taffy::LengthPercentageAuto::percent(percentage / 100.0),
        Unit::Auto => taffy::LengthPercentageAuto::auto(),
    }
}

fn unit_to_taffy_length_percentage_with_scale_factor(unit: Unit, scale_factor: f64) -> taffy::LengthPercentage {
    match unit {
        Unit::Px(px) => taffy::LengthPercentage::length(
            PhysicalPosition::from_logical(LogicalPosition::new(px as f64, px as f64), scale_factor).x,
        ),
        Unit::Percentage(percentage) => taffy::LengthPercentage::percent(percentage / 100.0),
        Unit::Auto => panic!("Auto is not a valid value for LengthPercentage"),
    }
}

impl Style {
    pub fn to_taffy_style_with_scale_factor(&self, scale_factor: f64) -> taffy::Style {
        let style = self;

        let gap = taffy::Size {
            width: unit_to_taffy_length_percentage_with_scale_factor(style.gap()[0], scale_factor),
            height: unit_to_taffy_length_percentage_with_scale_factor(style.gap()[1], scale_factor),
        };

        let display = match style.display() {
            Display::Flex => taffy::Display::Flex,
            Display::Block => taffy::Display::Block,
            Display::None => taffy::Display::None,
        };

        let size = taffy::Size {
            width: unit_to_taffy_dimension_with_scale_factor(style.width(), scale_factor),
            height: unit_to_taffy_dimension_with_scale_factor(style.height(), scale_factor),
        };

        let max_size = taffy::Size {
            width: unit_to_taffy_dimension_with_scale_factor(style.max_width(), scale_factor),
            height: unit_to_taffy_dimension_with_scale_factor(style.max_height(), scale_factor),
        };

        let min_size = taffy::Size {
            width: unit_to_taffy_dimension_with_scale_factor(style.min_width(), scale_factor),
            height: unit_to_taffy_dimension_with_scale_factor(style.min_height(), scale_factor),
        };

        let margin: taffy::Rect<taffy::LengthPercentageAuto> = taffy::Rect {
            left: unit_to_taffy_lengthpercentageauto_with_scale_factor(style.margin()[3], scale_factor),
            right: unit_to_taffy_lengthpercentageauto_with_scale_factor(style.margin()[1], scale_factor),
            top: unit_to_taffy_lengthpercentageauto_with_scale_factor(style.margin()[0], scale_factor),
            bottom: unit_to_taffy_lengthpercentageauto_with_scale_factor(style.margin()[2], scale_factor),
        };

        let padding: taffy::Rect<taffy::LengthPercentage> = taffy::Rect {
            left: unit_to_taffy_length_percentage_with_scale_factor(style.padding()[3], scale_factor),
            right: unit_to_taffy_length_percentage_with_scale_factor(style.padding()[1], scale_factor),
            top: unit_to_taffy_length_percentage_with_scale_factor(style.padding()[0], scale_factor),
            bottom: unit_to_taffy_length_percentage_with_scale_factor(style.padding()[2], scale_factor),
        };

        let border: taffy::Rect<taffy::LengthPercentage> = taffy::Rect {
            left: unit_to_taffy_length_percentage_with_scale_factor(style.border_width()[3], scale_factor),
            right: unit_to_taffy_length_percentage_with_scale_factor(style.border_width()[1], scale_factor),
            top: unit_to_taffy_length_percentage_with_scale_factor(style.border_width()[0], scale_factor),
            bottom: unit_to_taffy_length_percentage_with_scale_factor(style.border_width()[2], scale_factor),
        };

        let inset: taffy::Rect<taffy::LengthPercentageAuto> = taffy::Rect {
            left: unit_to_taffy_lengthpercentageauto_with_scale_factor(style.inset()[3], scale_factor),
            right: unit_to_taffy_lengthpercentageauto_with_scale_factor(style.inset()[1], scale_factor),
            top: unit_to_taffy_lengthpercentageauto_with_scale_factor(style.inset()[0], scale_factor),
            bottom: unit_to_taffy_lengthpercentageauto_with_scale_factor(style.inset()[2], scale_factor),
        };

        let align_items = match style.align_items() {
            None => None,
            Some(AlignItems::Start) => Some(taffy::AlignItems::Start),
            Some(AlignItems::End) => Some(taffy::AlignItems::End),
            Some(AlignItems::FlexStart) => Some(taffy::AlignItems::FlexStart),
            Some(AlignItems::FlexEnd) => Some(taffy::AlignItems::FlexEnd),
            Some(AlignItems::Center) => Some(taffy::AlignItems::Center),
            Some(AlignItems::Baseline) => Some(taffy::AlignItems::Baseline),
            Some(AlignItems::Stretch) => Some(taffy::AlignItems::Stretch),
        };

        let justify_content = match style.justify_content() {
            None => None,
            Some(JustifyContent::Start) => Some(taffy::JustifyContent::Start),
            Some(JustifyContent::End) => Some(taffy::JustifyContent::End),
            Some(JustifyContent::FlexStart) => Some(taffy::JustifyContent::FlexStart),
            Some(JustifyContent::FlexEnd) => Some(taffy::JustifyContent::FlexEnd),
            Some(JustifyContent::Center) => Some(taffy::JustifyContent::Center),
            Some(JustifyContent::Stretch) => Some(taffy::JustifyContent::Stretch),
            Some(JustifyContent::SpaceBetween) => Some(taffy::JustifyContent::SpaceBetween),
            Some(JustifyContent::SpaceEvenly) => Some(taffy::JustifyContent::SpaceEvenly),
            Some(JustifyContent::SpaceAround) => Some(taffy::JustifyContent::SpaceAround),
        };

        let flex_direction = match style.flex_direction() {
            FlexDirection::Row => taffy::FlexDirection::Row,
            FlexDirection::Column => taffy::FlexDirection::Column,
            FlexDirection::RowReverse => taffy::FlexDirection::RowReverse,
            FlexDirection::ColumnReverse => taffy::FlexDirection::ColumnReverse,
        };

        let flex_wrap = match style.wrap() {
            Wrap::NoWrap => FlexWrap::NoWrap,
            Wrap::Wrap => FlexWrap::Wrap,
            Wrap::WrapReverse => FlexWrap::WrapReverse,
        };

        let flex_grow =
            PhysicalPosition::from_logical(LogicalPosition::new(style.flex_grow(), style.flex_grow()), scale_factor).x;
        let flex_shrink = PhysicalPosition::from_logical(
            LogicalPosition::new(style.flex_shrink(), style.flex_shrink()),
            scale_factor,
        )
        .x;
        let flex_basis: taffy::Dimension = match style.flex_basis() {
            Unit::Px(px) => {
                taffy::Dimension::length(PhysicalPosition::from_logical(LogicalPosition::new(px, px), scale_factor).x)
            }
            Unit::Percentage(percentage) => taffy::Dimension::percent(percentage / 100.0),
            Unit::Auto => taffy::Dimension::auto(),
        };

        fn overflow_to_taffy_overflow(overflow: Overflow) -> Overflow {
            overflow
        }

        let overflow_x = overflow_to_taffy_overflow(style.overflow()[0]);
        let overflow_y = overflow_to_taffy_overflow(style.overflow()[1]);

        let scrollbar_width = PhysicalPosition::from_logical(
            LogicalPosition::new(style.scrollbar_width(), style.scrollbar_width()),
            scale_factor,
        )
        .x;

        let box_sizing = taffy::BoxSizing::BorderBox;

        taffy::Style {
            gap,
            box_sizing,
            inset,
            scrollbar_width,
            position: style.position(),
            size,
            min_size,
            max_size,
            flex_direction,
            margin,
            padding,
            justify_content,
            align_items,
            display,
            flex_wrap,
            flex_grow,
            flex_shrink,
            flex_basis,
            overflow: taffy::Point {
                x: overflow_x,
                y: overflow_y,
            },
            border,
            ..Default::default()
        }
    }
}
