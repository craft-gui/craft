use crate::engine::renderer::color::Color;
use taffy::FlexWrap;

#[derive(Clone, Copy, Debug)]
pub enum Unit {
    Px(f32),
    Percentage(f32),
    Auto,
}

impl Unit {
    pub fn is_auto(&self) -> bool {
        matches!(self, Unit::Auto)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Display {
    Flex,
    Block,
    Grid,
}

#[derive(Clone, Copy, Debug)]
pub enum AlignItems {
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
}

#[derive(Clone, Copy, Debug)]
pub enum AlignContent {
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
    SpaceBetween,
    SpaceEvenly,
    SpaceAround,
}

pub type JustifyContent = AlignContent;

#[derive(Clone, Copy, Debug)]
pub enum FlexDirection {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Hash)]
pub struct Weight(pub u16);

impl Weight {
    /// Thin weight (100), the thinnest value.
    pub const THIN: Weight = Weight(100);

    /// Extra light weight (200).
    pub const EXTRA_LIGHT: Weight = Weight(200);

    /// Light weight (300).
    pub const LIGHT: Weight = Weight(300);

    /// Normal (400).
    pub const NORMAL: Weight = Weight(400);

    /// Medium weight (500, higher than normal).
    pub const MEDIUM: Weight = Weight(500);

    /// Semibold weight (600).
    pub const SEMIBOLD: Weight = Weight(600);

    /// Bold weight (700).
    pub const BOLD: Weight = Weight(700);

    /// Extra-bold weight (800).
    pub const EXTRA_BOLD: Weight = Weight(800);

