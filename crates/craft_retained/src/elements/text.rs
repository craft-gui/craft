use std::any::Any;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

use craft_primitives::Color;
use craft_primitives::geometry::{Point, Rectangle};
use craft_renderer::renderer::RenderList;
#[cfg(feature = "accesskit")]
use parley::LayoutAccessibility;
use parley::{Alignment, AlignmentOptions, ContentWidths, Selection};

use crate::elements::element_data::ElementData;
use crate::events::{CraftMessage, Event};
use crate::layout::layout_context::{LayoutContext, TaffyTextContext, TextHashKey};
use crate::style::Style;
use crate::text::text_context::TextContext;
use crate::text::text_render_data;
use crate::text::text_render_data::TextRender;

const MAX_CACHE_SIZE: usize = 16;

#[cfg(not(target_arch = "wasm32"))]
use std::time;

#[cfg(feature = "accesskit")]
use accesskit::{Action, Role};
use craft_primitives::ColorBrush;
use craft_renderer::text_renderer_data::TextData;
use kurbo::Affine;
use rustc_hash::FxHashMap;
use smol_str::{SmolStr, ToSmolStr};
use taffy::{AvailableSpace, Size};
use time::{Duration, Instant};
use ui_events::pointer::{PointerButton, PointerId};
#[cfg(target_arch = "wasm32")]
use web_time as time;
use winit::dpi;

use crate::elements::Element;
use crate::elements::core::ElementInternals;
#[cfg(feature = "accesskit")]
use crate::elements::element_id::create_unique_element_id;
use crate::layout::TaffyTree;

// A stateful element that shows text.
#[derive(Clone)]
pub struct Text {
    element_data: ElementData,
    selectable: bool,
    pub(crate) state: TextState,
    me: Weak<RefCell<Self>>,
}

#[derive(Clone)]
pub struct TextState {
    text: SmolStr,
    scale_factor: f64,
    selection: Selection,
    pub(crate) text_render: Option<TextRender>,
    layout: Option<parley::Layout<ColorBrush>>,
    cache: FxHashMap<TextHashKey, Size<f32>>,
    current_layout_key: Option<TextHashKey>,
    last_requested_measure_key: Option<TextHashKey>,
    current_render_key: Option<TextHashKey>,
    content_widths: Option<ContentWidths>,

    pub(crate) last_click_time: Option<Instant>,
    pub(crate) click_count: u32,
    pub(crate) pointer_down: bool,
    /// The last known cursor position.
    ///
    /// The cursor is assumed to start at (0.0, 0.0). The cursor_pos may return points
    /// outside the text input.
    cursor_pos: Point,
    pub(crate) start_time: Option<Instant>,
    pub(crate) blink_period: Duration,

    is_layout_dirty: bool,
    is_render_dirty: bool,
}

impl Text {
    pub fn new(text: &str) -> Rc<RefCell<Self>> {
        let me = Rc::new_cyclic(|me: &Weak<RefCell<Self>>| {
            RefCell::new(Self {
                element_data: ElementData::new(me.clone(), false),
                selectable: true,
                state: TextState::default(),
                me: me.clone(),
            })
        });

        let text_context = Some(LayoutContext::Text(TaffyTextContext {
            element: me.borrow().me.clone(),
        }));
        me.borrow_mut().element_data.create_layout_node(text_context);

        me.borrow_mut().text(text);

        me
    }

    pub fn get_selectable(&self) -> bool {
        self.selectable
    }

    pub fn selectable(&mut self, selectable: bool) -> &mut Self {
        self.selectable = selectable;
        self
    }

    pub fn get_text(&self) -> &str {
        &self.state.text
    }

    /// Set the text.
    ///
    /// Updates the text content immediately. Mark layout and render caches as dirty. Layout and
    /// render caches will be computed in the next layout/render pass.
    pub fn text(&mut self, text: &str) -> &mut Self {
        self.set_text_smol_str(text.to_smolstr());
        self
    }

    /// Set the text.
    ///
    /// Updates the text content immediately. Mark layout and render caches as dirty. Layout and
    /// render caches will be computed in the next layout/render pass.
    pub fn set_text_smol_str(&mut self, text: SmolStr) {
        self.state.text = text;
        self.state.is_layout_dirty = true;
        self.state.is_render_dirty = true;
        self.mark_dirty();
    }

