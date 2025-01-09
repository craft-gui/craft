use std::fmt;
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
use crate::style::style_flags::StyleFlags;

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
    font_family_length: u8,
    font_family: [u8; 64],
    box_sizing: BoxSizing,
    scrollbar_width: f32,
    position: Position,
    margin: [Unit; 4],
    padding: [Unit; 4],
    gap: [Unit; 2],
    inset: [Unit; 4],
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
    overflow: [Overflow; 2],

    border_color: [Color; 4],
    border_width: [Unit; 4],
    border_radius: [(f32, f32); 4],

    pub dirty_flags: StyleFlags
}


impl Default for Style {
    
    fn default() -> Self {
        Style {
            font_family_length: 0,
            font_family: [0; 64],
            box_sizing: BoxSizing::BorderBox,
            scrollbar_width: if cfg!(any(taros = "android", taros = "ios")) {
                0.0
            } else {
                15.0
            },
            position: Position::Relative,
            margin: [Unit::Px(0.0); 4],
            padding: [Unit::Px(0.0); 4],
            border_width: [Unit::Px(0.0); 4],
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
            border_color: [Color::BLACK; 4],
            font_size: 16.0,
            font_weight: Default::default(),
            font_style: Default::default(),
            overflow: [Overflow::default(), Overflow::default()],
            border_radius: [(0.0, 0.0); 4],
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

    pub fn margin(&self) -> [Unit; 4] {
        self.margin
    }

    pub fn margin_mut(&mut self) -> &mut [Unit; 4] {
        self.dirty_flags.insert(StyleFlags::MARGIN);
        &mut self.margin
    }

    pub fn padding(&self) -> [Unit; 4] {
        self.padding
    }

    pub fn padding_mut(&mut self) -> &mut [Unit; 4] {
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

    pub fn inset(&self) -> [Unit; 4] {
        self.inset
    }

    pub fn inset_mut(&mut self) -> &mut [Unit; 4] {
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

    pub fn overflow(&self) -> [Overflow; 2] {
        self.overflow
    }

    pub fn overflow_mut(&mut self) -> &mut [Overflow; 2] {
        self.dirty_flags.insert(StyleFlags::OVERFLOW);
        &mut self.overflow
    }

    pub fn border_color(&self) -> [Color; 4] {
        self.border_color
    }

    pub fn border_color_mut(&mut self) -> &mut [Color; 4] {
        self.dirty_flags.insert(StyleFlags::BORDER_COLOR);
        &mut self.border_color
    }

    pub fn border_width(&self) -> [Unit; 4] {
        self.border_width
    }

    pub fn border_width_mut(&mut self) -> &mut [Unit; 4] {
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

}

