use std::borrow::Cow;
use std::fmt::Debug;

use crate::style::*;
use craft_primitives::geometry::TrblRectangle;
use craft_primitives::{Color, ColorBrush};

#[derive(Clone, Debug)]
pub struct Style {
    box_sizing: StyleProperty<BoxSizing>,
    position: StyleProperty<Position>,
    margin: StyleProperty<TrblRectangle<Unit>>,
    padding: StyleProperty<TrblRectangle<Unit>>,
    gap: StyleProperty<[Unit; 2]>,
    inset: StyleProperty<TrblRectangle<Unit>>,

    width: StyleProperty<Unit>,
    min_width: StyleProperty<Unit>,
    max_width: StyleProperty<Unit>,

    height: StyleProperty<Unit>,
    min_height: StyleProperty<Unit>,
    max_height: StyleProperty<Unit>,

    display: StyleProperty<Display>,
    wrap: StyleProperty<FlexWrap>,
    align_items: StyleProperty<Option<AlignItems>>,
    justify_content: StyleProperty<Option<JustifyContent>>,
    flex_direction: StyleProperty<FlexDirection>,
    flex_grow: StyleProperty<f32>,
    flex_shrink: StyleProperty<f32>,
    flex_basis: StyleProperty<Unit>,
    font_family: StyleProperty<FontFamily>,

    background_color: StyleProperty<Color>,
    color: StyleProperty<Color>,

    line_height: StyleProperty<f32>,
    font_size: StyleProperty<f32>,
    font_weight: StyleProperty<FontWeight>,
    font_style: StyleProperty<FontStyle>,
    underline: StyleProperty<Option<Underline>>,

    overflow: StyleProperty<[Overflow; 2]>,

    border_color: StyleProperty<TrblRectangle<Color>>,
    border_width: StyleProperty<TrblRectangle<Unit>>,
    border_radius: StyleProperty<[(f32, f32); 4]>,

    scrollbar_color: StyleProperty<ScrollbarColor>,
    scrollbar_thumb_margin: StyleProperty<TrblRectangle<f32>>,
    scrollbar_thumb_radius: StyleProperty<[(f32, f32); 4]>,
    scrollbar_width: StyleProperty<f32>,

    visible: StyleProperty<bool>,
    selection_color: StyleProperty<Color>,
    cursor_color: StyleProperty<Option<Color>>,

    /// Set to true anytime a setter is called.
    pub is_dirty: bool,
}
const SCROLLBAR_THUMB_MARGIN: TrblRectangle<f32> = if cfg!(any(target_os = "android", target_os = "ios")) {
    TrblRectangle::new_all(0.0)
} else {
    TrblRectangle::new(1.0, 2.0, 1.0, 2.0)
};

impl Style {
    pub(crate) fn new() -> Box<Self> {
        Box::new(Style {
            is_dirty: true,
            box_sizing: StyleProperty::new(BoxSizing::BorderBox),
            position: StyleProperty::new(Position::Relative),
            margin: StyleProperty::new(TrblRectangle::new_all(Unit::Px(0.0))),
            padding: StyleProperty::new(TrblRectangle::new_all(Unit::Px(0.0))),
            gap: StyleProperty::new([Unit::Px(0.0), Unit::Px(0.0)]),
            inset: StyleProperty::new(TrblRectangle::new_all(Unit::Px(0.0))),
            width: StyleProperty::new(Unit::Auto),
            min_width: StyleProperty::new(Unit::Auto),
            max_width: StyleProperty::new(Unit::Auto),
            height: StyleProperty::new(Unit::Auto),
            min_height: StyleProperty::new(Unit::Auto),
            max_height: StyleProperty::new(Unit::Auto),
            display: StyleProperty::new(Display::Flex),
            wrap: StyleProperty::new(FlexWrap::default()),
            align_items: StyleProperty::new(None),
            justify_content: StyleProperty::new(None),
            flex_direction: StyleProperty::new(FlexDirection::Row),
            flex_grow: StyleProperty::new(0.0),
            flex_shrink: StyleProperty::new(1.0),
            flex_basis: StyleProperty::new(Unit::Auto),
            font_family: StyleProperty::new(FontFamily::default()),
            background_color: StyleProperty::new(Color::TRANSPARENT),
            color: StyleProperty::new(Color::BLACK),
            line_height: StyleProperty::new(1.2),
            font_size: StyleProperty::new(16.0),
            font_weight: StyleProperty::new(FontWeight::default()),
            font_style: StyleProperty::new(FontStyle::default()),
            underline: StyleProperty::new(None),
            overflow: StyleProperty::new([Overflow::default(); 2]),
            border_color: StyleProperty::new(TrblRectangle::new_all(Color::BLACK)),
            border_width: StyleProperty::new(TrblRectangle::new_all(Unit::Px(0.0))),
            border_radius: StyleProperty::new([(0.0, 0.0); 4]),
            scrollbar_color: StyleProperty::new(ScrollbarColor {
                thumb_color: Color::from_rgb8(150, 150, 152),
                track_color: Color::TRANSPARENT
            }),
            scrollbar_thumb_margin: StyleProperty::new(SCROLLBAR_THUMB_MARGIN),
            scrollbar_thumb_radius: StyleProperty::new([(10.0, 10.0); 4]),
            scrollbar_width: StyleProperty::new(if cfg!(any(target_os = "android", target_os = "ios")) {
                0.0
            } else {
                10.0
            }),
            visible: StyleProperty::new(true),
            selection_color: StyleProperty::new(Color::from_rgb8(0, 120, 215)),
            cursor_color: StyleProperty::new(None),
        })
    }
}