    pub(crate) fn measure(
        &mut self,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
        text_context: &mut TextContext,
    ) -> Size<f32> {
        self.state.measure(
            &self.element_data.style,
            known_dimensions,
            available_space,
            text_context,
        )
    }
}

impl crate::elements::core::ElementData for Text {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl crate::elements::Element for Text {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl ElementInternals for Text {
    fn apply_layout(
        &mut self,
        taffy_tree: &mut TaffyTree,
        position: Point,
        z_index: &mut u32,
        transform: Affine,
        _pointer: Option<Point>,
        text_context: &mut TextContext,
        clip_bounds: Option<Rectangle>,
        scale_factor: f64,
    ) {
        let node = self.element_data.layout_item.taffy_node_id.unwrap();
        let result = taffy_tree.layout(node);
        let has_new_layout = taffy_tree.get_has_new_layout(node);

        let dirty = has_new_layout
            || transform != self.element_data.layout_item.get_transform()
            || position != self.element_data.layout_item.position;
        self.element_data.layout_item.has_new_layout = has_new_layout;
        if dirty {
            self.resolve_box(position, transform, result, z_index);
            self.apply_clip(clip_bounds);

            self.apply_borders(scale_factor);
        }

        if has_new_layout {
            taffy_tree.mark_seen(node);
        }

        let state: &mut TextState = &mut self.state;
        if state.current_layout_key != state.last_requested_measure_key {
            state.layout(
                state.last_requested_measure_key.unwrap().known_dimensions(),
                state.last_requested_measure_key.unwrap().available_space(),
            );
        }

        state.try_update_text_render(text_context, self.element_data.style.selection_color());
    }

    fn draw(
        &mut self,
        renderer: &mut RenderList,
        _text_context: &mut TextContext,
        _pointer: Option<Point>,
        scale_factor: f64,
    ) {
        if !self.is_visible() {
            return;
        }
        self.add_hit_testable(renderer, true, scale_factor);

        let computed_box_transformed = self.computed_box_transformed();
        let content_rectangle = computed_box_transformed.content_rectangle();

        self.draw_borders(renderer, scale_factor);

        /*if self.element_data.layout_item.has_new_layout {
            renderer.draw_rect_outline(self.element_data.layout_item.computed_box_transformed.padding_rectangle(), rgba(255, 0, 0, 100), 1.0);
        }*/

        renderer.draw_text(self.me.clone(), content_rectangle.scale(scale_factor), None, false);
    }

    #[cfg(feature = "accesskit")]
    fn compute_accessibility_tree(
        &mut self,
        tree: &mut accesskit::TreeUpdate,
        parent_index: Option<usize>,
        scale_factor: f64,
    ) {
        let padding_box = self
            .element_data
            .layout_item
            .computed_box_transformed
            .padding_rectangle()
            .scale(scale_factor);

        let state: &mut TextState = &mut self.state;
        if state.layout.is_none() {
            return;
        }

        let layout = state.layout.as_mut();
        let mut access = LayoutAccessibility::default();
        let text = state.text.as_ref();

        let current_node_id = accesskit::NodeId(self.element_data.internal_id);

        let mut current_node = accesskit::Node::new(Role::Label);
        current_node.set_value(text);
        current_node.add_action(Action::SetTextSelection);

        current_node.set_bounds(accesskit::Rect {
            x0: padding_box.left() as f64,
            y0: padding_box.top() as f64,
            x1: padding_box.right() as f64,
            y1: padding_box.bottom() as f64,
        });

        if let Some(layout) = layout {
            access.build_nodes(
                text,
                layout,
                tree,
                &mut current_node,
                || accesskit::NodeId(create_unique_element_id()),
                padding_box.x as f64,
                padding_box.y as f64,
            );
        }

        if let Some(parent_index) = parent_index {
            let parent_node = tree.nodes.get_mut(parent_index).unwrap();
            parent_node.1.push_child(current_node_id);
        }

        tree.nodes.push((current_node_id, current_node));
    }

