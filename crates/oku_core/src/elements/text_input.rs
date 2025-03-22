use crate::components::component::{ComponentId, ComponentSpecification};
use crate::components::Props;
use crate::components::UpdateResult;
use crate::elements::common_element_data::CommonElementData;
use crate::elements::element::{Element, ElementBox};
use crate::elements::layout_context::{LayoutContext, MetricsRaw, TaffyTextInputContext, TextHashKey};
use crate::elements::scroll_state::ScrollState;
use crate::elements::text::{hash_text, AttributesRaw, TextHashValue};
use crate::elements::ElementStyles;
use crate::events::OkuMessage;
use crate::geometry::{Point, Size};
use crate::reactive::element_state_store::{ElementStateStore, ElementStateStoreItem};
use crate::renderer::color::Color;
use crate::renderer::renderer::TextScroll;
use crate::style::{Display, Style, Unit};
use crate::{generate_component_methods_no_children, RendererBox};
use cosmic_text::Edit;
use cosmic_text::{Action, Buffer, Motion, Shaping};
use cosmic_text::{Editor, FontSystem};
use std::any::Any;
use std::collections::HashMap;
use taffy::{NodeId, TaffyTree};
use winit::keyboard::{Key, NamedKey};

// A stateful element that shows text.
#[derive(Clone, Default, Debug)]
pub struct TextInput {
    text: String,
    common_element_data: CommonElementData,
}

pub struct TextInputState<'a> {
    pub _id: ComponentId,
    pub text_hash: u64,
    pub cached_text_layout: HashMap<TextHashKey, TextHashValue>,
    pub last_key: Option<TextHashKey>,
    pub editor: Editor<'a>,
    pub dragging: bool,
    // Attributes
    pub(crate) attributes: AttributesRaw,
    // Metrics
    pub(crate) metrics: MetricsRaw,
    pub(crate) scroll_state: ScrollState,
}

impl TextInputState<'_> {
    pub(crate) fn get_last_cache_entry(&self) -> &TextHashValue {
        let key = self.last_key.unwrap();
        &self.cached_text_layout[&key]
    }
}

impl<'a> TextInputState<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: ComponentId,
        text_hash: u64,
        editor: Editor<'a>,
        metrics: MetricsRaw,
        attributes_raw: AttributesRaw,
    ) -> Self {
        Self {
            _id: id,
            text_hash,
            cached_text_layout: Default::default(),
            last_key: None,
            editor,
            dragging: false,
            metrics,
            attributes: attributes_raw,
            scroll_state: ScrollState::default(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn measure(
        &mut self,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        font_system: &mut FontSystem,
    ) -> taffy::Size<f32> {
        let cache_key = TextHashKey::new(known_dimensions, available_space);
        self.last_key = Some(cache_key);

        if self.cached_text_layout.len() > 3 {
            self.cached_text_layout.clear();
        }

        let cached_text_layout_value = self.cached_text_layout.get(&cache_key);

        if let Some(cached_text_layout_value) = cached_text_layout_value {
            taffy::Size {
                width: cached_text_layout_value.computed_width,
                height: cached_text_layout_value.computed_height,
            }
        } else {
            self.editor.with_buffer_mut(|buffer| {
                buffer.set_metrics_and_size(font_system, self.metrics.to_metrics(), cache_key.width_constraint.map(f32::from_bits), cache_key.height_constraint.map(f32::from_bits));
            });
            self.editor.shape_as_needed(font_system, true);

            // Determine measured size of text
            let cached_text_layout_value = self.editor.with_buffer(|buffer| {
                let (width, total_lines) = buffer
                    .layout_runs()
                    .fold((0.0, 0usize), |(width, total_lines), run| (run.line_w.max(width), total_lines + 1));
                let height = total_lines as f32 * buffer.metrics().line_height;

                TextHashValue {
                    computed_width: width,
                    computed_height: height,
                    buffer: buffer.clone(),
                }
            });

            let size = taffy::Size {
                width: cached_text_layout_value.computed_width,
                height: cached_text_layout_value.computed_height,
            };

            self.cached_text_layout.insert(cache_key, cached_text_layout_value);
            size
        }
    }
}

impl TextInput {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            common_element_data: CommonElementData::default(),
        }
    }

    #[allow(dead_code)]
    fn get_state<'a>(&self, element_state: &'a ElementStateStore) -> &'a TextInputState {
        element_state.storage.get(&self.common_element_data.component_id).unwrap().data.as_ref().downcast_ref().unwrap()
    }
}

impl Element for TextInput {
    fn common_element_data(&self) -> &CommonElementData {
        &self.common_element_data
    }

    fn common_element_data_mut(&mut self) -> &mut CommonElementData {
        &mut self.common_element_data
    }

    fn children_mut(&mut self) -> &mut Vec<ElementBox> {
        &mut self.common_element_data.children
    }

