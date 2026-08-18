use crate::style::{AlignContent, AlignItems, AlignSelf, BoxSizing, Display, FlexDirection, FlexWrap, JustifyContent, Overflow, Position, Style, Unit};

impl From<AlignItems> for gummy::AlignItems {
    fn from(value: AlignItems) -> Self {
        match value {
            AlignItems::Normal => Self::Normal,
            AlignItems::Start => Self::Start,
            AlignItems::End => Self::End,
            AlignItems::FlexStart => Self::FlexStart,
            AlignItems::FlexEnd => Self::FlexEnd,
            AlignItems::SelfStart => Self::SelfStart,
            AlignItems::SelfEnd => Self::SelfEnd,
            AlignItems::Center => Self::Center,
            AlignItems::Baseline => Self::Baseline,
            AlignItems::Stretch => Self::Stretch,
            AlignItems::SafeStart => Self::SafeStart,
            AlignItems::SafeEnd => Self::SafeEnd,
            AlignItems::SafeFlexStart => Self::SafeFlexStart,
            AlignItems::SafeFlexEnd => Self::SafeFlexEnd,
            AlignItems::SafeSelfStart => Self::SafeSelfStart,
            AlignItems::SafeSelfEnd => Self::SafeSelfEnd,
            AlignItems::SafeCenter => Self::SafeCenter,
        }
    }
}

impl From<AlignSelf> for gummy::AlignSelf {
    fn from(value: AlignSelf) -> Self {
        match value {
            AlignSelf::Auto => Self::Auto,
            AlignSelf::Normal => Self::Normal,
            AlignSelf::Start => Self::Start,
            AlignSelf::End => Self::End,
            AlignSelf::FlexStart => Self::FlexStart,
            AlignSelf::FlexEnd => Self::FlexEnd,
            AlignSelf::SelfStart => Self::SelfStart,
            AlignSelf::SelfEnd => Self::SelfEnd,
            AlignSelf::Center => Self::Center,
            AlignSelf::Baseline => Self::Baseline,
            AlignSelf::Stretch => Self::Stretch,
            AlignSelf::SafeStart => Self::SafeStart,
            AlignSelf::SafeEnd => Self::SafeEnd,
            AlignSelf::SafeFlexStart => Self::SafeFlexStart,
            AlignSelf::SafeFlexEnd => Self::SafeFlexEnd,
            AlignSelf::SafeSelfStart => Self::SafeSelfStart,
            AlignSelf::SafeSelfEnd => Self::SafeSelfEnd,
            AlignSelf::SafeCenter => Self::SafeCenter,
        }
    }
}

impl From<AlignContent> for gummy::AlignContent {
    fn from(value: AlignContent) -> Self {
        match value {
            AlignContent::Normal => Self::Normal,
            AlignContent::Start => Self::Start,
            AlignContent::End => Self::End,
            AlignContent::FlexStart => Self::FlexStart,
            AlignContent::FlexEnd => Self::FlexEnd,
            AlignContent::Center => Self::Center,
            AlignContent::Stretch => Self::Stretch,
            AlignContent::SpaceBetween => Self::SpaceBetween,
            AlignContent::SpaceEvenly => Self::SpaceEvenly,
            AlignContent::SpaceAround => Self::SpaceAround,
            AlignContent::SafeStart => Self::SafeStart,
            AlignContent::SafeEnd => Self::SafeEnd,
            AlignContent::SafeFlexStart => Self::SafeFlexStart,
            AlignContent::SafeFlexEnd => Self::SafeFlexEnd,
            AlignContent::SafeCenter => Self::SafeCenter,
        }
    }
}

impl From<JustifyContent> for gummy::JustifyContent {
    fn from(value: JustifyContent) -> Self {
        match value {
            JustifyContent::Normal => Self::Normal,
            JustifyContent::Start => Self::Start,
            JustifyContent::End => Self::End,
            JustifyContent::FlexStart => Self::FlexStart,
            JustifyContent::FlexEnd => Self::FlexEnd,
            JustifyContent::Center => Self::Center,
            JustifyContent::Stretch => Self::Stretch,
            JustifyContent::SpaceBetween => Self::SpaceBetween,
            JustifyContent::SpaceEvenly => Self::SpaceEvenly,
            JustifyContent::SpaceAround => Self::SpaceAround,
            JustifyContent::SafeStart => Self::SafeStart,
            JustifyContent::SafeEnd => Self::SafeEnd,
            JustifyContent::SafeFlexStart => Self::SafeFlexStart,
            JustifyContent::SafeFlexEnd => Self::SafeFlexEnd,
            JustifyContent::SafeCenter => Self::SafeCenter,
        }
    }
}

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

        let align_items = style.get_align_items().into();
        let align_self = style.get_align_self().into();
        let align_content = style.get_align_content().into();
        let justify_content = style.get_justify_content().into();

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
        let order = style.get_order();

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
            align_content,
            align_items,
            align_self,
            display,
            flex_wrap,
            flex_grow,
            flex_shrink,
            flex_basis,
            order,
            overflow: gummy::Point {
                x: overflow_x,
                y: overflow_y,
            },
            border,
            ..Default::default()
        }
    }
}