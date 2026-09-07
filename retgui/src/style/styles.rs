use std::borrow::Cow;
use std::fmt::Debug;

use retgui_primitives::Color;
use retgui_primitives::brush::Brush;
use retgui_primitives::geometry::TrblRectangle;

use crate::style::box_shadow::BoxShadow;
use crate::style::*;

/// An enum of all styles.
#[derive(Clone, Debug)]
pub enum StyleVariant {
    BoxSizing(BoxSizing),
    Position(Position),
    Margin(TrblRectangle<Unit>),
    Padding(TrblRectangle<Unit>),
    Gap([Unit; 2]),
    Inset(TrblRectangle<Unit>),

    Width(Unit),
    MinWidth(Unit),
    MaxWidth(Unit),

    Height(Unit),
    MinHeight(Unit),
    MaxHeight(Unit),

    Display(Display),
    Wrap(FlexWrap),
    AlignItems(AlignItems),
    AlignSelf(AlignSelf),
    AlignContent(AlignContent),
    JustifyContent(JustifyContent),
    FlexDirection(FlexDirection),
    FlexGrow(f32),
    FlexShrink(f32),
    FlexBasis(Unit),
    Order(i32),
    FontFamily(FontFamily),

    BackgroundBrush(Brush),
    TextBrush(Brush),

    LineHeight(f32),
    FontSize(f32),
    FontWeight(FontWeight),
    FontStyle(FontStyle),
    TextAlign(TextAlign),
    Underline(Option<Underline>),

    Overflow([Overflow; 2]),

    BorderColor(TrblRectangle<Color>),
    BorderWidth(TrblRectangle<Unit>),
    BorderRadius([(f32, f32); 4]),
    OutlineColor(TrblRectangle<Color>),
    OutlineWidth(TrblRectangle<Unit>),

    ScrollbarBrush(ScrollbarColor),
    ScrollbarThumbMargin(TrblRectangle<f32>),
    ScrollbarThumbRadius([(f32, f32); 4]),
    ScrollbarWidth(f32),

    Overlay(bool),
    Visible(bool),
    SelectionBrush(Brush),
    CursorBrush(Option<Brush>),

    BoxShadows(Vec<BoxShadow>),
}

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
    align_items: StyleProperty<AlignItems>,
    align_self: StyleProperty<AlignSelf>,
    align_content: StyleProperty<AlignContent>,
    justify_content: StyleProperty<JustifyContent>,
    flex_direction: StyleProperty<FlexDirection>,
    flex_grow: StyleProperty<f32>,
    flex_shrink: StyleProperty<f32>,
    flex_basis: StyleProperty<Unit>,
    order: StyleProperty<i32>,
    font_family: StyleProperty<FontFamily>,

    background_brush: StyleProperty<Brush>,
    text_brush: StyleProperty<Brush>,

    line_height: StyleProperty<f32>,
    font_size: StyleProperty<f32>,
    font_weight: StyleProperty<FontWeight>,
    font_style: StyleProperty<FontStyle>,
    text_align: StyleProperty<TextAlign>,
    underline: StyleProperty<Option<Underline>>,

    overflow: StyleProperty<[Overflow; 2]>,

    border_color: StyleProperty<TrblRectangle<Color>>,
    border_width: StyleProperty<TrblRectangle<Unit>>,
    border_radius: StyleProperty<[(f32, f32); 4]>,
    outline_color: StyleProperty<TrblRectangle<Color>>,
    outline_width: StyleProperty<TrblRectangle<Unit>>,

    scrollbar_brush: StyleProperty<ScrollbarColor>,
    scrollbar_thumb_margin: StyleProperty<TrblRectangle<f32>>,
    scrollbar_thumb_radius: StyleProperty<[(f32, f32); 4]>,
    scrollbar_width: StyleProperty<f32>,

    overlay: StyleProperty<bool>,
    visible: StyleProperty<bool>,
    selection_brush: StyleProperty<Brush>,
    cursor_brush: StyleProperty<Option<Brush>>,

    box_shadows: StyleProperty<Vec<BoxShadow>>,

    /// Set to true anytime a setter is called.
    pub is_dirty: bool,
}
const SCROLLBAR_THUMB_MARGIN: TrblRectangle<f32> = if cfg!(any(target_os = "android", target_os = "ios")) {
    TrblRectangle::new(0.0, 0.0, 0.0, 0.0)
} else {
    TrblRectangle::new(1.0, 2.0, 1.0, 2.0)
};

