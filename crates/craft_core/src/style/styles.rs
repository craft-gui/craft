use crate::renderer::color::Color;
use crate::style::style_flags::StyleFlags;
use std::borrow::Cow;

pub use taffy::BoxSizing;
pub use taffy::Overflow;
pub use taffy::Position;

use crate::geometry::TrblRectangle;
use crate::text::text_context::ColorBrush;
use parley::{FontFamily, FontSettings, FontStack, GenericFamily, StyleProperty, StyleSet, TextStyle};
use std::fmt;
use parley::LineHeight::FontSizeRelative;

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

#[derive(Clone, Copy, Debug)]
pub struct Style {
    font_family_length: u8,
    font_family: [u8; 64],
    box_sizing: BoxSizing,
    scrollbar_width: f32,
    position: Position,
    margin: TrblRectangle<Unit>,
    padding: TrblRectangle<Unit>,
    gap: [Unit; 2],
    inset: TrblRectangle<Unit>,
    width: Unit,
    height: Unit,
    max_width: Unit,
    max_height: Unit,
    min_width: Unit,
    min_height: Unit,
    x: f32,
    y: f32,
    display: Display,
    wrap: Wrap,
    align_items: Option<AlignItems>,
    justify_content: Option<JustifyContent>,
    flex_direction: FlexDirection,
    flex_grow: f32,
    flex_shrink: f32,
    flex_basis: Unit,

    color: Color,
    background: Color,
    font_size: f32,
    font_weight: Weight,
    font_style: FontStyle,
    underline: Option<Underline>,
    overflow: [Overflow; 2],

    border_color: TrblRectangle<Color>,
    border_width: TrblRectangle<Unit>,
    border_radius: [(f32, f32); 4],
    scrollbar_color: ScrollbarColor,

    /// The element is measured and occupies space, but is not drawn to the screen.
    visible: bool,

    pub dirty_flags: StyleFlags,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            font_family_length: 0,
            font_family: [0; 64],
            box_sizing: BoxSizing::BorderBox,
            scrollbar_width: if cfg!(any(target_os = "android", target_os = "ios")) { 0.0 } else { 10.0 },
            position: Position::Relative,
            margin: TrblRectangle::new_all(Unit::Px(0.0)),
            padding: TrblRectangle::new_all(Unit::Px(0.0)),
            border_width: TrblRectangle::new_all(Unit::Px(0.0)),
            gap: [Unit::Px(0.0); 2],
            inset: TrblRectangle::new_all(Unit::Px(0.0)),
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
            color: Color::BLACK,
            background: Color::TRANSPARENT,
            border_color: TrblRectangle::new_all(Color::BLACK),
            font_size: 16.0,
            font_weight: Default::default(),
            font_style: Default::default(),
            underline: None,
            overflow: [Overflow::default(), Overflow::default()],
            border_radius: [(0.0, 0.0); 4],
            scrollbar_color: ScrollbarColor {
                thumb_color: Color::from_rgb8(150, 150, 150),
                track_color: Color::from_rgb8(100, 100, 100),
            },
            visible: true,
            dirty_flags: StyleFlags::empty(),
        }
    }
}

impl Style {
    pub fn font_family(&self) -> Option<&str> {
        if self.font_family_length == 0 {
            None
        } else {
            Some(std::str::from_utf8(&self.font_family[..self.font_family_length as usize]).unwrap())
        }
    }

    pub(crate) fn set_font_family(&mut self, font_family: &str) {
        let chars = font_family.chars().collect::<Vec<char>>();

        self.font_family_length = chars.len() as u8;
        self.font_family[..font_family.len()].copy_from_slice(font_family.as_bytes());
        self.dirty_flags.insert(StyleFlags::FONT_FAMILY);
    }

    pub fn font_family_raw(&self) -> [u8; 64] {
        self.font_family
    }

    pub fn font_family_mut(&mut self) -> &mut [u8; 64] {
        self.dirty_flags.insert(StyleFlags::FONT_FAMILY);
        &mut self.font_family
    }

    pub fn font_family_length(&self) -> u8 {
        self.font_family_length
    }

    pub fn font_family_length_mut(&mut self) -> &mut u8 {
        self.dirty_flags.insert(StyleFlags::FONT_FAMILY_LENGTH);
        &mut self.font_family_length
    }

    pub fn box_sizing(&self) -> BoxSizing {
        self.box_sizing
    }

    pub fn box_sizing_mut(&mut self) -> &mut BoxSizing {
        self.dirty_flags.insert(StyleFlags::BOX_SIZING);
        &mut self.box_sizing
    }