    /// Black weight (900), the thickest value.
    pub const BLACK: Weight = Weight(900);
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Wrap {
    /// Items will not wrap and stay on a single line
    NoWrap,
    /// Items will wrap according to this item's [`taffy::FlexDirection`]
    Wrap,
    /// Items will wrap in the opposite direction to this item's [`taffy::FlexDirection`]
    WrapReverse,
}

impl Default for Wrap {
    fn default() -> Self {
        Self::NoWrap
    }
}

impl Default for Weight {
    #[inline]
    fn default() -> Weight {
        Weight::NORMAL
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

impl Default for FontStyle {
    #[inline]
    fn default() -> FontStyle {
        FontStyle::Normal
    }
}

/// How children overflowing their container should affect layout
///
/// In CSS the primary effect of this property is to control whether contents of a parent container that overflow that container should
/// be displayed anyway, be clipped, or trigger the container to become a scroll container. However it also has secondary effects on layout,
/// the main ones being:
///
///   - The automatic minimum size Flexbox/CSS Grid items with non-`Visible` overflow is `0` rather than being content based
///   - `Overflow::Scroll` nodes have space in the layout reserved for a scrollbar (width controlled by the `scrollbar_width` property)
///
/// In Taffy, we only implement the layout related secondary effects as we are not concerned with drawing/painting. The amount of space reserved for
/// a scrollbar is controlled by the `scrollbar_width` property. If this is `0` then `Scroll` behaves identically to `Hidden`.
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/overflow>
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum Overflow {
    /// The automatic minimum size of this node as a flexbox/grid item should be based on the size of its content.
    /// Content that overflows this node *should* contribute to the scroll region of its parent.
    #[default]
    Visible,
    /// The automatic minimum size of this node as a flexbox/grid item should be based on the size of its content.
    /// Content that overflows this node should *not* contribute to the scroll region of its parent.
    Clip,
    /// The automatic minimum size of this node as a flexbox/grid item should be `0`.
    /// Content that overflows this node should *not* contribute to the scroll region of its parent.
    Hidden,
    /// The automatic minimum size of this node as a flexbox/grid item should be `0`. Additionally, space should be reserved
    /// for a scrollbar. The amount of space reserved is controlled by the `scrollbar_width` property.
    /// Content that overflows this node should *not* contribute to the scroll region of its parent.
    Scroll,
}

impl Overflow {
    /// Returns true for overflow modes that contain their contents (`Overflow::Hidden`, `Overflow::Scroll`, `Overflow::Auto`)
    /// or else false for overflow modes that allow their contains to spill (`Overflow::Visible`).
    #[inline(always)]
    pub(crate) fn is_scroll_container(self) -> bool {
        match self {
            Self::Visible | Self::Clip => false,
            Self::Hidden | Self::Scroll => true,
        }
    }

    /// Returns `Some(0.0)` if the overflow mode would cause the automatic minimum size of a Flexbox or CSS Grid item
    /// to be `0`. Else returns None.
    #[inline(always)]
    pub(crate) fn maybe_into_automatic_min_size(self) -> Option<f32> {
        match self.is_scroll_container() {
            true => Some(0.0),
            false => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Style {
    pub margin: [f32; 4],
    pub padding: [f32; 4],
    pub width: Unit,
    pub height: Unit,
    pub max_width: Unit,
    pub max_height: Unit,
    pub x: f32,
    pub y: f32,
    pub display: Display,
    pub wrap: Wrap,
    pub align_items: Option<AlignItems>,
    pub justify_content: Option<JustifyContent>,
    pub flex_direction: FlexDirection,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: Unit,

    pub color: Color,
    pub background: Color,
    pub font_size: f32,
    pub font_weight: Weight,
    pub font_style: FontStyle,
    pub overflow: [Overflow; 2],
}

fn unit_to_taffy_dimension(unit: Unit) -> taffy::Dimension {
    match unit {
        Unit::Px(px) => taffy::Dimension::Length(px),
        Unit::Percentage(percentage) => taffy::Dimension::Percent(percentage / 100.0),
        Unit::Auto => taffy::Dimension::Auto,
    }
}

impl Default for Style {
    fn default() -> Self {
        Style {
            margin: [0.0; 4],
            padding: [0.0; 4],
            width: Unit::Auto,
            height: Unit::Auto,
            max_width: Unit::Auto,
            max_height: Unit::Auto,
            x: 0.0,
            y: 0.0,
            display: Display::Flex,
            wrap: Default::default(),
            align_items: None,
            justify_content: None,
            flex_direction: FlexDirection::Row,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: Unit::Auto,
            color: Color::new_from_rgba_u8(0, 0, 0, 255),
            background: Color::new_from_rgba_u8(0, 0, 0, 0),
            font_size: 16.0,
            font_weight: Default::default(),
            font_style: Default::default(),
            overflow: [Overflow::default(), Overflow::default()],
        }
    }
}

impl From<Style> for taffy::Style {
    fn from(style: Style) -> Self {
        let display = match style.display {
            Display::Flex => taffy::Display::Flex,
            Display::Block => taffy::Display::Block,
            Display::Grid => taffy::Display::Grid,
        };

        let size = taffy::Size {
            width: unit_to_taffy_dimension(style.width),
            height: unit_to_taffy_dimension(style.height),
        };

        let max_size = taffy::Size {
            width: unit_to_taffy_dimension(style.max_width),
            height: unit_to_taffy_dimension(style.max_height),
        };

        let margin: taffy::Rect<taffy::LengthPercentageAuto> = taffy::Rect {
            left: taffy::LengthPercentageAuto::Length(style.margin[3]),
            right: taffy::LengthPercentageAuto::Length(style.margin[1]),
            top: taffy::LengthPercentageAuto::Length(style.margin[0]),
            bottom: taffy::LengthPercentageAuto::Length(style.margin[2]),
        };

        let padding: taffy::Rect<taffy::LengthPercentage> = taffy::Rect {
            left: taffy::LengthPercentage::Length(style.padding[3]),
            right: taffy::LengthPercentage::Length(style.padding[1]),
            top: taffy::LengthPercentage::Length(style.padding[0]),
            bottom: taffy::LengthPercentage::Length(style.padding[2]),
        };

        let align_items = match style.align_items {
            None => None,
            Some(AlignItems::Start) => Some(taffy::AlignItems::Start),
            Some(AlignItems::End) => Some(taffy::AlignItems::End),
            Some(AlignItems::FlexStart) => Some(taffy::AlignItems::FlexStart),
            Some(AlignItems::FlexEnd) => Some(taffy::AlignItems::FlexEnd),
            Some(AlignItems::Center) => Some(taffy::AlignItems::Center),
            Some(AlignItems::Baseline) => Some(taffy::AlignItems::Baseline),
            Some(AlignItems::Stretch) => Some(taffy::AlignItems::Stretch),
        };

        let justify_content = match style.justify_content {
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

        let flex_direction = match style.flex_direction {
            FlexDirection::Row => taffy::FlexDirection::Row,
            FlexDirection::Column => taffy::FlexDirection::Column,
            FlexDirection::RowReverse => taffy::FlexDirection::RowReverse,
            FlexDirection::ColumnReverse => taffy::FlexDirection::ColumnReverse,
        };

        let flex_wrap = match style.wrap {
            Wrap::NoWrap => FlexWrap::NoWrap,
            Wrap::Wrap => FlexWrap::Wrap,
            Wrap::WrapReverse => FlexWrap::WrapReverse,
        };

        let flex_grow = style.flex_grow;
        let flex_shrink = style.flex_shrink;
        let flex_basis: taffy::Dimension = match style.flex_basis {
            Unit::Px(px) => taffy::Dimension::Length(px),
            Unit::Percentage(percentage) => taffy::Dimension::Percent(percentage / 100.0),
            Unit::Auto => taffy::Dimension::Auto,
        };

        fn overflow_to_taffy_overflow(overflow: Overflow) -> taffy::Overflow {
            match overflow {
                Overflow::Visible => taffy::Overflow::Visible,
                Overflow::Clip => taffy::Overflow::Clip,
                Overflow::Hidden => taffy::Overflow::Hidden,
                Overflow::Scroll => taffy::Overflow::Scroll,
            }
        }

        let overflow_x = overflow_to_taffy_overflow(style.overflow[0]);
        let overflow_y = overflow_to_taffy_overflow(style.overflow[1]);

        taffy::Style {
            size,
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
            ..Default::default()
        }
    }
}