impl Style {
    pub(crate) fn new() -> Self {
        Style {
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
            align_items: StyleProperty::new(AlignItems::default()),
            align_self: StyleProperty::new(AlignSelf::default()),
            align_content: StyleProperty::new(AlignContent::default()),
            justify_content: StyleProperty::new(JustifyContent::default()),
            flex_direction: StyleProperty::new(FlexDirection::Row),
            flex_grow: StyleProperty::new(0.0),
            flex_shrink: StyleProperty::new(1.0),
            flex_basis: StyleProperty::new(Unit::Auto),
            order: StyleProperty::new(0),
            font_family: StyleProperty::new(FontFamily::default()),
            background_brush: StyleProperty::new(Brush::Color(Color::TRANSPARENT)),
            text_brush: StyleProperty::new(Brush::Color(Color::BLACK)),
            line_height: StyleProperty::new(1.2),
            font_size: StyleProperty::new(16.0),
            font_weight: StyleProperty::new(FontWeight::default()),
            font_style: StyleProperty::new(FontStyle::default()),
            text_align: StyleProperty::new(TextAlign::default()),
            underline: StyleProperty::new(None),
            overflow: StyleProperty::new([Overflow::default(); 2]),
            border_color: StyleProperty::new(TrblRectangle::new_all(Color::BLACK)),
            border_width: StyleProperty::new(TrblRectangle::new_all(Unit::Px(0.0))),
            border_radius: StyleProperty::new([(0.0, 0.0); 4]),
            outline_color: StyleProperty::new(TrblRectangle::new_all(Color::BLACK)),
            outline_width: StyleProperty::new(TrblRectangle::new_all(Unit::Px(0.0))),
            scrollbar_brush: StyleProperty::new(ScrollbarColor {
                thumb_color: Brush::Color(Color::from_rgb8(150, 150, 152)),
                track_color: Brush::Color(Color::TRANSPARENT),
            }),
            scrollbar_thumb_margin: StyleProperty::new(SCROLLBAR_THUMB_MARGIN),
            scrollbar_thumb_radius: StyleProperty::new([(10.0, 10.0); 4]),
            scrollbar_width: StyleProperty::new(if cfg!(any(target_os = "android", target_os = "ios")) {
                0.0
            } else {
                10.0
            }),
            overlay: StyleProperty::new(false),
            visible: StyleProperty::new(true),
            selection_brush: StyleProperty::new(Brush::Color(Color::from_rgb8(0, 120, 215))),
            cursor_brush: StyleProperty::new(None),
            box_shadows: StyleProperty::new(Vec::new()),
        }
    }

    pub fn get_box_sizing(&self) -> BoxSizing {
        *self.box_sizing.get()
    }

    pub fn set_box_sizing(&mut self, val: BoxSizing) {
        self.is_dirty = true;
        self.box_sizing.set(val);
    }

    pub fn get_position(&self) -> Position {
        *self.position.get()
    }

    pub fn set_position(&mut self, val: Position) {
        self.is_dirty = true;
        self.position.set(val);
    }

    pub fn get_margin(&self) -> TrblRectangle<Unit> {
        *self.margin.get()
    }

    pub fn set_margin(&mut self, val: TrblRectangle<Unit>) {
        self.is_dirty = true;
        self.margin.set(val);
    }

    pub fn get_padding(&self) -> TrblRectangle<Unit> {
        *self.padding.get()
    }

    pub fn set_padding(&mut self, val: TrblRectangle<Unit>) {
        self.is_dirty = true;
        self.padding.set(val);
    }

    pub fn get_gap(&self) -> [Unit; 2] {
        *self.gap.get()
    }

    pub fn set_gap(&mut self, val: [Unit; 2]) {
        self.is_dirty = true;
        self.gap.set(val);
    }

    pub fn get_inset(&self) -> TrblRectangle<Unit> {
        *self.inset.get()
    }

    pub fn set_inset(&mut self, val: TrblRectangle<Unit>) {
        self.is_dirty = true;
        self.inset.set(val);
    }

    pub fn get_width(&self) -> Unit {
        *self.width.get()
    }

    pub fn set_width(&mut self, val: Unit) {
        self.is_dirty = true;
        self.width.set(val);
    }

    pub fn get_min_width(&self) -> Unit {
        *self.min_width.get()
    }

    pub fn set_min_width(&mut self, val: Unit) {
        self.is_dirty = true;
        self.min_width.set(val);
    }