    fn on_event(
        &mut self,
        message: &CraftMessage,
        _text_context: &mut TextContext,
        event: &mut Event,
        _target: Option<Rc<RefCell<dyn ElementInternals>>>,
    ) {
        if !self.selectable {
            return;
        }

        let _content_rect = self.computed_box().content_rectangle();

        // Handle selection.
        if self.selectable {
            let text_position = self.computed_box_transformed().content_rectangle();

            let state: &mut TextState = &mut self.state;
            match message {
                CraftMessage::PointerButtonDown(pointer_button) => {
                    if pointer_button
                        .button
                        .map(|button| button == PointerButton::Primary)
                        .unwrap_or_default()
                    {
                        state.update_text_selection(self.element_data.style.selection_color());
                        state.pointer_down = true;
                        state.cursor_reset();
                        let now = Instant::now();
                        if let Some(last) = state.last_click_time.take() {
                            if now.duration_since(last).as_secs_f64() < 0.25 {
                                state.click_count = (state.click_count + 1) % 4;
                            } else {
                                state.click_count = 1;
                            }
                        } else {
                            state.click_count = 1;
                        }
                        state.last_click_time = Some(now);
                        let click_count = state.click_count;
                        let cursor_pos = state.cursor_pos;
                        match click_count {
                            2 => state.select_word_at_point(cursor_pos),
                            3 => state.select_line_at_point(cursor_pos),
                            _ => state.move_to_point(cursor_pos),
                        }
                        if click_count == 1 {
                            self.set_pointer_capture(PointerId::new(1).unwrap());
                        }
                        event.prevent_defaults();
                    }
                }
                CraftMessage::PointerButtonUp(pointer_button) => {
                    if pointer_button
                        .button
                        .map(|button| button == PointerButton::Primary)
                        .unwrap_or_default()
                    {
                        state.update_text_selection(self.element_data.style.selection_color());
                        state.pointer_down = false;
                        state.cursor_reset();
                        self.release_pointer_capture(PointerId::new(1).unwrap());
                        event.prevent_defaults();
                    }
                }
                CraftMessage::PointerMovedEvent(pointer_moved) => {
                    let prev_pos = state.cursor_pos;
                    // NOTE: Cursor position should be relative to the top left of the text box.
                    state.cursor_pos = pointer_moved.current.logical_point()
                        - kurbo::Vec2::new(text_position.x as f64, text_position.y as f64);
                    // macOS seems to generate a spurious move after selecting word?
                    if state.pointer_down && prev_pos != state.cursor_pos {
                        state.update_text_selection(self.element_data.style.selection_color());
                        state.cursor_reset();
                        let cursor_pos = state.cursor_pos;
                        state.extend_selection_to_point(cursor_pos);
                    }
                    event.prevent_defaults();
                }
                _ => {}
            }
        }
    }

    fn scale_factor(&mut self, scale_factor: f64) {
        self.state.is_layout_dirty = true;
        self.state.is_render_dirty = true;
        self.mark_dirty();
        self.state.scale_factor = scale_factor;
    }
}

impl TextState {
    pub fn measure(
        &mut self,
        style: &Style,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
        text_context: &mut TextContext,
    ) -> Size<f32> {
        if self.is_layout_dirty {
            self.clear_cache();
        }

        if self.layout.is_none() {
            let mut builder = text_context.tree_builder(self.scale_factor as f32, &style.to_text_style());
            let text = &self.text;
            builder.push_text(text);
            let (layout, _) = builder.build();
            self.content_widths = Some(layout.calculate_content_widths());
            self.layout = Some(layout);
        }

        let key = TextHashKey::new(known_dimensions, available_space);

        self.last_requested_measure_key = Some(key);
        if let Some(value) = self.cache.get(&key) {
            return *value;
        }

        self.layout(known_dimensions, available_space)
    }

    pub fn layout(&mut self, known_dimensions: Size<Option<f32>>, available_space: Size<AvailableSpace>) -> Size<f32> {
        let key = TextHashKey::new(known_dimensions, available_space);

        let content_widths = self.content_widths.as_ref().unwrap();
        let width_constraint = known_dimensions.width.or(match available_space.width {
            AvailableSpace::MinContent => Some(content_widths.min),
            AvailableSpace::MaxContent => Some(content_widths.max),
            AvailableSpace::Definite(width) => {
                let scaled_width: f32 = dpi::PhysicalUnit::from_logical::<f32, f32>(width, self.scale_factor as f64).0;
                Some(scaled_width.clamp(content_widths.min, content_widths.max))
            }
        });

        let height_constraint = known_dimensions.height.or(match available_space.height {
            AvailableSpace::MinContent => None,
            AvailableSpace::MaxContent => None,
            AvailableSpace::Definite(height) => {
                let scaled_height = dpi::PhysicalUnit::from_logical::<f32, f32>(height, self.scale_factor as f64).0;
                Some(scaled_height)
            }
        });

        let layout = self.layout.as_mut().unwrap();
        layout.break_all_lines(width_constraint);
        layout.align(width_constraint, Alignment::Start, AlignmentOptions::default());

        let width = layout.width();
        let height = layout.height().min(height_constraint.unwrap_or(f32::MAX));

        let logical_width = dpi::LogicalUnit::from_physical::<f32, f32>(width, self.scale_factor as f64).0;
        let logical_height = dpi::LogicalUnit::from_physical::<f32, f32>(height, self.scale_factor as f64).0;

        let size = Size {
            width: logical_width,
            height: logical_height,
        };

        if self.cache.len() >= MAX_CACHE_SIZE {
            // TODO: Use LRU?
            let oldest_key = *self.cache.keys().next().unwrap();
            self.cache.remove(&oldest_key);
        }

        self.cache.insert(key, size);
        self.current_layout_key = Some(key);
        size
    }

