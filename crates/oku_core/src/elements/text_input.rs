use crate::components::component::{ComponentId, ComponentSpecification};
use crate::components::Props;
use crate::components::UpdateResult;
use crate::elements::common_element_data::CommonElementData;
use crate::elements::element::{Element, ElementBox};
use crate::elements::layout_context::{
    AvailableSpace, LayoutContext, MetricsDummy, TaffyTextInputContext, TextHashKey,
};
use crate::elements::text::TextHashValue;
use crate::elements::ElementStyles;
use crate::events::OkuMessage;
use crate::geometry::Point;
use crate::reactive::element_state_store::{ElementStateStore, ElementStateStoreItem};
use crate::renderer::color::Color;
use crate::style::{Style, Unit};
use crate::{generate_component_methods_no_children, RendererBox};
use cosmic_text::{Action, Buffer, Motion, Shaping};
use cosmic_text::{Attrs, Editor, FontSystem, Metrics};
use cosmic_text::{Edit, Family, Weight};
use rustc_hash::FxHasher;
use std::any::Any;
use std::collections::HashMap;
use std::hash::Hasher;
use taffy::{NodeId, TaffyTree};
use winit::dpi::{LogicalPosition, PhysicalPosition};
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
    pub last_key: TextHashKey,
    pub editor: Editor<'a>,
    pub text: String,
    pub original_text_hash: u64,
    pub dragging: bool,
    pub(crate) font_family_length: u8,
    pub(crate) font_family: [u8; 64],
    weight: Weight,
}

impl TextInputState<'_> {
    pub(crate) fn get_last_cache_entry(&self) -> &TextHashValue {
        let key = self.last_key;
        &self.cached_text_layout[&key]
    }
}