impl Style {
    pub fn get_box_sizing(&self) -> BoxSizing {
        self.box_sizing.get()
    }
    pub fn set_box_sizing(&mut self, val: BoxSizing) {
        self.is_dirty = true;
        self.box_sizing.set(val);
    }

    pub fn get_position(&self) -> Position {
        self.position.get()
    }
    pub fn set_position(&mut self, val: Position) {
        self.is_dirty = true;
        self.position.set(val);
    }

    pub fn get_margin(&self) -> TrblRectangle<Unit> {
        self.margin.get()
    }
    pub fn set_margin(&mut self, val: TrblRectangle<Unit>) {
        self.is_dirty = true;
        self.margin.set(val);
    }

    pub fn get_padding(&self) -> TrblRectangle<Unit> {
        self.padding.get()
    }
    pub fn set_padding(&mut self, val: TrblRectangle<Unit>) {
        self.is_dirty = true;
        self.padding.set(val);
    }

    pub fn get_gap(&self) -> [Unit; 2] {
        self.gap.get()
    }
    pub fn set_gap(&mut self, val: [Unit; 2]) {
        self.is_dirty = true;
        self.gap.set(val);
    }

    pub fn get_inset(&self) -> TrblRectangle<Unit> {
        self.inset.get()
    }
    pub fn set_inset(&mut self, val: TrblRectangle<Unit>) {
        self.is_dirty = true;
        self.inset.set(val);
    }

    pub fn get_width(&self) -> Unit {
        self.width.get()
    }
    pub fn set_width(&mut self, val: Unit) {
        self.is_dirty = true;
        self.width.set(val);
    }

    pub fn get_min_width(&self) -> Unit {
        self.min_width.get()
    }
    pub fn set_min_width(&mut self, val: Unit) {
        self.is_dirty = true;
        self.min_width.set(val);
    }

    pub fn get_max_width(&self) -> Unit {
        self.max_width.get()
    }
    pub fn set_max_width(&mut self, val: Unit) {
        self.is_dirty = true;
        self.max_width.set(val);
    }

    pub fn get_height(&self) -> Unit {
        self.height.get()
    }
    pub fn set_height(&mut self, val: Unit) {
        self.is_dirty = true;
        self.height.set(val);
    }

    pub fn get_min_height(&self) -> Unit {
        self.min_height.get()
    }
    pub fn set_min_height(&mut self, val: Unit) {
        self.is_dirty = true;
        self.min_height.set(val);
    }

    pub fn get_max_height(&self) -> Unit {
        self.max_height.get()
    }
    pub fn set_max_height(&mut self, val: Unit) {
        self.is_dirty = true;
        self.max_height.set(val);
    }

    pub fn get_display(&self) -> Display {
        self.display.get()
    }
    pub fn set_display(&mut self, val: Display) {
        self.is_dirty = true;
        self.display.set(val);
    }

    pub fn get_wrap(&self) -> FlexWrap {
        self.wrap.get()
    }
    pub fn set_wrap(&mut self, val: FlexWrap) {
        self.is_dirty = true;
        self.wrap.set(val);
    }

    pub fn get_align_items(&self) -> Option<AlignItems> {
        self.align_items.get()
    }
    pub fn set_align_items(&mut self, val: Option<AlignItems>) {
        self.is_dirty = true;
        self.align_items.set(val);
    }