    pub fn get_max_width(&self) -> Unit {
        *self.max_width.get()
    }

    pub fn set_max_width(&mut self, val: Unit) {
        self.is_dirty = true;
        self.max_width.set(val);
    }

    pub fn get_height(&self) -> Unit {
        *self.height.get()
    }

    pub fn set_height(&mut self, val: Unit) {
        self.is_dirty = true;
        self.height.set(val);
    }

    pub fn get_min_height(&self) -> Unit {
        *self.min_height.get()
    }

    pub fn set_min_height(&mut self, val: Unit) {
        self.is_dirty = true;
        self.min_height.set(val);
    }

    pub fn get_max_height(&self) -> Unit {
        *self.max_height.get()
    }

    pub fn set_max_height(&mut self, val: Unit) {
        self.is_dirty = true;
        self.max_height.set(val);
    }

    pub fn get_display(&self) -> Display {
        *self.display.get()
    }

    pub fn set_display(&mut self, val: Display) {
        self.is_dirty = true;
        self.display.set(val);
    }

    pub fn get_wrap(&self) -> FlexWrap {
        *self.wrap.get()
    }

    pub fn set_wrap(&mut self, val: FlexWrap) {
        self.is_dirty = true;
        self.wrap.set(val);
    }

    pub fn get_align_items(&self) -> AlignItems {
        *self.align_items.get()
    }

    pub fn set_align_items(&mut self, val: AlignItems) {
        self.is_dirty = true;
        self.align_items.set(val);
    }

    pub fn get_align_self(&self) -> AlignSelf {
        *self.align_self.get()
    }

    pub fn set_align_self(&mut self, val: AlignSelf) {
        self.is_dirty = true;
        self.align_self.set(val);
    }

    pub fn get_align_content(&self) -> AlignContent {
        *self.align_content.get()
    }

    pub fn set_align_content(&mut self, val: AlignContent) {
        self.is_dirty = true;
        self.align_content.set(val);
    }

    pub fn get_justify_content(&self) -> JustifyContent {
        *self.justify_content.get()
    }

    pub fn set_justify_content(&mut self, val: JustifyContent) {
        self.is_dirty = true;
        self.justify_content.set(val);
    }

    pub fn get_flex_direction(&self) -> FlexDirection {
        *self.flex_direction.get()
    }

    pub fn set_flex_direction(&mut self, val: FlexDirection) {
        self.is_dirty = true;
        self.flex_direction.set(val);
    }

    pub fn get_flex_grow(&self) -> f32 {
        *self.flex_grow.get()
    }

    pub fn set_flex_grow(&mut self, val: f32) {
        self.is_dirty = true;
        self.flex_grow.set(val);
    }

    pub fn get_flex_shrink(&self) -> f32 {
        *self.flex_shrink.get()
    }

    pub fn set_flex_shrink(&mut self, val: f32) {
        self.is_dirty = true;
        self.flex_shrink.set(val);
    }

    pub fn get_flex_basis(&self) -> Unit {
        *self.flex_basis.get()
    }

    pub fn set_flex_basis(&mut self, val: Unit) {
        self.is_dirty = true;
        self.flex_basis.set(val);
    }

    pub fn get_order(&self) -> i32 {
        *self.order.get()
    }

    pub fn set_order(&mut self, val: i32) {
        self.is_dirty = true;
        self.order.set(val);
    }

    pub fn get_font_family(&self) -> FontFamily {
        *self.font_family.get()
    }

    pub fn set_font_family(&mut self, val: FontFamily) {
        self.is_dirty = true;
        self.font_family.set(val);
    }

    pub fn get_text_brush(&self) -> Brush {
        self.text_brush.get().clone()
    }

    pub fn set_text_brush(&mut self, val: Brush) {
        self.is_dirty = true;
        self.text_brush.set(val);
    }

    pub fn get_background_brush(&self) -> Brush {
        self.background_brush.get().clone()
    }

    pub fn set_background_brush(&mut self, val: Brush) {
        self.is_dirty = true;
        self.background_brush.set(val);
    }

    pub fn get_font_size(&self) -> f32 {
        *self.font_size.get()
    }

    pub fn set_font_size(&mut self, val: f32) {
        self.is_dirty = true;
        self.font_size.set(val);
    }

    pub fn get_line_height(&self) -> f32 {
        *self.line_height.get()
    }

    pub fn set_line_height(&mut self, val: f32) {
        self.is_dirty = true;
        self.line_height.set(val);
    }