impl<'a> TextInputState<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: ComponentId,
        metrics: Metrics,
        text_hash: u64,
        editor: Editor<'a>,
        text: String,
        original_text_hash: u64,
        font_family_length: u8,
        font_family: [u8; 64],
        weight: Weight,
    ) -> Self {
        Self {
            _id: id,
            text_hash,
            cached_text_layout: Default::default(),
            last_key: TextHashKey {
                text_hash,
                width_constraint: None,
                height_constraint: None,
                available_space_width: AvailableSpace::MinContent,
                available_space_height: AvailableSpace::MinContent,
                metrics: MetricsDummy {
                    font_size: metrics.font_size.to_bits(),
                    line_height: metrics.line_height.to_bits(),
                },
                font_family_length,
                font_family,
            },
            editor,
            text,
            original_text_hash,
            dragging: false,
            font_family_length,
            font_family,
            weight,
        }
    }

    pub fn font_family(&self) -> Option<&str> {
        if self.font_family_length == 0 {
            None
        } else {
            Some(std::str::from_utf8(&self.font_family[..self.font_family_length as usize]).unwrap())
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn measure(
        &mut self,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        font_system: &mut FontSystem,
        text_hash: u64,
        metrics: Metrics,
        font_family_length: u8,
        font_family: [u8; 64],
    ) -> taffy::Size<f32> {
        // Set width constraint
        let width_constraint = known_dimensions.width.or(match available_space.width {
            taffy::AvailableSpace::MinContent => Some(0.0),
            taffy::AvailableSpace::MaxContent => None,
            taffy::AvailableSpace::Definite(width) => Some(width),
        });

        let height_constraint = known_dimensions.height;

        let available_space_width_u32: AvailableSpace = match available_space.width {
            taffy::AvailableSpace::MinContent => AvailableSpace::MinContent,
            taffy::AvailableSpace::MaxContent => AvailableSpace::MaxContent,
            taffy::AvailableSpace::Definite(width) => AvailableSpace::Definite(width.to_bits()),
        };
        let available_space_height_u32: AvailableSpace = match available_space.height {
            taffy::AvailableSpace::MinContent => AvailableSpace::MinContent,
            taffy::AvailableSpace::MaxContent => AvailableSpace::MaxContent,
            taffy::AvailableSpace::Definite(height) => AvailableSpace::Definite(height.to_bits()),
        };

        let key = TextHashKey {
            text_hash,
            width_constraint: width_constraint.map(|w| w.to_bits()),
            height_constraint: height_constraint.map(|h| h.to_bits()),
            available_space_width: available_space_width_u32,
            available_space_height: available_space_height_u32,
            metrics: MetricsDummy {
                font_size: metrics.font_size.to_bits(),
                line_height: metrics.line_height.to_bits(),
            },
            font_family_length,
            font_family,
        };

        self.last_key = key;
        let cached_text_layout_value = self.cached_text_layout.get(&key);
        self.text_hash = text_hash;

        if cached_text_layout_value.is_none() {
            self.editor.with_buffer_mut(|buffer| {
                buffer.set_metrics(font_system, metrics);
                buffer.set_size(font_system, width_constraint, height_constraint);
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
            
            self.cached_text_layout.insert(key, cached_text_layout_value);
            
            size
        } else {
            let cached_text_layout_value = cached_text_layout_value.unwrap();
            taffy::Size {
                width: cached_text_layout_value.computed_width,
                height: cached_text_layout_value.computed_height,
            }
        }
    }
}

impl TextInput {
    pub fn new(text: &str) -> Self {
        let mut common_element_data = CommonElementData::default();
        const BORDER_COLOR: Color = Color::from_rgb8(199, 199, 206);
        *common_element_data.style.border_color_mut() = [BORDER_COLOR; 4];
        *common_element_data.style.border_width_mut() = [Unit::Px(1.0); 4];
        *common_element_data.style.border_radius_mut() = [(5.0, 5.0); 4];

        Self {
            text: text.to_string(),
            common_element_data,
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
        _element_state: &ElementStateStore,
        _pointer: Option<Point>,
    ) {
        let computed_layer_rectangle_transformed =
            self.common_element_data.computed_layered_rectangle_transformed;
        let content_rectangle = computed_layer_rectangle_transformed.content_rectangle();

        self.draw_borders(renderer);

        renderer.draw_text(
            self.common_element_data.component_id,
            content_rectangle,
            self.common_element_data.style.color(),
        ); 
    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        _element_state: &mut ElementStateStore,
        scale_factor: f64,
    ) -> Option<NodeId> {
        let font_size = PhysicalPosition::from_logical(
            LogicalPosition::new(self.common_element_data.style.font_size(), self.common_element_data.style.font_size()),
            scale_factor,
        )
        .x;

        let font_line_height = font_size * 1.2;
        let metrics = Metrics::new(font_size, font_line_height);

        let style: taffy::Style = self.common_element_data.style.to_taffy_style_with_scale_factor(scale_factor);

        self.common_element_data_mut().taffy_node_id = Some(taffy_tree
            .new_leaf_with_context(
                style,
                LayoutContext::TextInput(TaffyTextInputContext::new(self.common_element_data.component_id, metrics)),
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
        _element_state: &mut ElementStateStore,
        _pointer: Option<Point>,
        _font_system: &mut FontSystem,
    ) {
        let result = taffy_tree.layout(root_node).unwrap();
        self.resolve_layer_rectangle(position, transform, result, z_index);
        
        self.finalize_borders();
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
        let state: &mut TextInputState = element_state
            .storage
            .get_mut(&self.common_element_data.component_id)
            .unwrap()
            .data
            .as_mut()
            .downcast_mut()
            .unwrap();

        let content_rect = self.common_element_data.computed_layered_rectangle.content_rectangle();
        let content_position = content_rect.position();
        let res = match message {
            OkuMessage::PointerButtonEvent(pointer_button) => {
                let pointer_position = pointer_button.position;
                let pointer_content_position = pointer_position - content_position;
                if pointer_button.state.is_pressed() && content_rect.contains(&pointer_button.position) {
                    state.editor.action(
                        font_system,
                        Action::Click {
                            x: pointer_content_position.x as i32,
                            y: pointer_content_position.y as i32,
                        },
                    );
                    state.dragging = true;
                } else {
                    state.dragging = false;
                }
                UpdateResult::new().prevent_defaults().prevent_propagate()
            }
            OkuMessage::PointerMovedEvent(moved) => {
                if state.dragging {
                    let pointer_position = moved.position;
                    let pointer_content_position = pointer_position - content_position;
                    state.editor.action(
                        font_system,
                        Action::Drag {
                            x: pointer_content_position.x as i32,
                            y: pointer_content_position.y as i32,
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
                            state.editor.action(font_system, Action::Motion(Motion::Left))
                        }
                        Key::Named(NamedKey::ArrowRight) => {
                            state.editor.action(font_system, Action::Motion(Motion::Right))
                        }
                        Key::Named(NamedKey::ArrowUp) => state.editor.action(font_system, Action::Motion(Motion::Up)),
                        Key::Named(NamedKey::ArrowDown) => {
                            state.editor.action(font_system, Action::Motion(Motion::Down))
                        }
                        Key::Named(NamedKey::Home) => state.editor.action(font_system, Action::Motion(Motion::Home)),
                        Key::Named(NamedKey::End) => state.editor.action(font_system, Action::Motion(Motion::End)),
                        Key::Named(NamedKey::PageUp) => {
                            state.editor.action(font_system, Action::Motion(Motion::PageUp))
                        }
                        Key::Named(NamedKey::PageDown) => {
                            state.editor.action(font_system, Action::Motion(Motion::PageDown))
                        }
                        Key::Named(NamedKey::Escape) => state.editor.action(font_system, Action::Escape),
                        Key::Named(NamedKey::Enter) => state.editor.action(font_system, Action::Enter),
                        Key::Named(NamedKey::Backspace) => state.editor.action(font_system, Action::Backspace),
                        Key::Named(NamedKey::Delete) => state.editor.action(font_system, Action::Delete),
                        Key::Named(key) => {
                            if let Some(text) = key.to_text() {
                                for char in text.chars() {
                                    state.editor.action(font_system, Action::Insert(char));
                                }
                            }
                        }
                        Key::Character(text) => {
                            for c in text.chars() {
                                state.editor.action(font_system, Action::Insert(c));

                                //text_context.editor.set_selection(Selection::Line(Cursor::new(0, 0)));
                            }
                        }
                        _ => {}
                    }
                }
                state.editor.shape_as_needed(font_system, true);

                state.editor.with_buffer(|buffer| {
                    let mut buffer_string: String = String::new();
                    let last_line = buffer.lines.len() - 1;
                    for (line_number, line) in buffer.lines.iter().enumerate() {
                        buffer_string.push_str(line.text());
                        if line_number != last_line {
                            buffer_string.push('\n');
                        }
                    }

                    let mut text_hasher = FxHasher::default();
                    text_hasher.write(buffer_string.as_bytes());
                    let text_hash = text_hasher.finish();

                    state.text_hash = text_hash;
                    state.text = buffer_string.clone();

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

    fn initialize_state(&self, font_system: &mut FontSystem) -> ElementStateStoreItem {
        let font_size = self.common_element_data.style.font_size();
        let font_line_height = font_size * 1.2;
        let metrics = Metrics::new(font_size, font_line_height);

        let new_font_family = self.common_element_data.style.font_family();

        let mut attributes = Attrs::new();

        if let Some(family) = new_font_family {
            attributes.family = Family::Name(family);
        }

        attributes.weight = Weight(self.common_element_data.style.font_weight().0);

        let buffer = Buffer::new(font_system, metrics);
        let mut editor = Editor::new(buffer);
        editor.borrow_with(font_system);

        let mut text_hasher = FxHasher::default();
        text_hasher.write(self.text.as_ref());
        let text_hash = text_hasher.finish();

        editor.with_buffer_mut(|buffer| buffer.set_text(font_system, &self.text, attributes, Shaping::Advanced));
        editor.action(font_system, Action::Motion(Motion::End));

        let cosmic_text_content = TextInputState::new(
            self.common_element_data.component_id,
            metrics,
            text_hash,
            editor,
            self.text.clone(),
            text_hash,
            self.common_element_data.style.font_family_length(),
            self.common_element_data.style.font_family_raw(),
            attributes.weight,
        );

        ElementStateStoreItem {
            base: Default::default(),
            data: Box::new(cosmic_text_content)
        }
    }

    fn update_state(&self, font_system: &mut FontSystem, element_state: &mut ElementStateStore, reload_fonts: bool) {
        let state: &mut TextInputState = element_state
            .storage
            .get_mut(&self.common_element_data.component_id)
            .unwrap()
            .data
            .as_mut()
            .downcast_mut()
            .unwrap();

        let mut text_hasher = FxHasher::default();
        text_hasher.write(self.text.as_ref());
        let text_hash = text_hasher.finish();

        let mut attributes = Attrs::new();

        attributes.weight = Weight(self.common_element_data.style.font_weight().0);

        let new_font_family = self.common_element_data().style.font_family();

        if let Some(family) = new_font_family {
            attributes.family = Family::Name(family);
        }

        if text_hash != state.original_text_hash
            || state.font_family() != new_font_family
            || reload_fonts
            || attributes.weight != state.weight
        {
            state.font_family_length = self.common_element_data.style.font_family_length();
            state.font_family = self.common_element_data.style.font_family_raw();
            state.original_text_hash = text_hash;
            state.text_hash = text_hash;
            state.text = self.text.clone();
            state.weight = attributes.weight;

            state.editor.with_buffer_mut(|buffer| {
                buffer.set_text(font_system, &self.text, attributes, Shaping::Advanced);
            });
        }
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