    pub fn get_justify_content(&self) -> Option<JustifyContent> {
        self.justify_content.get()
    }
    pub fn set_justify_content(&mut self, val: Option<JustifyContent>) {
        self.is_dirty = true;
        self.justify_content.set(val);
    }

    pub fn get_flex_direction(&self) -> FlexDirection {
        self.flex_direction.get()
    }
    pub fn set_flex_direction(&mut self, val: FlexDirection) {
        self.is_dirty = true;
        self.flex_direction.set(val);
    }

    pub fn get_flex_grow(&self) -> f32 {
        self.flex_grow.get()
    }
    pub fn set_flex_grow(&mut self, val: f32) {
        self.is_dirty = true;
        self.flex_grow.set(val);
    }

    pub fn get_flex_shrink(&self) -> f32 {
        self.flex_shrink.get()
    }
    pub fn set_flex_shrink(&mut self, val: f32) {
        self.is_dirty = true;
        self.flex_shrink.set(val);
    }

    pub fn get_flex_basis(&self) -> Unit {
        self.flex_basis.get()
    }
    pub fn set_flex_basis(&mut self, val: Unit) {
        self.is_dirty = true;
        self.flex_basis.set(val);
    }

    pub fn get_font_family(&self) -> FontFamily {
        self.font_family.get()
    }
    pub fn set_font_family(&mut self, val: FontFamily) {
        self.is_dirty = true;
        self.font_family.set(val);
    }

    pub fn get_color(&self) -> Color {
        self.color.get()
    }
    pub fn set_color(&mut self, val: Color) {
        self.is_dirty = true;
        self.color.set(val);
    }

    pub fn get_background_color(&self) -> Color {
        self.background_color.get()
    }
    pub fn set_background_color(&mut self, val: Color) {
        self.is_dirty = true;
        self.background_color.set(val);
    }

    pub fn get_font_size(&self) -> f32 {
        self.font_size.get()
    }
    pub fn set_font_size(&mut self, val: f32) {
        self.is_dirty = true;
        self.font_size.set(val);
    }

    pub fn get_line_height(&self) -> f32 {
        self.line_height.get()
    }
    pub fn set_line_height(&mut self, val: f32) {
        self.is_dirty = true;
        self.line_height.set(val);
    }

    pub fn get_font_weight(&self) -> FontWeight {
        self.font_weight.get()
    }
    pub fn set_font_weight(&mut self, val: FontWeight) {
        self.is_dirty = true;
        self.font_weight.set(val);
    }

    pub fn get_font_style(&self) -> FontStyle {
        self.font_style.get()
    }
    pub fn set_font_style(&mut self, val: FontStyle) {
        self.is_dirty = true;
        self.font_style.set(val);
    }

    pub fn get_underline(&self) -> Option<Underline> {
        self.underline.get()
    }
    pub fn set_underline(&mut self, val: Option<Underline>) {
        self.is_dirty = true;
        self.underline.set(val);
    }

    pub fn get_overflow(&self) -> [Overflow; 2] {
        self.overflow.get()
    }
    pub fn set_overflow(&mut self, val: [Overflow; 2]) {
        self.is_dirty = true;
        self.overflow.set(val);
    }

    pub fn get_border_color(&self) -> TrblRectangle<Color> {
        self.border_color.get()
    }
    pub fn set_border_color(&mut self, val: TrblRectangle<Color>) {
        self.is_dirty = true;
        self.border_color.set(val);
    }

    pub fn get_border_width(&self) -> TrblRectangle<Unit> {
        self.border_width.get()
    }
    pub fn set_border_width(&mut self, val: TrblRectangle<Unit>) {
        self.is_dirty = true;
        self.border_width.set(val);
    }

    pub fn get_border_radius(&self) -> [(f32, f32); 4] {
        self.border_radius.get()
    }
    pub fn set_border_radius(&mut self, val: [(f32, f32); 4]) {
        self.is_dirty = true;
        self.border_radius.set(val);
    }

    pub fn get_scrollbar_color(&self) -> ScrollbarColor {
        self.scrollbar_color.get()
    }
    pub fn set_scrollbar_color(&mut self, val: ScrollbarColor) {
        self.is_dirty = true;
        self.scrollbar_color.set(val);
    }

