use taffy::{FlexWrap, Overflow};

use crate::animations::Animation;
use crate::style::{AlignItems, Display, FlexDirection, JustifyContent, Style, Unit, Wrap};

fn unit_to_taffy_dimension(unit: Unit) -> taffy::Dimension {
    match unit {
        Unit::Px(px) => taffy::Dimension::length(px),
        Unit::Percentage(percentage) => taffy::Dimension::percent(percentage / 100.0),
        Unit::Auto => taffy::Dimension::auto(),
    }
}

fn unit_to_taffy_lengthpercentageauto(unit: Unit) -> taffy::LengthPercentageAuto {
    match unit {
        Unit::Px(px) => taffy::LengthPercentageAuto::length(px),
        Unit::Percentage(percentage) => taffy::LengthPercentageAuto::percent(percentage / 100.0),
        Unit::Auto => taffy::LengthPercentageAuto::auto(),
    }
}

fn unit_to_taffy_length_percentage(unit: Unit) -> taffy::LengthPercentage {
    match unit {
        Unit::Px(px) => taffy::LengthPercentage::length(px),
        Unit::Percentage(percentage) => taffy::LengthPercentage::percent(percentage / 100.0),
        Unit::Auto => panic!("Auto is not a valid value for LengthPercentage"),
    }
}

impl Style {
    pub fn animation(&self, animation: &str) -> Option<&Animation> {
        self.animations.iter().find(|ani| ani.name == animation)
    }

    pub fn to_taffy_style(&self) -> taffy::Style {
        let style = self;

        let gap = taffy::Size {
            width: unit_to_taffy_length_percentage(style.gap()[0]),
            height: unit_to_taffy_length_percentage(style.gap()[1]),
        };

        let display = match style.display() {
            Display::Flex => taffy::Display::Flex,
            Display::Block => taffy::Display::Block,
            Display::None => taffy::Display::None,
        };

        let size = taffy::Size {
            width: unit_to_taffy_dimension(style.width()),
            height: unit_to_taffy_dimension(style.height()),
        };

        let max_size = taffy::Size {
            width: unit_to_taffy_dimension(style.max_width()),
            height: unit_to_taffy_dimension(style.max_height()),
        };

        let min_size = taffy::Size {
            width: unit_to_taffy_dimension(style.min_width()),
            height: unit_to_taffy_dimension(style.min_height()),
        };

        let margin: taffy::Rect<taffy::LengthPercentageAuto> = taffy::Rect {
            left: unit_to_taffy_lengthpercentageauto(style.margin().left),
            right: unit_to_taffy_lengthpercentageauto(style.margin().right),
            top: unit_to_taffy_lengthpercentageauto(style.margin().top),
            bottom: unit_to_taffy_lengthpercentageauto(style.margin().bottom),
        };

        let padding: taffy::Rect<taffy::LengthPercentage> = taffy::Rect {
            left: unit_to_taffy_length_percentage(style.padding().left),
            right: unit_to_taffy_length_percentage(style.padding().right),
            top: unit_to_taffy_length_percentage(style.padding().top),
            bottom: unit_to_taffy_length_percentage(style.padding().bottom),
        };

        let border: taffy::Rect<taffy::LengthPercentage> = taffy::Rect {
            left: unit_to_taffy_length_percentage(style.border_width().left),
            right: unit_to_taffy_length_percentage(style.border_width().right),
            top: unit_to_taffy_length_percentage(style.border_width().top),
            bottom: unit_to_taffy_length_percentage(style.border_width().bottom),
        };

        let inset: taffy::Rect<taffy::LengthPercentageAuto> = taffy::Rect {
            left: unit_to_taffy_lengthpercentageauto(style.inset().left),
            right: unit_to_taffy_lengthpercentageauto(style.inset().right),
            top: unit_to_taffy_lengthpercentageauto(style.inset().top),
            bottom: unit_to_taffy_lengthpercentageauto(style.inset().bottom),
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

        let flex_grow = style.flex_grow();
        let flex_shrink = style.flex_shrink();
        let flex_basis: taffy::Dimension = unit_to_taffy_dimension(style.flex_basis());

        fn overflow_to_taffy_overflow(overflow: Overflow) -> Overflow {
            overflow
        }

        let overflow_x = overflow_to_taffy_overflow(style.overflow()[0]);
        let overflow_y = overflow_to_taffy_overflow(style.overflow()[1]);

        let scrollbar_width = style.scrollbar_width();

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
