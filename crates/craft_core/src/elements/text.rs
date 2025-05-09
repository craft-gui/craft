use crate::components::component::ComponentSpecification;
use crate::components::{Props, UpdateResult};
use crate::elements::element::{Element, ElementBoxed};
use crate::elements::element_data::ElementData;
use crate::elements::ElementStyles;
use crate::events::CraftMessage;
use crate::geometry::Point;
use crate::layout::layout_context::{LayoutContext, TaffyTextContext, TextHashKey};
use crate::reactive::element_state_store::{ElementStateStore, ElementStateStoreItem};
use crate::renderer::renderer::RenderList;
use crate::style::Style;
use crate::text::text_context::{ColorBrush, TextContext};
use crate::text::text_render_data;
use crate::text::text_render_data::TextRender;
use crate::generate_component_methods_no_children;
use parley::{Alignment, AlignmentOptions, Selection};
use rustc_hash::FxHasher;
use std::any::Any;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};
use taffy::{AvailableSpace, NodeId, Size, TaffyTree};
use winit::window::Window;

// A stateful element that shows text.
#[derive(Clone, Default, Debug)]
pub struct Text {
    text: Option<String>,
    element_data: ElementData,
    selectable: bool,
}

pub struct TextState {
    selection: Selection,
    text: Option<String>,
    text_hash: Option<u64>,
    text_render: Option<TextRender>,
    last_text_style: Style,
    layout: Option<parley::Layout<ColorBrush>>,
    cache: HashMap<TextHashKey, Size<f32>>,
    current_key: Option<TextHashKey>,
    last_requested_key: Option<TextHashKey>,

    pub(crate) last_click_time: Option<Instant>,
    pub(crate) click_count: u32,
    pub(crate) pointer_down: bool,
    pub(crate) cursor_pos: (f32, f32),
    pub(crate) start_time: Option<Instant>,
    pub(crate) blink_period: Duration,
}

impl Text {
    pub fn new(text: &str) -> Text {
        Text {
            text: Some(text.to_string()),
            element_data: Default::default(),
            selectable: true,
        }
    }

    pub fn disable_selection(mut self) -> Self {
        self.selectable = false;
        self
    }
}