    pub fn get_scrollbar_thumb_margin(&self) -> TrblRectangle<f32> {
        self.scrollbar_thumb_margin.get()
    }
    pub fn set_scrollbar_thumb_margin(&mut self, val: TrblRectangle<f32>) {
        self.is_dirty = true;
        self.scrollbar_thumb_margin.set(val);
    }

    pub fn get_scrollbar_thumb_radius(&self) -> [(f32, f32); 4] {
        self.scrollbar_thumb_radius.get()
    }
    pub fn set_scrollbar_thumb_radius(&mut self, val: [(f32, f32); 4]) {
        self.is_dirty = true;
        self.scrollbar_thumb_radius.set(val);
    }

    pub fn get_scrollbar_width(&self) -> f32 {
        self.scrollbar_width.get()
    }
    pub fn set_scrollbar_width(&mut self, val: f32) {
        self.is_dirty = true;
        self.scrollbar_width.set(val);
    }

    pub fn get_visible(&self) -> bool {
        self.visible.get()
    }
    pub fn set_visible(&mut self, val: bool) {
        self.is_dirty = true;
        self.visible.set(val);
    }

    pub fn get_selection_color(&self) -> Color {
        self.selection_color.get()
    }
    pub fn set_selection_color(&mut self, val: Color) {
        self.is_dirty = true;
        self.selection_color.set(val);
    }

    pub fn get_cursor_color(&self) -> Option<Color> {
        self.cursor_color.get()
    }
    pub fn set_cursor_color(&mut self, val: Option<Color>) {
        self.is_dirty = true;
        self.cursor_color.set(val);
    }
}

impl Style {
    pub fn has_border(&self) -> bool {
        self.border_width.is_dirty() ||
        self.border_radius.is_dirty() ||
        self.border_color.is_dirty()
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_text_style(&'_ self) -> parley::TextStyle<'_, ColorBrush> {
        let font_size = self.get_font_size();
        let line_height = self.get_line_height();
        let font_weight = parley::FontWeight::new(self.get_font_weight().0 as f32);
        let font_style = match self.get_font_style() {
            FontStyle::Normal => parley::FontStyle::Normal,
            FontStyle::Italic => parley::FontStyle::Italic,
            // FIXME: Allow an angle when setting the obliqueness.
            FontStyle::Oblique => parley::FontStyle::Oblique(None),
        };
        let brush = ColorBrush {
            color: self.get_color(),
        };

        let font_stack_cow_list = if let Some(font_family) = self.get_font_family().name() {
            // Use the user-provided font and fallback to system UI fonts as needed.
            Cow::Owned(vec![
                parley::FontFamily::Named(Cow::Owned(font_family.to_string())),
                parley::FontFamily::Generic(parley::GenericFamily::SystemUi),
            ])
        } else {
            // Just default to system UI fonts.
            Cow::Owned(vec![parley::FontFamily::Generic(parley::GenericFamily::SystemUi)])
        };

        let underline = self.get_underline();
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
            text_wrap_mode: Default::default(),
        }
    }

    pub fn add_styles_to_style_set(&self, style_set: &mut parley::StyleSet<ColorBrush>) {
        let font_size = self.get_font_size();
        let line_height = self.get_line_height();
        let font_weight = parley::FontWeight::new(self.get_font_weight().0 as f32);
        let font_style = match self.get_font_style() {
            FontStyle::Normal => parley::FontStyle::Normal,
            FontStyle::Italic => parley::FontStyle::Italic,
            // FIXME: Allow an angle when setting the obliqueness.
            FontStyle::Oblique => parley::FontStyle::Oblique(None),
        };
        let brush = ColorBrush {
            color: self.get_color(),
        };

        let underline = self.get_underline();
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

        let font_family = self.get_font_family();
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

        style_set.insert(parley::StyleProperty::from(parley::FontStack::List(
            font_stack_cow_list,
        )));
        style_set.insert(parley::StyleProperty::FontSize(font_size));
        style_set.insert(parley::StyleProperty::FontStyle(font_style));
        style_set.insert(parley::StyleProperty::FontWeight(font_weight));
        style_set.insert(parley::StyleProperty::Brush(brush));
        style_set.insert(parley::StyleProperty::LineHeight(parley::LineHeight::FontSizeRelative(
            line_height,
        )));
        style_set.insert(parley::StyleProperty::Underline(has_underline));
        style_set.insert(parley::StyleProperty::UnderlineBrush(underline_brush));
        style_set.insert(parley::StyleProperty::UnderlineOffset(underline_offset));
        style_set.insert(parley::StyleProperty::UnderlineSize(underline_size));
    }
}