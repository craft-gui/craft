use std::any::Any;
use crate::elements::element_data::ElementData;
use crate::events::{CraftMessage, Event};
use crate::layout::layout_context::{LayoutContext, TaffyTextContext, TextHashKey};
use crate::style::Style;
use crate::text::text_context::TextContext;
use crate::text::text_render_data;
use crate::text::text_render_data::TextRender;
use craft_primitives::geometry::{Point, Rectangle};
use craft_renderer::renderer::RenderList;
use parley::{Alignment, AlignmentOptions, ContentWidths, Selection};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::sync::Arc;
#[cfg(feature = "accesskit")]
use parley::LayoutAccessibility;

#[cfg(feature = "accesskit")]
use accesskit::{Action, Role};
use kurbo::Affine;
#[cfg(not(target_arch = "wasm32"))]
use std::time;
use taffy::{AvailableSpace, NodeId, Size, TaffyTree};
use time::{Duration, Instant};
use winit::dpi;
#[cfg(target_arch = "wasm32")]
use web_time as time;
use crate::elements::core::ElementData as ElementDataTrait;
use crate::elements::core::ElementInternals;
use crate::elements::element_id::create_unique_element_id;
use crate::elements::Element;
use craft_primitives::ColorBrush;
use craft_renderer::text_renderer_data::TextData;
use smol_str::{SmolStr, ToSmolStr};
use ui_events::pointer::{PointerButton, PointerId};
use winit::window::Window;

// A stateful element that shows text.
#[derive(Clone, Default)]
pub struct Text {
    element_data: ElementData,
    selectable: bool,
    pub(crate) state: TextState,
    me: Option<Weak<RefCell<Self>>>,
}

#[derive(Clone, Default)]
pub struct TextState {
    text: SmolStr,
    scale_factor: f32,
    selection: Selection,
    pub(crate) text_render: Option<TextRender>,
    layout: Option<parley::Layout<ColorBrush>>,
    cache: HashMap<TextHashKey, Size<f32>>,
    current_layout_key: Option<TextHashKey>,
    last_requested_measure_key: Option<TextHashKey>,
    current_render_key: Option<TextHashKey>,
    content_widths: Option<ContentWidths>,

    pub(crate) last_click_time: Option<Instant>,
    pub(crate) click_count: u32,
    pub(crate) pointer_down: bool,
    pub(crate) cursor_pos: Point,
    pub(crate) start_time: Option<Instant>,
    pub(crate) blink_period: Duration,

    is_layout_dirty: bool,
    is_render_dirty: bool,
}