    pub fn get_font_weight(&self) -> FontWeight {
        *self.font_weight.get()
    }

    pub fn set_font_weight(&mut self, val: FontWeight) {
        self.is_dirty = true;
        self.font_weight.set(val);
    }

    pub fn get_font_style(&self) -> FontStyle {
        *self.font_style.get()
    }

    pub fn set_font_style(&mut self, val: FontStyle) {
        self.is_dirty = true;
        self.font_style.set(val);
    }

    pub fn get_text_align(&self) -> TextAlign {
        *self.text_align.get()
    }

    pub fn set_text_align(&mut self, val: TextAlign) {
        self.is_dirty = true;
        self.text_align.set(val);
    }

    pub fn get_underline(&self) -> Option<Underline> {
        self.underline.get().clone()
    }

    pub fn set_underline(&mut self, val: Option<Underline>) {
        self.is_dirty = true;
        self.underline.set(val);
    }

    pub fn get_overflow(&self) -> [Overflow; 2] {
        *self.overflow.get()
    }

    pub fn set_overflow(&mut self, val: [Overflow; 2]) {
        self.is_dirty = true;
        self.overflow.set(val);
    }

    pub fn get_border_color(&self) -> TrblRectangle<Color> {
        *self.border_color.get()
    }

    pub fn set_border_color(&mut self, val: TrblRectangle<Color>) {
        self.is_dirty = true;
        self.border_color.set(val);
    }

    pub fn get_border_width(&self) -> TrblRectangle<Unit> {
        *self.border_width.get()
    }

    pub fn set_border_width(&mut self, val: TrblRectangle<Unit>) {
        self.is_dirty = true;
        self.border_width.set(val);
    }

    pub fn get_border_radius(&self) -> [(f32, f32); 4] {
        *self.border_radius.get()
    }

    pub fn set_border_radius(&mut self, val: [(f32, f32); 4]) {
        self.is_dirty = true;
        self.border_radius.set(val);
    }

    pub fn get_outline_color(&self) -> TrblRectangle<Color> {
        *self.outline_color.get()
    }

    pub fn set_outline_color(&mut self, val: TrblRectangle<Color>) {
        self.outline_color.set(val);
    }

    pub fn get_outline_width(&self) -> TrblRectangle<Unit> {
        *self.outline_width.get()
    }

    pub(crate) fn get_outline_width_px(&self) -> TrblRectangle<f32> {
        let width = self.get_outline_width();
        TrblRectangle::new(
            outline_unit_to_px(width.top),
            outline_unit_to_px(width.right),
            outline_unit_to_px(width.bottom),
            outline_unit_to_px(width.left),
        )
    }

    pub fn set_outline_width(&mut self, val: TrblRectangle<Unit>) {
        self.outline_width.set(val);
    }

    pub fn get_scrollbar_brush(&self) -> ScrollbarColor {
        self.scrollbar_brush.get().clone()
    }

    pub fn set_scrollbar_brush(&mut self, val: ScrollbarColor) {
        self.is_dirty = true;
        self.scrollbar_brush.set(val);
    }

    pub fn get_scrollbar_thumb_margin(&self) -> TrblRectangle<f32> {
        *self.scrollbar_thumb_margin.get()
    }

    pub fn set_scrollbar_thumb_margin(&mut self, val: TrblRectangle<f32>) {
        self.is_dirty = true;
        self.scrollbar_thumb_margin.set(val);
    }

    pub fn get_scrollbar_thumb_radius(&self) -> [(f32, f32); 4] {
        *self.scrollbar_thumb_radius.get()
    }

    pub fn set_scrollbar_thumb_radius(&mut self, val: [(f32, f32); 4]) {
        self.is_dirty = true;
        self.scrollbar_thumb_radius.set(val);
    }

    pub fn get_scrollbar_width(&self) -> f32 {
        *self.scrollbar_width.get()
    }

    pub fn set_scrollbar_width(&mut self, val: f32) {
        self.is_dirty = true;
        self.scrollbar_width.set(val);
    }

    pub fn get_overlay(&self) -> bool {
        *self.overlay.get()
    }

    pub fn set_overlay(&mut self, val: bool) {
        self.is_dirty = true;
        self.overlay.set(val);
    }

    pub fn get_visible(&self) -> bool {
        *self.visible.get()
    }

    pub fn set_visible(&mut self, val: bool) {
        self.is_dirty = true;
        self.visible.set(val);
    }