impl Element for Text {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }

    fn children_mut(&mut self) -> &mut Vec<ElementBoxed> {
        &mut self.element_data.children
    }

    fn name(&self) -> &'static str {
        "Text"
    }

    fn draw(
        &mut self,
        renderer: &mut RenderList,
        text_context: &mut TextContext,
        _taffy_tree: &mut TaffyTree<LayoutContext>,
        _root_node: NodeId,
        element_state: &mut ElementStateStore,
        _pointer: Option<Point>,
        window: Option<Arc<dyn Window>>,
    ) {
        if !self.element_data.style.visible() {
            return;
        }
        let computed_box_transformed = self.element_data.computed_box_transformed;
        let content_rectangle = computed_box_transformed.content_rectangle();

        self.draw_borders(renderer);

        let state: &mut TextState = element_state
            .storage
            .get_mut(&self.element_data.component_id)
            .unwrap()
            .data
            .as_mut()
            .downcast_mut()
            .unwrap();

        if let Some(text_render) = state.text_render.as_ref() {
            renderer.draw_text(text_render.clone(), content_rectangle, None, false);
        }
    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        _element_state: &mut ElementStateStore,
        scale_factor: f64,
    ) -> Option<NodeId> {
        let style: taffy::Style = self.element_data.style.to_taffy_style_with_scale_factor(scale_factor);

        self.element_data_mut().taffy_node_id = Some(
            taffy_tree
                .new_leaf_with_context(
                    style,
                    LayoutContext::Text(TaffyTextContext::new(self.element_data.component_id)),
                )
                .unwrap(),
        );

        self.element_data().taffy_node_id
    }

    fn finalize_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        position: Point,
        z_index: &mut u32,
        transform: glam::Mat4,
        element_state: &mut ElementStateStore,
        pointer: Option<Point>,
        text_context: &mut TextContext,
    ) {
        let result = taffy_tree.layout(root_node).unwrap();
        self.resolve_box(position, transform, result, z_index);

        self.finalize_borders();

        let state: &mut TextState = element_state
            .storage
            .get_mut(&self.element_data.component_id)
            .unwrap()
            .data
            .as_mut()
            .downcast_mut()
            .unwrap();

        if state.current_key != state.last_requested_key {
            state.layout(
                state.last_requested_key.unwrap().known_dimensions(),
                state.last_requested_key.unwrap().available_space(),
            );
        }

        let layout = state.layout.as_ref().unwrap();
        let text_renderer = state.text_render.as_mut().unwrap();
        for line in text_renderer.lines.iter_mut() {
            line.selections.clear();
        }
        state.selection.geometry_with(layout, |rect, line| {
            text_renderer.lines[line].selections.push(rect.into());
        });
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn on_event(&self, _message: &CraftMessage, _element_state: &mut ElementStateStore, text_context: &mut TextContext) -> UpdateResult {
        let state: &mut TextState = _element_state
            .storage
            .get_mut(&self.element_data.component_id)
            .unwrap()
            .data
            .as_mut()
            .downcast_mut()
            .unwrap();

        let content_rect = self.element_data.computed_box.content_rectangle();
        let content_position = content_rect.position();

        // Handle selection.
        if self.selectable {
            let text_position = self.element_data().computed_box_transformed.content_rectangle();

            match _message {
                CraftMessage::PointerButtonEvent(pointer_button) => {
                    if pointer_button.button.mouse_button() == winit::event::MouseButton::Left {
                        state.pointer_down = pointer_button.state.is_pressed();
                        state.cursor_reset();
                        if state.pointer_down {
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
                                2 => state.select_word_at_point(cursor_pos.0, cursor_pos.1),
                                3 => state.select_line_at_point(cursor_pos.0, cursor_pos.1),
                                _ => state.move_to_point(cursor_pos.0, cursor_pos.1),
                            }
                        }
                    }
                    UpdateResult::new().prevent_defaults()
                }
                CraftMessage::PointerMovedEvent(pointer_moved) => {
                    let prev_pos = state.cursor_pos;
                    // NOTE: Cursor position should be relative to the top left of the text box.
                    state.cursor_pos = (pointer_moved.position.x - text_position.x, pointer_moved.position.y - text_position.y);
                    // macOS seems to generate a spurious move after selecting word?
                    if state.pointer_down && prev_pos != state.cursor_pos {
                        state.cursor_reset();
                        let cursor_pos = state.cursor_pos;
                        state.extend_selection_to_point(cursor_pos.0, cursor_pos.1);
                    }
                    UpdateResult::new().prevent_defaults()
                }
                _ => UpdateResult::new(),
            }
        } else {
            UpdateResult::default()
        }
    }

    fn initialize_state(&mut self, _scaling_factor: f64) -> ElementStateStoreItem {
        let hash = hash_string(self.text.as_ref().unwrap());
        let text_state = TextState {
            selection: Selection::default(),
            text: std::mem::take(&mut self.text),
            text_hash: Some(hash),
            text_render: None,
            last_text_style: self.style().clone(),
            layout: None,
            cache: Default::default(),
            current_key: None,
            last_requested_key: None,
            last_click_time: None,
            click_count: 0,
            pointer_down: false,
            cursor_pos: (0.0, 0.0),
            start_time: None,
            blink_period: Default::default(),
        };

        //parley::editor::PlainEditor::new()
        //parley::editor::PlainEditorDriver::

        ElementStateStoreItem {
            base: Default::default(),
            data: Box::new(text_state),
        }
    }

    fn update_state(&mut self, element_state: &mut ElementStateStore, reload_fonts: bool, scaling_factor: f64) {
        let text_hash = hash_string(self.text.as_ref().unwrap());

        let state: &mut TextState = element_state
            .storage
            .get_mut(&self.element_data.component_id)
            .unwrap()
            .data
            .as_mut()
            .downcast_mut()
            .unwrap();

        let style_changed = {
            let style = self.style();
            let last_style = &state.last_text_style;

            if style.font_size() != last_style.font_size()
                || style.font_weight() != last_style.font_weight()
                || style.font_style() != last_style.font_style()
                || style.font_family() != last_style.font_family()
            {
                true
            } else {
                false
            }
        };

        let text = std::mem::take(&mut self.text);

        if state.text_hash != Some(text_hash) || reload_fonts || style_changed {
            state.text_hash = Some(text_hash);
            state.text = text;
            state.layout = None;
            state.cache.clear();
            state.current_key = None;
            state.last_requested_key = None;
        }

        state.last_text_style = self.style().clone();
    }
}