impl Text {
    pub fn new(text: &str) -> Rc<RefCell<Self>> {
        let text_state = TextState {
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
        };
        let me = Rc::new(RefCell::new(Text {
            element_data: Default::default(),
            selectable: true,
            state: text_state,
            me: None,
        }));
        let me2 = me.clone();
        me.borrow_mut().me = Some(Rc::downgrade(&me2));

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

    fn draw(
        &mut self,
        renderer: &mut RenderList,
        _text_context: &mut TextContext,
        _pointer: Option<Point>,
        _window: Option<Arc<Window>>,
        scale_factor: f64,
    ) {
        if !self.element_data.style.visible() {
            return;
        }
        let computed_box_transformed = self.computed_box_transformed();
        let content_rectangle = computed_box_transformed.content_rectangle();

        self.draw_borders(renderer, scale_factor);

        if self.state.text_render.as_ref().is_some() {
            renderer.draw_text(self.me.clone().unwrap(), content_rectangle.scale(scale_factor), None, false);
        }
    }

    #[cfg(feature = "accesskit")]
    fn compute_accessibility_tree(
        &mut self,
        tree: &mut accesskit::TreeUpdate,
        parent_index: Option<usize>,
        scale_factor: f64,
    ) {
        let padding_box = self.element_data().layout_item.computed_box_transformed.padding_rectangle().scale(scale_factor);

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

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        _scale_factor: f64,
    ) -> Option<NodeId> {
        if self.state.is_layout_dirty {
            self.state.layout = None;
            self.state.cache.clear();
            self.state.current_layout_key = None;
            self.state.last_requested_measure_key = None;
            self.state.current_render_key = None;
            self.state.text_render = None;
            self.state.content_widths = None;
        }
        //self.merge_default_style();

        let style: taffy::Style = self.element_data.style.to_taffy_style();

        self.element_data.layout_item.build_tree_with_context(
            taffy_tree,
            style,
            LayoutContext::Text(TaffyTextContext{
                element: self.me.clone().unwrap()
            }),
        )
    }

    fn finalize_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        position: Point,
        z_index: &mut u32,
        transform: Affine,
        _pointer: Option<Point>,
        text_context: &mut TextContext,
        clip_bounds: Option<Rectangle>,
    ) {
        let result = taffy_tree.layout(root_node).unwrap();
        self.resolve_box(position, transform, result, z_index);
        self.resolve_clip(clip_bounds);

        self.finalize_borders();

        let state: &mut TextState = &mut self.state;
        if state.current_layout_key != state.last_requested_measure_key {
            state.layout(
                state.last_requested_measure_key.unwrap().known_dimensions(),
                state.last_requested_measure_key.unwrap().available_space(),
            );
        }

        state.try_update_text_render(text_context);

        let layout = state.layout.as_ref().unwrap();
        let text_renderer = state.text_render.as_mut().unwrap();
        for line in text_renderer.lines.iter_mut() {
            line.selections.clear();
        }
        state.selection.geometry_with(layout, |rect, line| {
            text_renderer.lines[line].selections.push((Rectangle::new(rect.x0 as f32, rect.y0 as f32, rect.width() as f32, rect.height() as f32), self.element_data.style.selection_color()));
        });
    }

