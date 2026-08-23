use std::borrow::Cow;
use std::fmt;
use std::fmt::Debug;

use parley::GenericFamily;

use retgui_primitives::brush::Brush;

pub use animations::animation::{Animation, Repeat};
pub use animations::keyframe::KeyFrame;
pub use animations::timing_function::TimingFunction;
pub use box_shadow::BoxShadow;
pub use styles::*;

mod animations;
mod box_shadow;
mod gummy_conversions;
mod styles;

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
            Unit::Auto => 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Display {
    Flex,
    Block,
    None,
}

/// Controls how child nodes are aligned in the cross/block axis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AlignItems {
    #[default]
    Normal,
    Start,
    End,
    FlexStart,
    FlexEnd,
    SelfStart,
    SelfEnd,
    Center,
    Baseline,
    Stretch,
    SafeStart,
    SafeEnd,
    SafeFlexStart,
    SafeFlexEnd,
    SafeSelfStart,
    SafeSelfEnd,
    SafeCenter,
}

/// Controls how an individual node is aligned in the cross/block axis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AlignSelf {
    #[default]
    Auto,
    Normal,
    Start,
    End,
    FlexStart,
    FlexEnd,
    SelfStart,
    SelfEnd,
    Center,
    Baseline,
    Stretch,
    SafeStart,
    SafeEnd,
    SafeFlexStart,
    SafeFlexEnd,
    SafeSelfStart,
    SafeSelfEnd,
    SafeCenter,
}

/// Controls how content is distributed in the cross/block axis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AlignContent {
    #[default]
    Normal,
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
    SpaceBetween,
    SpaceEvenly,
    SpaceAround,
    SafeStart,
    SafeEnd,
    SafeFlexStart,
    SafeFlexEnd,
    SafeCenter,
}

/// Controls how content is distributed in the main/inline axis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum JustifyContent {
    #[default]
    Normal,
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
    SpaceBetween,
    SpaceEvenly,
    SpaceAround,
    SafeStart,
    SafeEnd,
    SafeFlexStart,
    SafeFlexEnd,
    SafeCenter,
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

#[derive(Clone, Debug)]
pub struct ScrollbarColor {
    pub thumb_color: Brush,
    pub track_color: Brush,
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

#[derive(Clone, Debug, PartialEq)]
pub struct Underline {
    pub thickness: Option<f32>,
    pub brush: Brush,
    pub offset: Option<f32>,
}

#[derive(Copy, Clone, Default, PartialEq, Eq, Debug, Hash)]
pub enum TextAlign {
    #[default]
    Start,
    End,
    Left,
    Center,
    Right,
    Justify,
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
    Color(Brush),
    FontFamily(String),
    FontSize(f32),
    FontWeight(FontWeight),
    FontStyle(FontStyle),
    UnderlineOffset(f32),
    Underline(bool),
    UnderlineSize(f32),
    UnderlineBrush(Brush),
    Link(String),
    BackgroundBrush(Brush),
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

#[derive(Clone, Debug)]
struct StyleProperty<T>
where
    T: Clone + Debug,
{
    property: T,
    is_dirty: bool,
}

impl<T> StyleProperty<T>
where
    T: Clone + Debug,
{
    fn new(property: T) -> StyleProperty<T> {
        Self {
            property,
            is_dirty: false,
        }
    }

    #[inline(always)]
    fn set(&mut self, property: T) {
        self.property = property;
        self.is_dirty = true;
    }

    #[inline(always)]
    fn get(&self) -> &T {
        &self.property
    }

    #[inline(always)]
    fn is_dirty(&self) -> bool {
        self.is_dirty
    }
}

impl TextStyleProperty {
    pub(crate) fn to_parley_style_property(&self) -> Option<parley::StyleProperty<'_, Brush>> {
        match self {
            TextStyleProperty::FontFamily(font_family) => {
                let font_stack_cow_list = Cow::Owned(vec![
                    parley::FontFamilyName::named(font_family),
                    parley::FontFamilyName::Generic(GenericFamily::SystemUi),
                ]);
                let font_stack = parley::FontFamily::List(font_stack_cow_list);

                Some(parley::StyleProperty::FontFamily(font_stack))
            }

            TextStyleProperty::FontSize(font_size) => Some(parley::StyleProperty::FontSize(*font_size)),

            TextStyleProperty::Color(brush) => Some(parley::StyleProperty::Brush(brush.clone())),

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

            TextStyleProperty::UnderlineBrush(brush) => {
                Some(parley::StyleProperty::UnderlineBrush(Some(brush.clone())))
            }
            TextStyleProperty::Link(_) | TextStyleProperty::BackgroundBrush(_) => None,
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