    pub fn scrollbar_width(&self) -> f32 {
        self.scrollbar_width
    }

    pub fn scrollbar_width_mut(&mut self) -> &mut f32 {
        self.dirty_flags.insert(StyleFlags::SCROLLBAR_WIDTH);
        &mut self.scrollbar_width
    }

    pub fn position(&self) -> Position {
        self.position
    }

    pub fn position_mut(&mut self) -> &mut Position {
        self.dirty_flags.insert(StyleFlags::POSITION);
        &mut self.position
    }

    pub fn margin(&self) -> TrblRectangle<Unit> {
        self.margin
    }

    pub fn margin_mut(&mut self) -> &mut TrblRectangle<Unit> {
        self.dirty_flags.insert(StyleFlags::MARGIN);
        &mut self.margin
    }

    pub fn padding(&self) -> TrblRectangle<Unit> {
        self.padding
    }

    pub fn padding_mut(&mut self) -> &mut TrblRectangle<Unit> {
        self.dirty_flags.insert(StyleFlags::PADDING);
        &mut self.padding
    }

    pub fn gap(&self) -> [Unit; 2] {
        self.gap
    }

    pub fn gap_mut(&mut self) -> &mut [Unit; 2] {
        self.dirty_flags.insert(StyleFlags::GAP);
        &mut self.gap
    }

    pub fn inset(&self) -> TrblRectangle<Unit> {
        self.inset
    }

    pub fn inset_mut(&mut self) -> &mut TrblRectangle<Unit> {
        self.dirty_flags.insert(StyleFlags::INSET);
        &mut self.inset
    }

    pub fn width(&self) -> Unit {
        self.width
    }

    pub fn width_mut(&mut self) -> &mut Unit {
        self.dirty_flags.insert(StyleFlags::WIDTH);
        &mut self.width
    }

    pub fn height(&self) -> Unit {
        self.height
    }

    pub fn height_mut(&mut self) -> &mut Unit {
        self.dirty_flags.insert(StyleFlags::HEIGHT);
        &mut self.height
    }

    pub fn max_width(&self) -> Unit {
        self.max_width
    }

    pub fn max_width_mut(&mut self) -> &mut Unit {
        self.dirty_flags.insert(StyleFlags::MAX_WIDTH);
        &mut self.max_width
    }

    pub fn max_height(&self) -> Unit {
        self.max_height
    }

    pub fn max_height_mut(&mut self) -> &mut Unit {
        self.dirty_flags.insert(StyleFlags::MAX_HEIGHT);
        &mut self.max_height
    }

    pub fn min_width(&self) -> Unit {
        self.min_width
    }

    pub fn min_width_mut(&mut self) -> &mut Unit {
        self.dirty_flags.insert(StyleFlags::MIN_WIDTH);
        &mut self.min_width
    }

    pub fn min_height(&self) -> Unit {
        self.min_height
    }

    pub fn min_height_mut(&mut self) -> &mut Unit {
        self.dirty_flags.insert(StyleFlags::MIN_HEIGHT);
        &mut self.min_height
    }

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn x_mut(&mut self) -> &mut f32 {
        self.dirty_flags.insert(StyleFlags::X);
        &mut self.x
    }

    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn y_mut(&mut self) -> &mut f32 {
        self.dirty_flags.insert(StyleFlags::Y);
        &mut self.y
    }

    pub fn display(&self) -> Display {
        self.display
    }

    pub fn display_mut(&mut self) -> &mut Display {
        self.dirty_flags.insert(StyleFlags::DISPLAY);
        &mut self.display
    }

    pub fn wrap(&self) -> Wrap {
        self.wrap
    }

    pub fn wrap_mut(&mut self) -> &mut Wrap {
        self.dirty_flags.insert(StyleFlags::WRAP);
        &mut self.wrap
    }

    pub fn align_items(&self) -> Option<AlignItems> {
        self.align_items
    }

    pub fn align_items_mut(&mut self) -> &mut Option<AlignItems> {
        self.dirty_flags.insert(StyleFlags::ALIGN_ITEMS);
        &mut self.align_items
    }

    pub fn justify_content(&self) -> Option<JustifyContent> {
        self.justify_content
    }

    pub fn justify_content_mut(&mut self) -> &mut Option<JustifyContent> {
        self.dirty_flags.insert(StyleFlags::JUSTIFY_CONTENT);
        &mut self.justify_content
    }