    fn on_event(
        &mut self,
        message: &CraftMessage,
        _text_context: &mut TextContext,
        event: &mut Event,
        _target: Option<Rc<RefCell<dyn ElementInternals>>>,
    ) {
        //self.on_style_event(message, should_style, event);
        //self.maybe_unset_focus(message, event, target);

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
                    if pointer_button.button.map(|button| button == PointerButton::Primary).unwrap_or_default() {
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
                    if pointer_button.button.map(|button| button == PointerButton::Primary).unwrap_or_default() {
                        state.pointer_down = false;
                        state.cursor_reset();
                        self.release_pointer_capture(PointerId::new(1).unwrap());
                        event.prevent_defaults();
                    }
                }
                CraftMessage::PointerMovedEvent(pointer_moved) => {
                    let prev_pos = state.cursor_pos;
                    // NOTE: Cursor position should be relative to the top left of the text box.
                    state.cursor_pos = pointer_moved.current.logical_point() - kurbo::Vec2::new(text_position.x as f64, text_position.y as f64);
                    // macOS seems to generate a spurious move after selecting word?
                    if state.pointer_down && prev_pos != state.cursor_pos {
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

    /*fn resolve_clip(&mut self, clip_bounds: Option<Rectangle>) {
        resolve_clip_for_scrollable(self, clip_bounds);
    }*/

/*    fn initialize_state(&mut self, scaling_factor: f64) -> ElementStateStoreItem {
        let hash = hash_string(self.text.as_ref().unwrap());
        let text_state = TextState {
            scale_factor: scaling_factor as f32,
            selection: Selection::default(),
            text: std::mem::take(&mut self.text),
            text_hash: Some(hash),
            text_render: None,
            last_text_style: self.style().clone(),
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
        };

        ElementStateStoreItem {
            base: Default::default(),
            data: Box::new(text_state),
        }
    }*/

  /*  #[cfg(feature = "accesskit")]
    fn compute_accessibility_tree(
        &mut self,
        tree: &mut accesskit::TreeUpdate,
        parent_index: Option<usize>,
        scale_factor: f64,
    ) {
        let state: &mut TextState = &mut self.state;
        if state.layout.is_none() {
            return;
        }

        let layout = state.layout.as_mut();
        let mut access = LayoutAccessibility::default();
        let text = state.text.as_ref().unwrap();

        let current_node_id = accesskit::NodeId(self.element_data().component_id);

        let mut current_node = accesskit::Node::new(Role::Label);
        let padding_box = self.element_data().layout_item.computed_box_transformed.padding_rectangle().scale(scale_factor);
        current_node.set_value(state.text.as_deref().unwrap());
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
    }*/

 /*   fn update_state(&mut self, reload_fonts: bool, scaling_factor: f64) {
        let text_hash = hash_string(self.text.as_ref().unwrap());
        let (state, base_state) = self.state_and_base_mut(element_state);

        let scale_factor_changed = if let Some(layout) = &state.layout {
            if layout.scale() != scaling_factor as f32 {
                state.scale_factor = scaling_factor as f32;
                true
            } else {
                false
            }
        } else {
            false
        };

        let last_style = &state.last_text_style;

        let current_style = base_state.current_style(self.element_data()).clone();
        if last_style.color() != current_style.color() {
            if let Some(text_render) = state.text_render.as_mut() {
                text_render.override_brush = Some(ColorBrush::new(current_style.color()));
            }
        }

        let style_changed = {
            let last_style = &state.last_text_style;

            current_style.font_size() != last_style.font_size()
                || current_style.font_weight() != last_style.font_weight()
                || current_style.font_style() != last_style.font_style()
                || current_style.font_family() != last_style.font_family()
                || current_style.underline() != last_style.underline()
        };

        let text = std::mem::take(&mut self.text);

        if state.text_hash != Some(text_hash) || reload_fonts || style_changed || scale_factor_changed {
            state.text_hash = Some(text_hash);
            state.text = text;
            state.layout = None;
            state.cache.clear();
            state.current_layout_key = None;
            state.last_requested_measure_key = None;
            state.current_render_key = None;
            state.text_render = None;
            state.content_widths = None;
        }

        state.last_text_style = current_style;
    }*/
}

/*impl Text {
    generate_component_methods_no_children!();
}*/

/*impl ElementStyles for Text {
    fn styles_mut(&mut self) -> &mut Style {
        self.element_data.current_style_mut()
    }
}
*/
impl TextState {
    pub fn measure(
        &mut self,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
        text_context: &mut TextContext,
    ) -> Size<f32> {
        if self.layout.is_none() {
            let mut builder = text_context.tree_builder(self.scale_factor, &Style::default().to_text_style());
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
            },
        });

        let height_constraint = known_dimensions.height.or(match available_space.height {
            AvailableSpace::MinContent => None,
            AvailableSpace::MaxContent => None,
            AvailableSpace::Definite(height) => {
                let scaled_height = dpi::PhysicalUnit::from_logical::<f32, f32>(height, self.scale_factor as f64).0;
                Some(scaled_height)
            },
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

        self.cache.insert(key, size);
        self.current_layout_key = Some(key);
        size
    }

    pub fn try_update_text_render(&mut self, _text_context: &mut TextContext) {
        if self.current_render_key == self.current_layout_key {
            return;
        }

        let layout = self.layout.as_ref().unwrap();
        self.text_render = Some(text_render_data::from_editor(layout));
        self.current_render_key = self.current_layout_key;
    }

    pub fn cursor_reset(&mut self) {
        self.start_time = Some(Instant::now());
        self.blink_period = Duration::from_millis(500);
    }

    pub fn extend_selection_to_point(&mut self, point: Point) {
        let scale_factor = self.layout.as_ref().unwrap().scale() as f64;
        let point = Point::new(point.x * scale_factor, point.y * scale_factor);
        self.selection = self.selection.extend_to_point(self.layout.as_ref().unwrap(), point.x as f32, point.y as f32);
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
}

impl TextData for Text {
    fn get_text_renderer(&self) -> Option<&TextRender> {
        self.state.text_render.as_ref()
    }
}