    pub fn get_selection_brush(&self) -> Brush {
        self.selection_brush.get().clone()
    }

    pub fn set_selection_brush(&mut self, val: Brush) {
        self.is_dirty = true;
        self.selection_brush.set(val);
    }

    pub fn get_cursor_brush(&self) -> Option<Brush> {
        self.cursor_brush.get().clone()
    }

    pub fn set_cursor_brush(&mut self, val: Option<Brush>) {
        self.is_dirty = true;
        self.cursor_brush.set(val);
    }

    pub fn get_box_shadows(&self) -> &[BoxShadow] {
        self.box_shadows.get()
    }

    pub fn set_box_shadows(&mut self, box_shadows: Vec<BoxShadow>) {
        self.box_shadows = StyleProperty::new(box_shadows)
    }

    pub fn has_border(&self) -> bool {
        self.border_width.is_dirty() || self.border_radius.is_dirty() || self.border_color.is_dirty()
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_text_style(&'_ self) -> parley::TextStyle<'_, '_, Brush> {
        let font_size = self.get_font_size();
        let line_height = self.get_line_height();
        let font_weight = parley::FontWeight::new(self.get_font_weight().0 as f32);
        let font_style = match self.get_font_style() {
            FontStyle::Normal => parley::FontStyle::Normal,
            FontStyle::Italic => parley::FontStyle::Italic,
            // FIXME: Allow an angle when setting the obliqueness.
            FontStyle::Oblique => parley::FontStyle::Oblique(None),
        };

        let brush = self.get_text_brush();

        let font_stack_cow_list = if let Some(font_family) = self.get_font_family().name() {
            // Use the user-provided font and fallback to system UI fonts as needed.
            Cow::Owned(vec![
                parley::FontFamilyName::named(font_family).into_owned(),
                parley::FontFamilyName::Generic(GenericFamily::SystemUi),
            ])
        } else {
            // Just default to system UI fonts.
            Cow::Owned(vec![parley::FontFamilyName::Generic(GenericFamily::SystemUi)])
        };

        let underline = self.get_underline();
        let has_underline = underline.is_some();
        let mut underline_offset = None;
        let mut underline_size = None;
        let mut underline_brush = None;

        if let Some(underline) = underline {
            underline_offset = underline.offset;
            underline_size = underline.thickness;
            underline_brush = Some(underline.brush);
        }

        let font_family = parley::FontFamily::List(font_stack_cow_list);
        parley::TextStyle {
            font_family,
            font_size,
            font_width: Default::default(),
            font_style,
            font_weight,
            font_variations: parley::FontVariations::List(Cow::Borrowed(&[])),
            font_features: parley::FontFeatures::List(Cow::Borrowed(&[])),
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

    pub fn add_styles_to_style_set<'a>(&'a self, style_set: &'a mut parley::StyleSet<Brush>) {
        let font_size = self.get_font_size();
        let line_height = self.get_line_height();
        let font_weight = parley::FontWeight::new(self.get_font_weight().0 as f32);
        let font_style = match self.get_font_style() {
            FontStyle::Normal => parley::FontStyle::Normal,
            FontStyle::Italic => parley::FontStyle::Italic,
            // FIXME: Allow an angle when setting the obliqueness.
            FontStyle::Oblique => parley::FontStyle::Oblique(None),
        };
        let brush = self.get_text_brush();

        let underline = self.get_underline();
        let has_underline = underline.is_some();
        let mut underline_offset = None;
        let mut underline_size = None;
        let mut underline_brush = None;

        if let Some(underline) = underline {
            underline_offset = underline.offset;
            underline_size = underline.thickness;
            underline_brush = Some(underline.brush);
        }

        let font_family = self.get_font_family();
        let font_stack_cow_list = if let Some(font_family) = font_family.name() {
            // Use the user-provided font and fallback to system UI fonts as needed.
            Cow::Owned(vec![
                parley::FontFamilyName::named(font_family).into_owned(),
                parley::FontFamilyName::Generic(GenericFamily::SystemUi),
            ])
        } else {
            // Just default to system UI fonts.
            Cow::Owned(vec![parley::FontFamilyName::Generic(parley::GenericFamily::SystemUi)])
        };

        style_set.insert(parley::StyleProperty::from(parley::FontFamily::List(
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

fn outline_unit_to_px(unit: Unit) -> f32 {
    match unit {
        Unit::Px(value) => value.max(0.0),
        Unit::Percentage(_) | Unit::Auto => 0.0,
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
