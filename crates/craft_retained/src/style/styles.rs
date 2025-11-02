use crate::style::style_flags::StyleFlags;
use craft_primitives::Color;
use std::borrow::Cow;

pub use taffy::BoxSizing;
pub use taffy::Overflow;
pub use taffy::Position;

use craft_primitives::geometry::TrblRectangle;
use craft_primitives::ColorBrush;
use smallvec::SmallVec;
use std::fmt;
use std::fmt::Debug;
use crate::animations::Animation;

#[derive(Clone, Copy, Debug)]
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
}

#[derive(Clone, Copy, Debug)]
pub enum Display {
    Flex,
    Block,
    None,
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

#[derive(Clone, Copy, Debug)]
pub struct ScrollbarColor {
    pub thumb_color: Color,
    pub track_color: Color,
}

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
    FontWeight(Weight),
    FontStyle(FontStyle),
    UnderlineOffset(f32),
    Underline(bool),
    UnderlineSize(f32),
    UnderlineBrush(Color),
    Link(String),
    BackgroundColor(Color),
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

            TextStyleProperty::FontWeight(font_weight) => {
                Some(parley::StyleProperty::FontWeight(parley::FontWeight::new(font_weight.0 as f32)))
            }
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

#[derive(Clone, Debug, Copy)]
pub enum StyleProperty {
    BoxSizing(BoxSizing),
    Position(Position),
    Margin(TrblRectangle<Unit>),
    Padding(TrblRectangle<Unit>),
    Gap([Unit; 2]),
    Inset(TrblRectangle<Unit>),
    Width(Unit),
    Height(Unit),
    MaxWidth(Unit),
    MaxHeight(Unit),
    MinWidth(Unit),
    MinHeight(Unit),
    X(f32),
    Y(f32),
    Display(Display),
    Wrap(Wrap),
    AlignItems(Option<AlignItems>),
    JustifyContent(Option<JustifyContent>),
    FlexDirection(FlexDirection),
    FlexGrow(f32),
    FlexShrink(f32),
    FlexBasis(Unit),

    Color(Color),
    Background(Color),
    /// Defaults to the text color, if it is None.
    CursorColor(Option<Color>),
    SelectionColor(Color),
    FontFamily(FontFamily),
    FontSize(f32),
    LineHeight(f32),
    FontWeight(Weight),
    FontStyle(FontStyle),
    Underline(Option<Underline>),
    Overflow([Overflow; 2]),

    BorderColor(TrblRectangle<Color>),
    BorderWidth(TrblRectangle<Unit>),
    BorderRadius([(f32, f32); 4]),

    ScrollbarColor(ScrollbarColor),
    ScrollbarThumbMargin(TrblRectangle<f32>),
    ScrollbarRadius([(f32, f32); 4]),
    ScrollbarWidth(f32),

    Visible(bool),
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

#[derive(Clone, Debug)]
pub struct Style {
    properties: SmallVec<[StyleProperty; 5]>,
    pub dirty_flags: StyleFlags,
    pub animations: Vec<Animation>,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            properties: SmallVec::new(),
            dirty_flags: StyleFlags::empty(),
            animations: Vec::new(),
        }
    }
}

macro_rules! style_property {
    (
        $get:ident, $set:ident, $variant:ident, $inner:ty, $flag:ident, $default:expr
    ) => {
        impl Style {
            pub fn $get(&self) -> $inner {
                self.properties
                    .iter()
                    .find_map(|p| if let StyleProperty::$variant(val) = p { Some(*val) } else { None })
                    .unwrap_or($default)
            }

            pub fn $set(&mut self, val: $inner) {
                if self.dirty_flags.contains(StyleFlags::$flag) {
                    self.remove_property(|p| matches!(p, StyleProperty::$variant(_)));
                }
                self.properties.push(StyleProperty::$variant(val));
                self.dirty_flags.insert(StyleFlags::$flag);
            }
        }
    };
}

style_property!(box_sizing, set_box_sizing, BoxSizing, BoxSizing, BOX_SIZING, BoxSizing::BorderBox);
style_property!(position, set_position, Position, Position, POSITION, Position::Relative);
style_property!(margin, set_margin, Margin, TrblRectangle<Unit>, MARGIN, TrblRectangle::new_all(Unit::Px(0.0)));
style_property!(padding, set_padding, Padding, TrblRectangle<Unit>, PADDING, TrblRectangle::new_all(Unit::Px(0.0)));
style_property!(gap, set_gap, Gap, [Unit; 2], GAP, [Unit::Px(0.0); 2]);
style_property!(inset, set_inset, Inset, TrblRectangle<Unit>, INSET, TrblRectangle::new_all(Unit::Px(0.0)));