    fn name(&self) -> &'static str {
        "TextInput"
    }

    fn draw(
        &mut self,
        renderer: &mut RendererBox,
        _font_system: &mut FontSystem,
        _taffy_tree: &mut TaffyTree<LayoutContext>,
        _root_node: NodeId,
        element_state: &ElementStateStore,
        _pointer: Option<Point>,
    ) {
        let computed_layer_rectangle_transformed =
            self.common_element_data.computed_layered_rectangle_transformed;
        let content_rectangle = computed_layer_rectangle_transformed.content_rectangle();

        self.draw_borders(renderer);

        let is_scrollable = self.common_element_data.is_scrollable();

        if is_scrollable {
            self.maybe_start_layer(renderer);
        }

        let scroll_y = if let Some(state) = element_state
            .storage
            .get(&self.common_element_data.component_id)
            .unwrap()
            .data
            .downcast_ref::<TextInputState>()
        {
            state.scroll_state.scroll_y
        } else {
            0.0
        };

        renderer.draw_text(
            self.common_element_data.component_id,
            content_rectangle,
            self.common_element_data.style.color(),
            Some(TextScroll::new(scroll_y, self.common_element_data.computed_scroll_track.height))
        );

        if is_scrollable {
            self.maybe_end_layer(renderer);
        }

        self.draw_scrollbar(renderer);
    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        _element_state: &mut ElementStateStore,
        scale_factor: f64,
    ) -> Option<NodeId> {
        self.merge_default_style();
        let style: taffy::Style = self.common_element_data.style.to_taffy_style_with_scale_factor(scale_factor);

        self.common_element_data_mut().taffy_node_id = Some(taffy_tree
            .new_leaf_with_context(
                style,
                LayoutContext::TextInput(TaffyTextInputContext::new(self.common_element_data.component_id)),
            )
            .unwrap());

        self.common_element_data().taffy_node_id
    }

    fn finalize_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        position: Point,
        z_index: &mut u32,
        transform: glam::Mat4,
        element_state: &mut ElementStateStore,
        _pointer: Option<Point>,
        _font_system: &mut FontSystem,
    ) {
        let result = taffy_tree.layout(root_node).unwrap();
        self.resolve_layer_rectangle(position, transform, result, z_index);
        self.finalize_borders();

        self.common_element_data.scrollbar_size = Size::new(result.scrollbar_size.width, result.scrollbar_size.height);
        self.common_element_data.computed_scrollbar_size = Size::new(result.scroll_width(), result.scroll_height());

        let scroll_y = if let Some(container_state) = element_state
            .storage
            .get(&self.common_element_data.component_id)
            .unwrap()
            .data
            .downcast_ref::<TextInputState>()
        {
            container_state.scroll_state.scroll_y
        } else {
            0.0
        };

        self.finalize_scrollbar(scroll_y);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn on_event(
        &self,
        message: OkuMessage,
        element_state: &mut ElementStateStore,
        font_system: &mut FontSystem,
    ) -> UpdateResult {
        let base_state = self.get_base_state_mut(element_state);
        let text_input_state = base_state.data.as_mut().downcast_mut::<TextInputState>().unwrap();

        let scroll_result = text_input_state.scroll_state.on_event(&message, &self.common_element_data, &mut base_state.base);
        if !scroll_result.propagate {
            return scroll_result;
        }

        let scroll_y = text_input_state.scroll_state.scroll_y;

        let content_rect = self.common_element_data.computed_layered_rectangle.content_rectangle();
        let content_position = content_rect.position();
        let res = match message {
            OkuMessage::PointerButtonEvent(pointer_button) => {
                let pointer_position = pointer_button.position;
                let pointer_content_position = pointer_position - content_position;
                if pointer_button.state.is_pressed() && content_rect.contains(&pointer_button.position) {
                    text_input_state.editor.action(
                        font_system,
                        Action::Click {
                            x: pointer_content_position.x as i32,
                            y: (pointer_content_position.y + scroll_y) as i32,
                        },
                    );
                    text_input_state.dragging = true;
                } else {
                    text_input_state.dragging = false;
                }
                UpdateResult::new().prevent_defaults().prevent_propagate()
            }
            OkuMessage::PointerMovedEvent(moved) => {
                if text_input_state.dragging {
                    let pointer_position = moved.position;
                    let pointer_content_position = pointer_position - content_position;
                    text_input_state.editor.action(
                        font_system,
                        Action::Drag {
                            x: pointer_content_position.x as i32,
                            y: (pointer_content_position.y + scroll_y) as i32,
                        },
                    );
                }
                UpdateResult::new().prevent_defaults().prevent_propagate()
            }
            OkuMessage::KeyboardInputEvent(keyboard_input) => {
                let logical_key = keyboard_input.event.logical_key;
                let key_state = keyboard_input.event.state;

                if key_state.is_pressed() {
                    match logical_key {
                        Key::Named(NamedKey::ArrowLeft) => {
                            text_input_state.editor.action(font_system, Action::Motion(Motion::Left))
                        }
                        Key::Named(NamedKey::ArrowRight) => {
                            text_input_state.editor.action(font_system, Action::Motion(Motion::Right))
                        }
                        Key::Named(NamedKey::ArrowUp) => text_input_state.editor.action(font_system, Action::Motion(Motion::Up)),
                        Key::Named(NamedKey::ArrowDown) => {
                            text_input_state.editor.action(font_system, Action::Motion(Motion::Down))
                        }
                        Key::Named(NamedKey::Home) => text_input_state.editor.action(font_system, Action::Motion(Motion::Home)),
                        Key::Named(NamedKey::End) => text_input_state.editor.action(font_system, Action::Motion(Motion::End)),
                        Key::Named(NamedKey::PageUp) => {
                            text_input_state.editor.action(font_system, Action::Motion(Motion::PageUp))
                        }
                        Key::Named(NamedKey::PageDown) => {
                            text_input_state.editor.action(font_system, Action::Motion(Motion::PageDown))
                        }
                        Key::Named(NamedKey::Escape) => text_input_state.editor.action(font_system, Action::Escape),
                        Key::Named(NamedKey::Enter) => text_input_state.editor.action(font_system, Action::Enter),
                        Key::Named(NamedKey::Backspace) => text_input_state.editor.action(font_system, Action::Backspace),
                        Key::Named(NamedKey::Delete) => text_input_state.editor.action(font_system, Action::Delete),
                        Key::Named(key) => {
                            if let Some(text) = key.to_text() {
                                for char in text.chars() {
                                    text_input_state.editor.action(font_system, Action::Insert(char));
                                }
                            }
                        }
                        Key::Character(text) => {
                            for c in text.chars() {
                                text_input_state.editor.action(font_system, Action::Insert(c));

                                //text_context.editor.set_selection(Selection::Line(Cursor::new(0, 0)));
                            }
                        }
                        _ => {}
                    }
                }
                text_input_state.editor.shape_as_needed(font_system, true);
                text_input_state.cached_text_layout.clear();
                text_input_state.last_key = None;
                text_input_state.editor.with_buffer(|buffer| {

                    let mut buffer_string: String = String::new();
                    let last_line = buffer.lines.len() - 1;
                    for (line_number, line) in buffer.lines.iter().enumerate() {
                        buffer_string.push_str(line.text());
                        if line_number != last_line {
                            buffer_string.push('\n');
                        }
                    }

                    UpdateResult::new()
                        .prevent_defaults()
                        .prevent_propagate()
                        .result_message(OkuMessage::TextInputChanged(buffer_string))
                })
            }
            _ => UpdateResult::new(),
        };

        res
    }

    fn initialize_state(&self, font_system: &mut FontSystem, scaling_factor: f64) -> ElementStateStoreItem {
        let metrics = MetricsRaw::from(&self.common_element_data().style, scaling_factor);

        let buffer = Buffer::new(font_system, metrics.to_metrics());
        let mut editor = Editor::new(buffer);
        editor.borrow_with(font_system);

        let text_hash = hash_text(&self.text);
        let attributes = AttributesRaw::from(&self.common_element_data.style);
        editor.with_buffer_mut(|buffer| buffer.set_text(font_system, &self.text, attributes.to_attrs(), Shaping::Advanced));
        editor.action(font_system, Action::Motion(Motion::End));

        let cosmic_text_content = TextInputState::new(
            self.common_element_data.component_id,
            text_hash,
            editor,
            metrics,
            attributes,
        );

        ElementStateStoreItem {
            base: Default::default(),
            data: Box::new(cosmic_text_content)
        }
    }

    fn update_state(&self, font_system: &mut FontSystem, element_state: &mut ElementStateStore, reload_fonts: bool, scaling_factor: f64) {
        let state: &mut TextInputState = element_state
            .storage
            .get_mut(&self.common_element_data.component_id)
            .unwrap()
            .data
            .as_mut()
            .downcast_mut()
            .unwrap();

        let text_hash = hash_text(&self.text);
        let attributes = AttributesRaw::from(&self.common_element_data.style);
        let metrics = MetricsRaw::from(&self.common_element_data.style, scaling_factor);

        let text_changed = text_hash != state.text_hash
            || reload_fonts
            || attributes != state.attributes;
        let size_changed = metrics != state.metrics;

        if text_changed || size_changed {
            state.cached_text_layout.clear();
            state.last_key = None;
        }

        if size_changed {
            state.metrics = metrics;
        }
        
        
        if text_changed {
            state.editor.with_buffer_mut(|buffer| {
                buffer.set_text(font_system, &self.text, attributes.to_attrs(), Shaping::Advanced);
            });
            
            state.attributes = attributes;
            state.text_hash = text_hash;
        }
    }

    fn default_style(&self) -> Style {
        let mut style = Style::default();
        *style.display_mut() = Display::Block;
        const BORDER_COLOR: Color = Color::from_rgb8(199, 199, 206);
        *style.border_color_mut() = [BORDER_COLOR; 4];
        *style.border_width_mut() = [Unit::Px(1.0); 4];
        *style.border_radius_mut() = [(5.0, 5.0); 4];
        let padding = Unit::Px(4.0);
        *style.padding_mut() = [padding, padding, padding, padding];

        style
    }
}

impl TextInput {
    generate_component_methods_no_children!();
}

impl ElementStyles for TextInput {
    fn styles_mut(&mut self) -> &mut Style {
        self.common_element_data.current_style_mut()
    }
}
