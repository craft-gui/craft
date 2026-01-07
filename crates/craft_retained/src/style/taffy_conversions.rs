use crate::style::{AlignItems, Display, FlexDirection, JustifyContent, Style, Unit, FlexWrap, Overflow, BoxSizing, Position};

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

    pub fn to_taffy_style(&self) -> taffy::Style {
        let style = self;

        let gap = taffy::Size {
            width: unit_to_taffy_length_percentage(style.get_gap()[0]),
            height: unit_to_taffy_length_percentage(style.get_gap()[1]),
        };

        let display = match style.get_display() {
            Display::Flex => taffy::Display::Flex,
            Display::Block => taffy::Display::Block,
            Display::None => taffy::Display::None,
        };

        let size = taffy::Size {
            width: unit_to_taffy_dimension(style.get_width()),
            height: unit_to_taffy_dimension(style.get_height()),
        };

        let max_size = taffy::Size {
            width: unit_to_taffy_dimension(style.get_max_width()),
            height: unit_to_taffy_dimension(style.get_max_height()),
        };

        let min_size = taffy::Size {
            width: unit_to_taffy_dimension(style.get_min_width()),
            height: unit_to_taffy_dimension(style.get_min_height()),
        };

        let margin: taffy::Rect<taffy::LengthPercentageAuto> = taffy::Rect {
            left: unit_to_taffy_lengthpercentageauto(style.get_margin().left),
            right: unit_to_taffy_lengthpercentageauto(style.get_margin().right),
            top: unit_to_taffy_lengthpercentageauto(style.get_margin().top),
            bottom: unit_to_taffy_lengthpercentageauto(style.get_margin().bottom),
        };

        let padding: taffy::Rect<taffy::LengthPercentage> = taffy::Rect {
            left: unit_to_taffy_length_percentage(style.get_padding().left),
            right: unit_to_taffy_length_percentage(style.get_padding().right),
            top: unit_to_taffy_length_percentage(style.get_padding().top),
            bottom: unit_to_taffy_length_percentage(style.get_padding().bottom),
        };

        let border: taffy::Rect<taffy::LengthPercentage> = taffy::Rect {
            left: unit_to_taffy_length_percentage(style.get_border_width().left),
            right: unit_to_taffy_length_percentage(style.get_border_width().right),
            top: unit_to_taffy_length_percentage(style.get_border_width().top),
            bottom: unit_to_taffy_length_percentage(style.get_border_width().bottom),
        };

        let inset: taffy::Rect<taffy::LengthPercentageAuto> = taffy::Rect {
            left: unit_to_taffy_lengthpercentageauto(style.get_inset().left),
            right: unit_to_taffy_lengthpercentageauto(style.get_inset().right),
            top: unit_to_taffy_lengthpercentageauto(style.get_inset().top),
            bottom: unit_to_taffy_lengthpercentageauto(style.get_inset().bottom),
        };

        let align_items = match style.get_align_items() {
            None => None,
            Some(AlignItems::Start) => Some(taffy::AlignItems::Start),
            Some(AlignItems::End) => Some(taffy::AlignItems::End),
            Some(AlignItems::FlexStart) => Some(taffy::AlignItems::FlexStart),
            Some(AlignItems::FlexEnd) => Some(taffy::AlignItems::FlexEnd),
            Some(AlignItems::Center) => Some(taffy::AlignItems::Center),
            Some(AlignItems::Baseline) => Some(taffy::AlignItems::Baseline),
            Some(AlignItems::Stretch) => Some(taffy::AlignItems::Stretch),
        };

        let justify_content = match style.get_justify_content() {
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

        let flex_direction = match style.get_flex_direction() {
            FlexDirection::Row => taffy::FlexDirection::Row,
            FlexDirection::Column => taffy::FlexDirection::Column,
            FlexDirection::RowReverse => taffy::FlexDirection::RowReverse,
            FlexDirection::ColumnReverse => taffy::FlexDirection::ColumnReverse,
        };

        let flex_wrap = match style.get_wrap() {
            FlexWrap::NoWrap => taffy::FlexWrap::NoWrap,
            FlexWrap::Wrap => taffy::FlexWrap::Wrap,
            FlexWrap::WrapReverse => taffy::FlexWrap::WrapReverse,
        };

        let flex_grow = style.get_flex_grow();
        let flex_shrink = style.get_flex_shrink();
        let flex_basis: taffy::Dimension = unit_to_taffy_dimension(style.get_flex_basis());

        fn overflow_to_taffy_overflow(overflow: Overflow) -> taffy::Overflow {
            match overflow {
                Overflow::Visible => taffy::Overflow::Visible,
                Overflow::Clip => taffy::Overflow::Clip,
                Overflow::Hidden => taffy::Overflow::Hidden,
                Overflow::Scroll => taffy::Overflow::Scroll,
            }
        }

        let overflow_x = overflow_to_taffy_overflow(style.get_overflow()[0]);
        let overflow_y = overflow_to_taffy_overflow(style.get_overflow()[1]);

        let scrollbar_width = style.get_scrollbar_width();
        let box_sizing = match style.get_box_sizing() {
            BoxSizing::BorderBox => taffy::BoxSizing::BorderBox,
            BoxSizing::ContentBox => taffy::BoxSizing::ContentBox,
        };

        let position = match style.get_position() {
            Position::Relative => taffy::Position::Relative,
            Position::Absolute => taffy::Position::Absolute
        };

        taffy::Style {
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
            overflow: taffy::Point {
                x: overflow_x,
                y: overflow_y,
            },
            border,
            ..Default::default()
        }
    }
}