style_property!(width, set_width, Width, Unit, WIDTH, Unit::Auto);
style_property!(height, set_height, Height, Unit, HEIGHT, Unit::Auto);
style_property!(max_width, set_max_width, MaxWidth, Unit, MAX_WIDTH, Unit::Auto);
style_property!(max_height, set_max_height, MaxHeight, Unit, MAX_HEIGHT, Unit::Auto);
style_property!(min_width, set_min_width, MinWidth, Unit, MIN_WIDTH, Unit::Auto);
style_property!(min_height, set_min_height, MinHeight, Unit, MIN_HEIGHT, Unit::Auto);

style_property!(x, set_x, X, f32, X, 0.0);
style_property!(y, set_y, Y, f32, Y, 0.0);

style_property!(display, set_display, Display, Display, DISPLAY, Display::Flex);
style_property!(wrap, set_wrap, Wrap, Wrap, WRAP, Wrap::default());
style_property!(align_items, set_align_items, AlignItems, Option<AlignItems>, ALIGN_ITEMS, None);
style_property!(justify_content, set_justify_content, JustifyContent, Option<JustifyContent>, JUSTIFY_CONTENT, None);
style_property!(flex_direction, set_flex_direction, FlexDirection, FlexDirection, FLEX_DIRECTION, FlexDirection::Row);
style_property!(flex_grow, set_flex_grow, FlexGrow, f32, FLEX_GROW, 0.0);
style_property!(flex_shrink, set_flex_shrink, FlexShrink, f32, FLEX_SHRINK, 1.0);
style_property!(flex_basis, set_flex_basis, FlexBasis, Unit, FLEX_BASIS, Unit::Auto);

style_property!(font_family, set_font_family, FontFamily, FontFamily, FONT_FAMILY, FontFamily::default());
style_property!(color, set_color, Color, Color, COLOR, Color::BLACK);
style_property!(background, set_background, Background, Color, BACKGROUND, Color::TRANSPARENT);
style_property!(font_size, set_font_size, FontSize, f32, FONT_SIZE, 16.0);
style_property!(line_height, set_line_height, LineHeight, f32, LINE_HEIGHT, 1.2);
style_property!(font_weight, set_font_weight, FontWeight, Weight, FONT_WEIGHT, Weight::default());
style_property!(font_style, set_font_style, FontStyle, FontStyle, FONT_STYLE, FontStyle::default());
style_property!(underline, set_underline, Underline, Option<Underline>, UNDERLINE, None);
style_property!(overflow, set_overflow, Overflow, [Overflow; 2], OVERFLOW, [Overflow::default(); 2]);

style_property!(
    border_color,
    set_border_color,
    BorderColor,
    TrblRectangle<Color>,
    BORDER_COLOR,
    TrblRectangle::new_all(Color::BLACK)
);
style_property!(
    border_width,
    set_border_width,
    BorderWidth,
    TrblRectangle<Unit>,
    BORDER_WIDTH,
    TrblRectangle::new_all(Unit::Px(0.0))
);
style_property!(border_radius, set_border_radius, BorderRadius, [(f32, f32); 4], BORDER_RADIUS, [(0.0, 0.0); 4]);

style_property!(
    scrollbar_color,
    set_scrollbar_color,
    ScrollbarColor,
    ScrollbarColor,
    SCROLLBAR_COLOR,
    ScrollbarColor {
        thumb_color: Color::from_rgb8(150, 150, 152),
        track_color: Color::TRANSPARENT
    }
);
const SCROLLBAR_THUMB_MARGIN: TrblRectangle<f32> = if cfg!(any(target_os = "android", target_os = "ios")) {
    TrblRectangle::new_all(0.0)
} else {
    TrblRectangle::new(1.0, 2.0, 1.0, 2.0)
};
style_property!(
    scrollbar_thumb_margin,
    set_scrollbar_thumb_margin,
    ScrollbarThumbMargin,
    TrblRectangle<f32>,
    SCROLLBAR_THUMB_MARGIN,
    SCROLLBAR_THUMB_MARGIN
);
style_property!(
    scrollbar_thumb_radius,
    set_scrollbar_thumb_radius,
    ScrollbarRadius,
    [(f32, f32); 4],
    SCROLLBAR_RADIUS,
    [(10.0, 10.0); 4]
);
style_property!(
    scrollbar_width,
    set_scrollbar_width,
    ScrollbarWidth,
    f32,
    SCROLLBAR_WIDTH,
    if cfg!(any(target_os = "android", target_os = "ios")) { 0.0 } else { 10.0 }
);

style_property!(visible, set_visible, Visible, bool, VISIBLE, true);
style_property!(
    selection_color,
    set_selection_color,
    SelectionColor,
    Color,
    SELECTION_COLOR,
    Color::from_rgb8(0, 120, 215)
);
style_property!(cursor_color, set_cursor_color, CursorColor, Option<Color>, CURSOR_COLOR, None);

