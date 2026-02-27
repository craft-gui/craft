mod styles;
mod taffy_conversions;

use std::borrow::Cow;
use std::fmt;
use std::fmt::Debug;

use craft_primitives::ColorBrush;
use peniko::Color;
pub use styles::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Unit {
    Px(f32),
    Percentage(f32),
    Auto,
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Unit::Px(value) => write!(f, "{value}px"),
            Unit::Percentage(value) => write!(f, "{value}%"),
            Unit::Auto => write!(f, "auto"),
        }
    }
}

impl Unit {
    pub fn is_auto(&self) -> bool {
        matches!(self, Unit::Auto)
    }

    /// Gets the raw unit value. The backing f32 or 0 if self == Unit::Auto
    pub fn raw_value(&self) -> f32 {
        match self {
            Unit::Px(px) => *px,
            Unit::Percentage(pct) => *pct,
            Unit::Auto => 0.0
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Display {
    Flex,
    Block,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq)]
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

#[derive(Clone, Copy, Debug)]
pub enum JustifyContent {
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

#[derive(Clone, Copy, Debug)]
pub enum FlexDirection {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Hash)]
pub struct FontWeight(pub u16);

#[derive(Clone, Copy, Debug)]
pub struct ScrollbarColor {
    pub thumb_color: Color,
    pub track_color: Color,
}

impl FontWeight {
    /// Black weight (900), the thickest value.
    pub const BLACK: FontWeight = FontWeight(900);
    /// Bold weight (700).
    pub const BOLD: FontWeight = FontWeight(700);
    /// Extra-bold weight (800).
    pub const EXTRA_BOLD: FontWeight = FontWeight(800);
    /// Extra light weight (200).
    pub const EXTRA_LIGHT: FontWeight = FontWeight(200);
    /// Light weight (300).
    pub const LIGHT: FontWeight = FontWeight(300);
    /// Medium weight (500, higher than normal).
    pub const MEDIUM: FontWeight = FontWeight(500);
    /// Normal (400).
    pub const NORMAL: FontWeight = FontWeight(400);
    /// Semibold weight (600).
    pub const SEMIBOLD: FontWeight = FontWeight(600);
    /// Thin weight (100), the thinnest value.
    pub const THIN: FontWeight = FontWeight(100);
}

#[derive(Default, Copy, Clone, PartialEq, Eq, Debug)]
pub enum FlexWrap {
    #[default]
    NoWrap,
    Wrap,
    WrapReverse,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum Overflow {
    #[default]
    Visible,
    Clip,
    Hidden,
    Scroll,
}

impl Default for FontWeight {
    #[inline]
    fn default() -> FontWeight {
        FontWeight::NORMAL
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Underline {
    pub thickness: Option<f32>,
    pub color: Color,
    pub offset: Option<f32>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
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

#[derive(Clone, PartialEq)]
pub enum TextStyleProperty {
    Color(Color),
    FontFamily(String),
    FontSize(f32),
    FontWeight(FontWeight),
    FontStyle(FontStyle),
    UnderlineOffset(f32),
    Underline(bool),
    UnderlineSize(f32),
    UnderlineBrush(Color),
    Link(String),
    BackgroundColor(Color),
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum BoxSizing {
    #[default]
    BorderBox,
    ContentBox,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum Position {
    #[default]
    Relative,
    Absolute,
}

#[derive(Clone, Debug, Copy)]
pub struct StyleProperty<T>
where
    T: Clone + Debug + Copy,
{
    property: T,
    is_dirty: bool,
}

impl<T> StyleProperty<T>
where
    T: Clone + Debug + Copy,
{
    pub fn new(property: T) -> StyleProperty<T> {
        Self {
            property,
            is_dirty: false,
        }
    }

    #[inline(always)]
    pub fn set(&mut self, property: T) {
        self.property = property;
        self.is_dirty = true;
    }

    #[inline(always)]
    pub fn get(&self) -> T {
        self.property
    }

    #[inline(always)]
    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }
}

impl TextStyleProperty {
    pub(crate) fn to_parley_style_property(&self) -> Option<parley::StyleProperty<'static, ColorBrush>> {
        match self {
            TextStyleProperty::FontFamily(font_family) => {
                let font_stack_cow_list = Cow::Owned(vec![
                    parley::FontFamily::Named(Cow::Owned(font_family.to_string())),
                    parley::FontFamily::Generic(parley::GenericFamily::SystemUi),
                ]);
                let font_stack = parley::FontStack::List(font_stack_cow_list);

                Some(parley::StyleProperty::FontStack(font_stack))
            }

            TextStyleProperty::FontSize(font_size) => Some(parley::StyleProperty::FontSize(*font_size)),

            TextStyleProperty::Color(color) => {
                let brush = ColorBrush { color: *color };

                Some(parley::StyleProperty::Brush(brush))
            }

            TextStyleProperty::FontStyle(font_style) => {
                let font_style = match font_style {
                    FontStyle::Normal => parley::FontStyle::Normal,
                    FontStyle::Italic => parley::FontStyle::Italic,
                    // FIXME: Allow an angle when setting the obliqueness.
                    FontStyle::Oblique => parley::FontStyle::Oblique(None),
                };

                Some(parley::StyleProperty::FontStyle(font_style))
            }

            TextStyleProperty::FontWeight(font_weight) => Some(parley::StyleProperty::FontWeight(
                parley::FontWeight::new(font_weight.0 as f32),
            )),
            TextStyleProperty::Underline(underline) => Some(parley::StyleProperty::Underline(*underline)),
            TextStyleProperty::UnderlineOffset(offset) => Some(parley::StyleProperty::UnderlineOffset(Some(*offset))),

            TextStyleProperty::UnderlineSize(size) => Some(parley::StyleProperty::UnderlineSize(Some(*size))),

            TextStyleProperty::UnderlineBrush(color) => {
                let brush = ColorBrush { color: *color };

                Some(parley::StyleProperty::UnderlineBrush(Some(brush)))
            }
            TextStyleProperty::Link(_) | TextStyleProperty::BackgroundColor(_) => None,
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub struct FontFamily {
    font_family_length: u8,
    font_family_name: [u8; 64],
}

impl FontFamily {
    pub fn new(font_family: &str) -> FontFamily {
        let mut font_family_res = FontFamily {
            font_family_length: 0,
            font_family_name: [0; 64],
        };

        let chars = font_family.chars().collect::<Vec<char>>();
        font_family_res.font_family_length = chars.len() as u8;
        font_family_res.font_family_name[..font_family.len()].copy_from_slice(font_family.as_bytes());

        font_family_res
    }

    fn is_empty(&self) -> bool {
        self.font_family_length == 0
    }

    pub fn name(&self) -> Option<&str> {
        if self.is_empty() {
            None
        } else {
            Some(std::str::from_utf8(&self.font_family_name[..self.font_family_length as usize]).unwrap())
        }
    }
}

impl Default for FontFamily {
    fn default() -> FontFamily {
        Self {
            font_family_length: 0,
            font_family_name: [0; 64],
        }
    }
}
