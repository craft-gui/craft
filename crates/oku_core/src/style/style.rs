use crate::engine::renderer::color::Color;
use taffy::{FlexWrap};

#[derive(Clone, Copy, Debug)]
pub enum Unit {
    Px(f32),
    Percentage(f32),
    Auto,
}

pub use taffy::Position;
pub use taffy::BoxSizing;
pub use taffy::Overflow;

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

#[derive(Clone, Copy, Debug)]
pub struct Style {
    pub box_sizing: BoxSizing,
    pub scrollbar_width: f32,
    pub position: Position,
    pub margin: [f32; 4],
    pub padding: [f32; 4],
    pub border: [Unit; 4],
    pub inset: [Unit; 4],
    pub width: Unit,
    pub height: Unit,
    pub max_width: Unit,
    pub max_height: Unit,
    pub min_width: Unit,
    pub min_height: Unit,
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
    pub border_color: Color,
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

fn unit_to_taffy_lengthpercentageauto(unit: Unit) -> taffy::LengthPercentageAuto {
    match unit {
        Unit::Px(px) => taffy::LengthPercentageAuto::Length(px),
        Unit::Percentage(percentage) => taffy::LengthPercentageAuto::Percent(percentage / 100.0),
        Unit::Auto => taffy::LengthPercentageAuto::Auto,
    }
}

fn unit_to_taffy_length_percentage(unit: Unit) -> taffy::LengthPercentage {
    match unit {
        Unit::Px(px) => taffy::LengthPercentage::Length(px),
        Unit::Percentage(percentage) => taffy::LengthPercentage::Percent(percentage / 100.0),
        Unit::Auto => panic!("Auto is not a valid value for LengthPercentage"),
    }
}


impl Default for Style {
    fn default() -> Self {
        Style {
            box_sizing: BoxSizing::BorderBox,
            scrollbar_width: 15.0,
            position: Position::Relative,
            margin: [0.0; 4],
            padding: [0.0; 4],
            border: [Unit::Px(0.0); 4],
            inset: [Unit::Px(0.0); 4],
            width: Unit::Auto,
            height: Unit::Auto,
            min_width: Unit::Auto,
            min_height: Unit::Auto,
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
            color: Color::rgba(0, 0, 0, 255),
            background: Color::rgba(0, 0, 0, 0),
            border_color: Color::rgba(0, 0, 0, 255),
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

        let min_size = taffy::Size {
            width: unit_to_taffy_dimension(style.min_width),
            height: unit_to_taffy_dimension(style.min_height),
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

        let border: taffy::Rect<taffy::LengthPercentage> = taffy::Rect {
            left: unit_to_taffy_length_percentage(style.border[3]),
            right: unit_to_taffy_length_percentage(style.border[1]),
            top: unit_to_taffy_length_percentage(style.border[0]),
            bottom: unit_to_taffy_length_percentage(style.border[2]),
        };

        let inset: taffy::Rect<taffy::LengthPercentageAuto> = taffy::Rect {
            left: unit_to_taffy_lengthpercentageauto(style.inset[3]),
            right: unit_to_taffy_lengthpercentageauto(style.inset[1]),
            top: unit_to_taffy_lengthpercentageauto(style.inset[0]),
            bottom: unit_to_taffy_lengthpercentageauto(style.inset[2]),
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

        let scrollbar_width = style.scrollbar_width;

        let box_sizing = taffy::BoxSizing::BorderBox;

        taffy::Style {
            box_sizing,
            inset,
            scrollbar_width,
            position: style.position,
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