    pub fn try_update_text_render(&mut self, _text_context: &mut TextContext, selection_color: Color) {
        if self.current_render_key == self.current_layout_key {
            return;
        }

        let layout = self.layout.as_ref().unwrap();
        self.text_render = Some(text_render_data::from_editor(layout));
        self.current_render_key = self.current_layout_key;

        self.update_text_selection(selection_color);
    }

    pub fn cursor_reset(&mut self) {
        self.start_time = Some(Instant::now());
        self.blink_period = Duration::from_millis(500);
    }

    pub fn extend_selection_to_point(&mut self, point: Point) {
        let scale_factor = self.layout.as_ref().unwrap().scale() as f64;
        let point = Point::new(point.x * scale_factor, point.y * scale_factor);
        self.selection = self
            .selection
            .extend_to_point(self.layout.as_ref().unwrap(), point.x as f32, point.y as f32);
    }

    pub fn select_word_at_point(&mut self, point: Point) {
        let scale_factor = self.layout.as_ref().unwrap().scale() as f64;
        let point = Point::new(point.x * scale_factor, point.y * scale_factor);
        self.selection = Selection::word_from_point(self.layout.as_ref().unwrap(), point.x as f32, point.y as f32);
    }

    pub fn select_line_at_point(&mut self, point: Point) {
        let scale_factor = self.layout.as_ref().unwrap().scale() as f64;
        let point = Point::new(point.x * scale_factor, point.y * scale_factor);
        self.selection = Selection::line_from_point(self.layout.as_ref().unwrap(), point.x as f32, point.y as f32);
    }

    pub fn move_to_point(&mut self, point: Point) {
        let scale_factor = self.layout.as_ref().unwrap().scale() as f64;
        let point = Point::new(point.x * scale_factor, point.y * scale_factor);
        self.selection = Selection::from_point(self.layout.as_ref().unwrap(), point.x as f32, point.y as f32);
    }

    pub fn clear_cache(&mut self) {
        self.layout = None;
        self.cache.clear();
        self.current_layout_key = None;
        self.last_requested_measure_key = None;
        self.current_render_key = None;
        self.text_render = None;
        self.content_widths = None;
        self.is_layout_dirty = false;
    }

    fn update_text_selection(&mut self, selection_color: Color) {
        if let Some(layout) = self.layout.as_ref() {
            let text_renderer = self.text_render.as_mut().unwrap();
            for line in text_renderer.lines.iter_mut() {
                line.selections.clear();
            }
            self.selection.geometry_with(layout, |rect, line| {
                text_renderer.lines[line].selections.push((
                    Rectangle::new(
                        rect.x0 as f32,
                        rect.y0 as f32,
                        rect.width() as f32,
                        rect.height() as f32,
                    ),
                    selection_color,
                ));
            });
        }
    }
}

impl TextData for Text {
    fn get_text_renderer(&self) -> Option<&TextRender> {
        self.state.text_render.as_ref()
    }
}

impl Default for TextState {
    fn default() -> Self {
        Self {
            text: SmolStr::new(""),
            scale_factor: 1.0,
            selection: Selection::default(),
            text_render: None,
            layout: None,
            cache: Default::default(),
            current_layout_key: None,
            last_requested_measure_key: None,
            current_render_key: None,
            content_widths: None,
            last_click_time: None,
            click_count: 0,
            pointer_down: false,
            cursor_pos: Point::new(0.0, 0.0),
            start_time: None,
            blink_period: Default::default(),
            is_layout_dirty: false,
            is_render_dirty: false,
        }
    }
}
