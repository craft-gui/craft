use crate::renderer::color::Color;
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
use winit::dpi::{LogicalPosition, PhysicalPosition, PhysicalSize};

impl Unit {
    pub fn is_auto(&self) -> bool {
        matches!(self, Unit::Auto)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Display {
    Flex,
    Block,
    None
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
    pub margin: [Unit; 4],
    pub padding: [Unit; 4],
    pub border: [Unit; 4],
    pub gap: [Unit; 2],
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

fn unit_to_taffy_dimension_with_scale_factor(unit: Unit, scale_factor: f64) -> taffy::Dimension {
    match unit {
        Unit::Px(px) => taffy::Dimension::Length(PhysicalPosition::from_logical(LogicalPosition::new(px as f64, px as f64), scale_factor).x),
        Unit::Percentage(percentage) => taffy::Dimension::Percent(percentage / 100.0),
        Unit::Auto => taffy::Dimension::Auto,
    }
}

fn unit_to_taffy_lengthpercentageauto_with_scale_factor(unit: Unit, scale_factor: f64) -> taffy::LengthPercentageAuto {
    match unit {
        Unit::Px(px) => taffy::LengthPercentageAuto::Length(PhysicalPosition::from_logical(LogicalPosition::new(px as f64, px as f64), scale_factor).x),
        Unit::Percentage(percentage) => taffy::LengthPercentageAuto::Percent(percentage / 100.0),
        Unit::Auto => taffy::LengthPercentageAuto::Auto,
    }
}

fn unit_to_taffy_length_percentage_with_scale_factor(unit: Unit, scale_factor: f64) -> taffy::LengthPercentage {
    match unit {
        Unit::Px(px) => taffy::LengthPercentage::Length(PhysicalPosition::from_logical(LogicalPosition::new(px as f64, px as f64), scale_factor).x),
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
            margin: [Unit::Px(0.0); 4],
            padding: [Unit::Px(0.0); 4],
            border: [Unit::Px(0.0); 4],
            gap: [Unit::Px(0.0); 2],
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

impl Style {
    pub fn to_taffy_style_with_scale_factor(&self, scale_factor: f64) -> taffy::Style {
        let style = self;

        let gap = taffy::Size {
            width: unit_to_taffy_length_percentage_with_scale_factor(style.gap[0], scale_factor),
            height: unit_to_taffy_length_percentage_with_scale_factor(style.gap[1], scale_factor),
        };

        let display = match style.display {
            Display::Flex => taffy::Display::Flex,
            Display::Block => taffy::Display::Block,
            Display::None => taffy::Display::None,
        };

        let size = taffy::Size {
            width: unit_to_taffy_dimension_with_scale_factor(style.width, scale_factor),
            height: unit_to_taffy_dimension_with_scale_factor(style.height, scale_factor),
        };

        let max_size = taffy::Size {
            width: unit_to_taffy_dimension_with_scale_factor(style.max_width, scale_factor),
            height: unit_to_taffy_dimension_with_scale_factor(style.max_height, scale_factor),
        };

        let min_size = taffy::Size {
            width: unit_to_taffy_dimension_with_scale_factor(style.min_width, scale_factor),
            height: unit_to_taffy_dimension_with_scale_factor(style.min_height, scale_factor),
        };

        let margin: taffy::Rect<taffy::LengthPercentageAuto> = taffy::Rect {
            left: unit_to_taffy_lengthpercentageauto_with_scale_factor(style.margin[3], scale_factor),
            right: unit_to_taffy_lengthpercentageauto_with_scale_factor(style.margin[1], scale_factor),
            top: unit_to_taffy_lengthpercentageauto_with_scale_factor(style.margin[0], scale_factor),
            bottom: unit_to_taffy_lengthpercentageauto_with_scale_factor(style.margin[2], scale_factor),
        };

        let padding: taffy::Rect<taffy::LengthPercentage> = taffy::Rect {
            left: unit_to_taffy_length_percentage_with_scale_factor(style.padding[3], scale_factor),
            right: unit_to_taffy_length_percentage_with_scale_factor(style.padding[1], scale_factor),
            top: unit_to_taffy_length_percentage_with_scale_factor(style.padding[0], scale_factor),
            bottom: unit_to_taffy_length_percentage_with_scale_factor(style.padding[2], scale_factor),
        };

        let border: taffy::Rect<taffy::LengthPercentage> = taffy::Rect {
            left: unit_to_taffy_length_percentage_with_scale_factor(style.border[3], scale_factor),
            right: unit_to_taffy_length_percentage_with_scale_factor(style.border[1], scale_factor),
            top: unit_to_taffy_length_percentage_with_scale_factor(style.border[0], scale_factor),
            bottom: unit_to_taffy_length_percentage_with_scale_factor(style.border[2], scale_factor),
        };

        let inset: taffy::Rect<taffy::LengthPercentageAuto> = taffy::Rect {
            left: unit_to_taffy_lengthpercentageauto_with_scale_factor(style.inset[3], scale_factor),
            right: unit_to_taffy_lengthpercentageauto_with_scale_factor(style.inset[1], scale_factor),
            top: unit_to_taffy_lengthpercentageauto_with_scale_factor(style.inset[0], scale_factor),
            bottom: unit_to_taffy_lengthpercentageauto_with_scale_factor(style.inset[2], scale_factor),
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

        let flex_grow = PhysicalPosition::from_logical(LogicalPosition::new(style.flex_grow, style.flex_grow), scale_factor).x;
        let flex_shrink = PhysicalPosition::from_logical(LogicalPosition::new(style.flex_shrink, style.flex_shrink), scale_factor).x;
        let flex_basis: taffy::Dimension = match style.flex_basis {
            Unit::Px(px) => taffy::Dimension::Length(PhysicalPosition::from_logical(LogicalPosition::new(px, px), scale_factor).x),
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
        
        let scrollbar_width = PhysicalPosition::from_logical(LogicalPosition::new(style.scrollbar_width, style.scrollbar_width), scale_factor).x;

        let box_sizing = taffy::BoxSizing::BorderBox;

        taffy::Style {
            gap,
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