    pub fn flex_direction(&self) -> FlexDirection {
        self.flex_direction
    }

    pub fn flex_direction_mut(&mut self) -> &mut FlexDirection {
        self.dirty_flags.insert(StyleFlags::FLEX_DIRECTION);
        &mut self.flex_direction
    }

    pub fn flex_grow(&self) -> f32 {
        self.flex_grow
    }

    pub fn flex_grow_mut(&mut self) -> &mut f32 {
        self.dirty_flags.insert(StyleFlags::FLEX_GROW);
        &mut self.flex_grow
    }

    pub fn flex_shrink(&self) -> f32 {
        self.flex_shrink
    }

    pub fn flex_shrink_mut(&mut self) -> &mut f32 {
        self.dirty_flags.insert(StyleFlags::FLEX_SHRINK);
        &mut self.flex_shrink
    }

    pub fn flex_basis(&self) -> Unit {
        self.flex_basis
    }

    pub fn flex_basis_mut(&mut self) -> &mut Unit {
        self.dirty_flags.insert(StyleFlags::FLEX_BASIS);
        &mut self.flex_basis
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn color_mut(&mut self) -> &mut Color {
        self.dirty_flags.insert(StyleFlags::COLOR);
        &mut self.color
    }

    pub fn background(&self) -> Color {
        self.background
    }

    pub fn background_mut(&mut self) -> &mut Color {
        self.dirty_flags.insert(StyleFlags::BACKGROUND);
        &mut self.background
    }

    pub fn font_size(&self) -> f32 {
        self.font_size
    }

    pub fn font_size_mut(&mut self) -> &mut f32 {
        self.dirty_flags.insert(StyleFlags::FONT_SIZE);
        &mut self.font_size
    }

    pub fn font_weight(&self) -> Weight {
        self.font_weight
    }

    pub fn font_weight_mut(&mut self) -> &mut Weight {
        self.dirty_flags.insert(StyleFlags::FONT_WEIGHT);
        &mut self.font_weight
    }

    pub fn font_style(&self) -> FontStyle {
        self.font_style
    }

    pub fn font_style_mut(&mut self) -> &mut FontStyle {
        self.dirty_flags.insert(StyleFlags::FONT_STYLE);
        &mut self.font_style
    }

    pub fn underline(&self) -> Option<Underline> {
        self.underline
    }

    pub fn underline_mut(&mut self) -> &mut Option<Underline> {
        self.dirty_flags.insert(StyleFlags::UNDERLINE);
        &mut self.underline
    }

    pub fn overflow(&self) -> [Overflow; 2] {
        self.overflow
    }

    pub fn overflow_mut(&mut self) -> &mut [Overflow; 2] {
        self.dirty_flags.insert(StyleFlags::OVERFLOW);
        &mut self.overflow
    }

    pub fn border_color(&self) -> TrblRectangle<Color> {
        self.border_color
    }

    pub fn border_color_mut(&mut self) -> &mut TrblRectangle<Color> {
        self.dirty_flags.insert(StyleFlags::BORDER_COLOR);
        &mut self.border_color
    }

    pub fn border_width(&self) -> TrblRectangle<Unit> {
        self.border_width
    }

    pub fn border_width_mut(&mut self) -> &mut TrblRectangle<Unit> {
        self.dirty_flags.insert(StyleFlags::BORDER_WIDTH);
        &mut self.border_width
    }

    pub fn border_radius(&self) -> [(f32, f32); 4] {
        self.border_radius
    }

    pub fn border_radius_mut(&mut self) -> &mut [(f32, f32); 4] {
        self.dirty_flags.insert(StyleFlags::BORDER_RADIUS);
        &mut self.border_radius
    }

    pub fn scrollbar_color(&self) -> ScrollbarColor {
        self.scrollbar_color
    }

    pub fn scrollbar_color_mut(&mut self) -> &mut ScrollbarColor {
        self.dirty_flags.insert(StyleFlags::SCROLLBAR_COLOR);
        &mut self.scrollbar_color
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    pub fn visible_mut(&mut self) -> &mut bool {
        self.dirty_flags.insert(StyleFlags::VISIBLE);
        &mut self.visible
    }

    pub fn has_border(&self) -> bool {
        self.dirty_flags.contains(StyleFlags::BORDER_WIDTH)
            || self.dirty_flags.contains(StyleFlags::BORDER_RADIUS)
            || self.dirty_flags.contains(StyleFlags::BORDER_COLOR)
    }

    /// Take an old style and update it with the non-default values from the new style.
    pub fn merge(old: &Self, new: &Self) -> Self {
        let old_dirty_flags = old.dirty_flags;
        let new_dirty_flags = new.dirty_flags;

        if old_dirty_flags.is_empty() {
            return *new;
        }

        if new_dirty_flags.is_empty() {
            return *old;
        }

        let font_family_length = if new_dirty_flags.contains(StyleFlags::FONT_FAMILY_LENGTH) {
            new.font_family_length
        } else {
            old.font_family_length
        };

        let font_family =
            if new_dirty_flags.contains(StyleFlags::FONT_FAMILY) { new.font_family } else { old.font_family };

        let box_sizing = if new_dirty_flags.contains(StyleFlags::BOX_SIZING) { new.box_sizing } else { old.box_sizing };

        let scrollbar_width = if new_dirty_flags.contains(StyleFlags::SCROLLBAR_WIDTH) {
            new.scrollbar_width
        } else {
            old.scrollbar_width
        };

        let position = if new_dirty_flags.contains(StyleFlags::POSITION) { new.position } else { old.position };

        let margin = if new_dirty_flags.contains(StyleFlags::MARGIN) { new.margin } else { old.margin };

        let padding = if new_dirty_flags.contains(StyleFlags::PADDING) { new.padding } else { old.padding };

        let gap = if new_dirty_flags.contains(StyleFlags::GAP) { new.gap } else { old.gap };

        let inset = if new_dirty_flags.contains(StyleFlags::INSET) { new.inset } else { old.inset };

        let width = if new_dirty_flags.contains(StyleFlags::WIDTH) { new.width } else { old.width };

        let height = if new_dirty_flags.contains(StyleFlags::HEIGHT) { new.height } else { old.height };

        let max_width = if new_dirty_flags.contains(StyleFlags::MAX_WIDTH) { new.max_width } else { old.max_width };

        let max_height = if new_dirty_flags.contains(StyleFlags::MAX_HEIGHT) { new.max_height } else { old.max_height };

        let min_width = if new_dirty_flags.contains(StyleFlags::MIN_WIDTH) { new.min_width } else { old.min_width };

        let min_height = if new_dirty_flags.contains(StyleFlags::MIN_HEIGHT) { new.min_height } else { old.min_height };

        let x = if new_dirty_flags.contains(StyleFlags::X) { new.x } else { old.x };

        let y = if new_dirty_flags.contains(StyleFlags::Y) { new.y } else { old.y };

        let display = if new_dirty_flags.contains(StyleFlags::DISPLAY) { new.display } else { old.display };

        let wrap = if new_dirty_flags.contains(StyleFlags::WRAP) { new.wrap } else { old.wrap };

        let align_items =
            if new_dirty_flags.contains(StyleFlags::ALIGN_ITEMS) { new.align_items } else { old.align_items };

        let justify_content = if new_dirty_flags.contains(StyleFlags::JUSTIFY_CONTENT) {
            new.justify_content
        } else {
            old.justify_content
        };

        let flex_direction =
            if new_dirty_flags.contains(StyleFlags::FLEX_DIRECTION) { new.flex_direction } else { old.flex_direction };

        let flex_grow = if new_dirty_flags.contains(StyleFlags::FLEX_GROW) { new.flex_grow } else { old.flex_grow };

        let flex_shrink =
            if new_dirty_flags.contains(StyleFlags::FLEX_SHRINK) { new.flex_shrink } else { old.flex_shrink };

        let flex_basis = if new_dirty_flags.contains(StyleFlags::FLEX_BASIS) { new.flex_basis } else { old.flex_basis };

        let color = if new_dirty_flags.contains(StyleFlags::COLOR) { new.color } else { old.color };

        let background = if new_dirty_flags.contains(StyleFlags::BACKGROUND) { new.background } else { old.background };

        let font_size = if new_dirty_flags.contains(StyleFlags::FONT_SIZE) { new.font_size } else { old.font_size };

        let font_weight =
            if new_dirty_flags.contains(StyleFlags::FONT_WEIGHT) { new.font_weight } else { old.font_weight };

        let font_style = if new_dirty_flags.contains(StyleFlags::FONT_STYLE) { new.font_style } else { old.font_style };

        let overflow = if new_dirty_flags.contains(StyleFlags::OVERFLOW) { new.overflow } else { old.overflow };

        let border_color =
            if new_dirty_flags.contains(StyleFlags::BORDER_COLOR) { new.border_color } else { old.border_color };

        let border_width =
            if new_dirty_flags.contains(StyleFlags::BORDER_WIDTH) { new.border_width } else { old.border_width };

        let border_radius =
            if new_dirty_flags.contains(StyleFlags::BORDER_RADIUS) { new.border_radius } else { old.border_radius };

        let scrollbar_color = if new_dirty_flags.contains(StyleFlags::SCROLLBAR_COLOR) {
            new.scrollbar_color
        } else {
            old.scrollbar_color
        };

        let visible = if new_dirty_flags.contains(StyleFlags::VISIBLE) { new.visible } else { old.visible };

        let underline = if new_dirty_flags.contains(StyleFlags::UNDERLINE) { new.underline } else { old.underline };
        
        let dirty_flags = old_dirty_flags | new_dirty_flags;

        Self {
            font_family_length,
            font_family,
            box_sizing,
            scrollbar_width,
            position,
            margin,
            padding,
            gap,
            inset,
            width,
            height,
            max_width,
            max_height,
            min_width,
            min_height,
            x,
            y,
            display,
            wrap,
            align_items,
            justify_content,
            flex_direction,
            flex_grow,
            flex_shrink,
            flex_basis,
            color,
            background,
            font_size,
            font_weight,
            font_style,
            underline,
            overflow,
            border_color,
            border_width,
            border_radius,
            scrollbar_color,
            visible,
            dirty_flags,
        }
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_text_style(&self) -> TextStyle<ColorBrush> {
        let font_size = self.font_size();
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

        let font_stack_cow_list = if let Some(font_family) = self.font_family() {
            // Use the user-provided font and fallback to system UI fonts as needed.
            Cow::Owned(vec![
                FontFamily::Named(Cow::Borrowed(font_family)),
                FontFamily::Generic(GenericFamily::SystemUi),
            ])
        } else {
            // Just default to system UI fonts.
            Cow::Owned(vec![FontFamily::Generic(GenericFamily::SystemUi)])
        };
        
        let has_underline = self.underline.is_some();
        let mut underline_offset = None;
        let mut underline_size = None;
        let mut underline_brush = None;
        
        if let Some(underline) = self.underline {
            underline_offset = underline.offset;
            underline_size = underline.thickness;
            underline_brush = Some(ColorBrush {
                color: underline.color,
            });
        }

        let font_stack = FontStack::List(font_stack_cow_list);
        TextStyle {
            font_stack,
            font_size,
            font_width: Default::default(),
            font_style,
            font_weight,
            font_variations: FontSettings::List(Cow::Borrowed(&[])),
            font_features: FontSettings::List(Cow::Borrowed(&[])),
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
            line_height: FontSizeRelative(1.2),
            word_spacing: Default::default(),
            letter_spacing: Default::default(),
            word_break: Default::default(),
            overflow_wrap: Default::default(),
        }
    }

    pub fn add_styles_to_style_set(&self, style_set: &mut StyleSet<ColorBrush>) {
        let font_size = self.font_size();
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

        let has_underline = self.underline.is_some();
        let mut underline_offset = None;
        let mut underline_size = None;
        let mut underline_brush = None;

        if let Some(underline) = self.underline {
            underline_offset = underline.offset;
            underline_size = underline.thickness;
            underline_brush = Some(ColorBrush {
                color: underline.color,
            });
        }

        let font_stack_cow_list = if let Some(font_family) = self.font_family() {
            // Use the user-provided font and fallback to system UI fonts as needed.
            Cow::Owned(vec![
                FontFamily::Named(Cow::Owned(font_family.to_string())),
                FontFamily::Generic(GenericFamily::SystemUi),
            ])
        } else {
            // Just default to system UI fonts.
            Cow::Owned(vec![FontFamily::Generic(GenericFamily::SystemUi)])
        };

        style_set.insert(StyleProperty::from(FontStack::List(font_stack_cow_list)));
        style_set.insert(StyleProperty::FontSize(font_size));
        style_set.insert(StyleProperty::FontStyle(font_style));
        style_set.insert(StyleProperty::FontWeight(font_weight));
        style_set.insert(StyleProperty::Brush(brush));
        style_set.insert(StyleProperty::LineHeight(FontSizeRelative(1.2)));
        style_set.insert(StyleProperty::Underline(has_underline));
        style_set.insert(StyleProperty::UnderlineBrush(underline_brush));
        style_set.insert(StyleProperty::UnderlineOffset(underline_offset));
        style_set.insert(StyleProperty::UnderlineSize(underline_size));
    }
    
}