impl Style {

    fn remove_property(&mut self, f: impl Fn(&StyleProperty) -> bool) {
        if let Some(pos) = self.properties.iter().position(f) {
            self.properties.remove(pos);
        }
    }

    pub fn has_border(&self) -> bool {
        self.dirty_flags.contains(StyleFlags::BORDER_WIDTH)
            || self.dirty_flags.contains(StyleFlags::BORDER_RADIUS)
            || self.dirty_flags.contains(StyleFlags::BORDER_COLOR)
    }

    /// Take an old style and update it with the non-default values from the new style.
    pub fn merge(old: &Self, new: &Self) -> Self {
        // If either is fully default, return the other early.
        if old.dirty_flags.is_empty() {
            return new.clone();
        }

        if new.dirty_flags.is_empty() {
            return old.clone();
        }

        let mut merged = old.clone();

        for prop in &new.properties {
            let flag = match prop {
                StyleProperty::BoxSizing(_) => StyleFlags::BOX_SIZING,
                StyleProperty::Position(_) => StyleFlags::POSITION,
                StyleProperty::Margin(_) => StyleFlags::MARGIN,
                StyleProperty::Padding(_) => StyleFlags::PADDING,
                StyleProperty::Gap(_) => StyleFlags::GAP,
                StyleProperty::Inset(_) => StyleFlags::INSET,
                StyleProperty::Width(_) => StyleFlags::WIDTH,
                StyleProperty::Height(_) => StyleFlags::HEIGHT,
                StyleProperty::MaxWidth(_) => StyleFlags::MAX_WIDTH,
                StyleProperty::MaxHeight(_) => StyleFlags::MAX_HEIGHT,
                StyleProperty::MinWidth(_) => StyleFlags::MIN_WIDTH,
                StyleProperty::MinHeight(_) => StyleFlags::MIN_HEIGHT,
                StyleProperty::X(_) => StyleFlags::X,
                StyleProperty::Y(_) => StyleFlags::Y,
                StyleProperty::Display(_) => StyleFlags::DISPLAY,
                StyleProperty::Wrap(_) => StyleFlags::WRAP,
                StyleProperty::AlignItems(_) => StyleFlags::ALIGN_ITEMS,
                StyleProperty::JustifyContent(_) => StyleFlags::JUSTIFY_CONTENT,
                StyleProperty::FlexDirection(_) => StyleFlags::FLEX_DIRECTION,
                StyleProperty::FlexGrow(_) => StyleFlags::FLEX_GROW,
                StyleProperty::FlexShrink(_) => StyleFlags::FLEX_SHRINK,
                StyleProperty::FlexBasis(_) => StyleFlags::FLEX_BASIS,
                StyleProperty::FontFamily(_) => StyleFlags::FONT_FAMILY,
                StyleProperty::Color(_) => StyleFlags::COLOR,
                StyleProperty::Background(_) => StyleFlags::BACKGROUND,
                StyleProperty::FontSize(_) => StyleFlags::FONT_SIZE,
                StyleProperty::FontWeight(_) => StyleFlags::FONT_WEIGHT,
                StyleProperty::FontStyle(_) => StyleFlags::FONT_STYLE,
                StyleProperty::Underline(_) => StyleFlags::UNDERLINE,
                StyleProperty::Overflow(_) => StyleFlags::OVERFLOW,
                StyleProperty::BorderColor(_) => StyleFlags::BORDER_COLOR,
                StyleProperty::BorderWidth(_) => StyleFlags::BORDER_WIDTH,
                StyleProperty::BorderRadius(_) => StyleFlags::BORDER_RADIUS,
                StyleProperty::ScrollbarColor(_) => StyleFlags::SCROLLBAR_COLOR,
                StyleProperty::ScrollbarRadius(_) => StyleFlags::SCROLLBAR_RADIUS,
                StyleProperty::ScrollbarThumbMargin(_) => StyleFlags::SCROLLBAR_THUMB_MARGIN,
                StyleProperty::ScrollbarWidth(_) => StyleFlags::SCROLLBAR_WIDTH,
                StyleProperty::Visible(_) => StyleFlags::VISIBLE,
                StyleProperty::SelectionColor(_) => StyleFlags::SELECTION_COLOR,
                StyleProperty::CursorColor(_) => StyleFlags::CURSOR_COLOR,
                StyleProperty::LineHeight(_) => StyleFlags::LINE_HEIGHT,
            };

            if new.dirty_flags.contains(flag) {
                // Remove from merged if it already exists
                merged.remove_property(|p| std::mem::discriminant(p) == std::mem::discriminant(prop));
                // Push the updated property
                merged.properties.push(*prop);
                merged.dirty_flags.insert(flag);
            }
        }

        merged
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_text_style(&self) -> parley::TextStyle<ColorBrush> {
        let font_size = self.font_size();
        let line_height = self.line_height();
        let font_weight = parley::FontWeight::new(self.font_weight().0 as f32);
        let font_style = match self.font_style() {
            FontStyle::Normal => parley::FontStyle::Normal,
            FontStyle::Italic => parley::FontStyle::Italic,
            // FIXME: Allow an angle when setting the obliqueness.
            FontStyle::Oblique => parley::FontStyle::Oblique(None),
        };
        let brush = ColorBrush {
            color: self.color(),
        };

        let font_stack_cow_list = if let Some(font_family) = self.font_family().name() {
            // Use the user-provided font and fallback to system UI fonts as needed.
            Cow::Owned(vec![
                parley::FontFamily::Named(Cow::Owned(font_family.to_string())),
                parley::FontFamily::Generic(parley::GenericFamily::SystemUi),
            ])
        } else {
            // Just default to system UI fonts.
            Cow::Owned(vec![parley::FontFamily::Generic(parley::GenericFamily::SystemUi)])
        };

        let underline = self.underline();
        let has_underline = underline.is_some();
        let mut underline_offset = None;
        let mut underline_size = None;
        let mut underline_brush = None;

        if let Some(underline) = underline {
            underline_offset = underline.offset;
            underline_size = underline.thickness;
            underline_brush = Some(ColorBrush {
                color: underline.color,
            });
        }

        let font_stack = parley::FontStack::List(font_stack_cow_list);
        parley::TextStyle {
            font_stack,
            font_size,
            font_width: Default::default(),
            font_style,
            font_weight,
            font_variations: parley::FontSettings::List(Cow::Borrowed(&[])),
            font_features: parley::FontSettings::List(Cow::Borrowed(&[])),
            locale: Default::default(),
            brush,
            has_underline,
            underline_offset,
            underline_size,
            underline_brush,
            has_strikethrough: Default::default(),
            strikethrough_offset: Default::default(),
            strikethrough_size: Default::default(),
            strikethrough_brush: Default::default(),
            line_height: parley::LineHeight::FontSizeRelative(line_height),
            word_spacing: Default::default(),
            letter_spacing: Default::default(),
            word_break: Default::default(),
            overflow_wrap: Default::default(),
        }
    }

    pub fn add_styles_to_style_set(&self, style_set: &mut parley::StyleSet<ColorBrush>) {
        let font_size = self.font_size();
        let line_height = self.line_height();
        let font_weight = parley::FontWeight::new(self.font_weight().0 as f32);
        let font_style = match self.font_style() {
            FontStyle::Normal => parley::FontStyle::Normal,
            FontStyle::Italic => parley::FontStyle::Italic,
            // FIXME: Allow an angle when setting the obliqueness.
            FontStyle::Oblique => parley::FontStyle::Oblique(None),
        };
        let brush = ColorBrush {
            color: self.color(),
        };

        let underline = self.underline();
        let has_underline = underline.is_some();
        let mut underline_offset = None;
        let mut underline_size = None;
        let mut underline_brush = None;

        if let Some(underline) = underline {
            underline_offset = underline.offset;
            underline_size = underline.thickness;
            underline_brush = Some(ColorBrush {
                color: underline.color,
            });
        }

        let font_family = self.font_family();
        let font_stack_cow_list = if let Some(font_family) = font_family.name() {
            // Use the user-provided font and fallback to system UI fonts as needed.
            Cow::Owned(vec![
                parley::FontFamily::Named(Cow::Owned(font_family.to_string())),
                parley::FontFamily::Generic(parley::GenericFamily::SystemUi),
            ])
        } else {
            // Just default to system UI fonts.
            Cow::Owned(vec![parley::FontFamily::Generic(parley::GenericFamily::SystemUi)])
        };

        style_set.insert(parley::StyleProperty::from(parley::FontStack::List(font_stack_cow_list)));
        style_set.insert(parley::StyleProperty::FontSize(font_size));
        style_set.insert(parley::StyleProperty::FontStyle(font_style));
        style_set.insert(parley::StyleProperty::FontWeight(font_weight));
        style_set.insert(parley::StyleProperty::Brush(brush));
        style_set.insert(parley::StyleProperty::LineHeight(parley::LineHeight::FontSizeRelative(line_height)));
        style_set.insert(parley::StyleProperty::Underline(has_underline));
        style_set.insert(parley::StyleProperty::UnderlineBrush(underline_brush));
        style_set.insert(parley::StyleProperty::UnderlineOffset(underline_offset));
        style_set.insert(parley::StyleProperty::UnderlineSize(underline_size));
    }
}
