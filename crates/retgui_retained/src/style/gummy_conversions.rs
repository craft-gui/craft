use crate::style::{AlignItems, BoxSizing, Display, FlexDirection, FlexWrap, JustifyContent, Overflow, Position, Style, Unit};

fn unit_to_gummy_dimension(unit: Unit) -> gummy::Dimension {
    match unit {
        Unit::Px(px) => gummy::Dimension::length(px),
        Unit::Percentage(percentage) => gummy::Dimension::percent(percentage / 100.0),
        Unit::Auto => gummy::Dimension::auto(),
    }
}

fn unit_to_gummy_lengthpercentageauto(unit: Unit) -> gummy::LengthPercentageAuto {
    match unit {
        Unit::Px(px) => gummy::LengthPercentageAuto::length(px),
        Unit::Percentage(percentage) => gummy::LengthPercentageAuto::percent(percentage / 100.0),
        Unit::Auto => gummy::LengthPercentageAuto::auto(),
    }
}

fn unit_to_gummy_length_percentage(unit: Unit) -> gummy::LengthPercentage {
    match unit {
        Unit::Px(px) => gummy::LengthPercentage::length(px),
        Unit::Percentage(percentage) => gummy::LengthPercentage::percent(percentage / 100.0),
        Unit::Auto => panic!("Auto is not a valid value for LengthPercentage"),
    }
}

impl Style {
    pub fn to_gummy_style(&self) -> gummy::Style {
        let style = self;

        let gap = gummy::Size {
            width: unit_to_gummy_length_percentage(style.get_gap()[0]),
            height: unit_to_gummy_length_percentage(style.get_gap()[1]),
        };

        let display = match style.get_display() {
            Display::Flex => gummy::Display::Flex,
            Display::Block => gummy::Display::Block,
            Display::None => gummy::Display::None,
        };

        let size = gummy::Size {
            width: unit_to_gummy_dimension(style.get_width()),
            height: unit_to_gummy_dimension(style.get_height()),
        };

        let max_size = gummy::Size {
            width: unit_to_gummy_dimension(style.get_max_width()),
            height: unit_to_gummy_dimension(style.get_max_height()),
        };

        let min_size = gummy::Size {
            width: unit_to_gummy_dimension(style.get_min_width()),
            height: unit_to_gummy_dimension(style.get_min_height()),
        };

        let margin: gummy::Rect<gummy::LengthPercentageAuto> = gummy::Rect {
            top: unit_to_gummy_lengthpercentageauto(style.get_margin().top),
            right: unit_to_gummy_lengthpercentageauto(style.get_margin().right),
            bottom: unit_to_gummy_lengthpercentageauto(style.get_margin().bottom),
            left: unit_to_gummy_lengthpercentageauto(style.get_margin().left),
        };

        let padding: gummy::Rect<gummy::LengthPercentage> = gummy::Rect {
            top: unit_to_gummy_length_percentage(style.get_padding().top),
            right: unit_to_gummy_length_percentage(style.get_padding().right),
            bottom: unit_to_gummy_length_percentage(style.get_padding().bottom),
            left: unit_to_gummy_length_percentage(style.get_padding().left),
        };

        let border: gummy::Rect<gummy::LengthPercentage> = gummy::Rect {
            top: unit_to_gummy_length_percentage(style.get_border_width().top),
            right: unit_to_gummy_length_percentage(style.get_border_width().right),
            bottom: unit_to_gummy_length_percentage(style.get_border_width().bottom),
            left: unit_to_gummy_length_percentage(style.get_border_width().left),
        };

        let inset: gummy::Rect<gummy::LengthPercentageAuto> = gummy::Rect {
            top: unit_to_gummy_lengthpercentageauto(style.get_inset().top),
            right: unit_to_gummy_lengthpercentageauto(style.get_inset().right),
            bottom: unit_to_gummy_lengthpercentageauto(style.get_inset().bottom),
            left: unit_to_gummy_lengthpercentageauto(style.get_inset().left),
        };

        let align_items = match style.get_align_items() {
            None => None,
            Some(AlignItems::Start) => Some(gummy::AlignItems::START),
            Some(AlignItems::End) => Some(gummy::AlignItems::END),
            Some(AlignItems::FlexStart) => Some(gummy::AlignItems::FLEX_START),
            Some(AlignItems::FlexEnd) => Some(gummy::AlignItems::FLEX_END),
            Some(AlignItems::Center) => Some(gummy::AlignItems::CENTER),
            Some(AlignItems::Baseline) => Some(gummy::AlignItems::BASELINE),
            Some(AlignItems::Stretch) => Some(gummy::AlignItems::STRETCH),
        };

        let justify_content = match style.get_justify_content() {
            None => None,
            Some(JustifyContent::Start) => Some(gummy::JustifyContent::START),
            Some(JustifyContent::End) => Some(gummy::JustifyContent::END),
            Some(JustifyContent::FlexStart) => Some(gummy::JustifyContent::FLEX_START),
            Some(JustifyContent::FlexEnd) => Some(gummy::JustifyContent::FLEX_END),
            Some(JustifyContent::Center) => Some(gummy::JustifyContent::CENTER),
            Some(JustifyContent::Stretch) => Some(gummy::JustifyContent::STRETCH),
            Some(JustifyContent::SpaceBetween) => Some(gummy::JustifyContent::SPACE_BETWEEN),
            Some(JustifyContent::SpaceEvenly) => Some(gummy::JustifyContent::SPACE_EVENLY),
            Some(JustifyContent::SpaceAround) => Some(gummy::JustifyContent::SPACE_AROUND),
        };

        let flex_direction = match style.get_flex_direction() {
            FlexDirection::Row => gummy::FlexDirection::Row,
            FlexDirection::Column => gummy::FlexDirection::Column,
            FlexDirection::RowReverse => gummy::FlexDirection::RowReverse,
            FlexDirection::ColumnReverse => gummy::FlexDirection::ColumnReverse,
        };

        let flex_wrap = match style.get_wrap() {
            FlexWrap::NoWrap => gummy::FlexWrap::NoWrap,
            FlexWrap::Wrap => gummy::FlexWrap::Wrap,
            FlexWrap::WrapReverse => gummy::FlexWrap::WrapReverse,
        };

        let flex_grow = style.get_flex_grow();
        let flex_shrink = style.get_flex_shrink();
        let flex_basis: gummy::Dimension = unit_to_gummy_dimension(style.get_flex_basis());

        fn overflow_to_gummy_overflow(overflow: Overflow) -> gummy::Overflow {
            match overflow {
                Overflow::Visible => gummy::Overflow::Visible,
                Overflow::Clip => gummy::Overflow::Clip,
                Overflow::Hidden => gummy::Overflow::Hidden,
                Overflow::Scroll => gummy::Overflow::Scroll,
            }
        }

        let overflow_x = overflow_to_gummy_overflow(style.get_overflow()[0]);
        let overflow_y = overflow_to_gummy_overflow(style.get_overflow()[1]);

        let scrollbar_width = style.get_scrollbar_width();
        let box_sizing = match style.get_box_sizing() {
            BoxSizing::BorderBox => gummy::BoxSizing::BorderBox,
            BoxSizing::ContentBox => gummy::BoxSizing::ContentBox,
        };

        let position = match style.get_position() {
            Position::Relative => gummy::Position::Relative,
            Position::Absolute => gummy::Position::Absolute,
        };

        gummy::Style {
            gap,
            box_sizing,
            inset,
            scrollbar_width,
            position,
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
            overflow: gummy::Point {
                x: overflow_x,
                y: overflow_y,
            },
            border,
            ..Default::default()
        }
    }
}