fn hash_string(text: &str) -> u64 {
    let mut hasher = FxHasher::default();
    text.hash(&mut hasher);
    hasher.finish()
}

impl Text {
    generate_component_methods_no_children!();
}

impl ElementStyles for Text {
    fn styles_mut(&mut self) -> &mut Style {
        self.element_data.current_style_mut()
    }
}

impl TextState {
    pub fn measure(
        &mut self,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<taffy::AvailableSpace>,
        text_context: &mut TextContext,
    ) -> Size<f32> {
        if self.layout.is_none() {
            let mut builder = text_context.tree_builder(&self.last_text_style.to_text_style());
            let text = std::mem::take(&mut self.text).unwrap();
            builder.push_text(&text);
            let (layout, _) = builder.build();
            self.layout = Some(layout);
        }

        let key = TextHashKey::new(known_dimensions, available_space);

        self.last_requested_key = Some(key);

        if let Some(value) = self.cache.get(&key) {
            return *value;
        }

        self.layout(known_dimensions, available_space)
    }

    pub fn layout(
        &mut self,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<taffy::AvailableSpace>,
    ) -> Size<f32> {
        let key = TextHashKey::new(known_dimensions, available_space);

        if let Some(value) = self.cache.get(&key) {
            return *value;
        }

        let layout = self.layout.as_mut().unwrap();

        let width_constraint = known_dimensions.width.or(match available_space.width {
            AvailableSpace::MinContent => Some(layout.min_content_width()),
            AvailableSpace::MaxContent => Some(layout.max_content_width()),
            AvailableSpace::Definite(width) => Some(width),
        });
        // Some(self.text_style.font_size * self.text_style.line_height)
        let height_constraint = known_dimensions.height.or(match available_space.height {
            AvailableSpace::MinContent => None,
            AvailableSpace::MaxContent => None,
            AvailableSpace::Definite(height) => Some(height),
        });
        layout.break_all_lines(width_constraint);
        layout.align(width_constraint, Alignment::Start, AlignmentOptions::default());

        let width = layout.width();
        let height = layout.height().min(height_constraint.unwrap_or(f32::MAX));

        self.text_render = Some(text_render_data::from_editor(&layout));

        let size = Size { width, height };

        self.cache.insert(key, size);
        self.current_key = Some(key);
        size
    }

    pub fn cursor_reset(&mut self) {
        self.start_time = Some(Instant::now());
        self.blink_period = Duration::from_millis(500);
    }

    pub fn extend_selection_to_point(&mut self, x: f32, y: f32) {
        self.selection = self.selection.extend_to_point(self.layout.as_ref().unwrap(), x, y);
    }

    pub fn select_word_at_point(&mut self, x: f32, y: f32) {
        self.selection = Selection::word_from_point(self.layout.as_ref().unwrap(), x, y);
    }

    pub fn select_line_at_point(&mut self, x: f32, y: f32) {
        self.selection = Selection::line_from_point(self.layout.as_ref().unwrap(), x, y);
    }

    pub fn move_to_point(&mut self, x: f32, y: f32) {
        self.selection = Selection::from_point(self.layout.as_ref().unwrap(), x, y);
    }
}