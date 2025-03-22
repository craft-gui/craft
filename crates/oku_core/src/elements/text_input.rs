use crate::components::component::ComponentSpecification;
use crate::components::Props;
use crate::components::UpdateResult;
use crate::elements::cached_editor::CachedEditor;
use crate::elements::common_element_data::CommonElementData;
use crate::elements::element::{Element, ElementBox};
use crate::elements::layout_context::{LayoutContext, TaffyTextInputContext};
use crate::elements::ElementStyles;
use crate::events::OkuMessage;
use crate::geometry::Point;
use crate::reactive::element_state_store::{ElementStateStore, ElementStateStoreItem};
use crate::renderer::color::Color;
use crate::style::{Display, Style, Unit};
use crate::{generate_component_methods_no_children, RendererBox};
use cosmic_text::Edit;
use cosmic_text::{Action, Motion};
use cosmic_text::FontSystem;
use std::any::Any;
use taffy::{NodeId, TaffyTree};
use winit::keyboard::{Key, NamedKey};

// A stateful element that shows text.
#[derive(Clone, Default, Debug)]
pub struct TextInput {
    text: String,
    common_element_data: CommonElementData,
}

pub struct TextInputState<'a> {
    pub cached_editor: CachedEditor<'a>,
    pub dragging: bool,
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
        
        let cached_editor = &mut state.cached_editor;

        let content_rect = self.common_element_data.computed_layered_rectangle.content_rectangle();
        let content_position = content_rect.position();
        
        match message {
            OkuMessage::PointerButtonEvent(pointer_button) => {
                let pointer_position = pointer_button.position;
                let pointer_content_position = pointer_position - content_position;
                if pointer_button.state.is_pressed() && content_rect.contains(&pointer_button.position) {
                    cached_editor.editor.action(
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
                    cached_editor.editor.action(
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
                            cached_editor.editor.action(font_system, Action::Motion(Motion::Left))
                        }
                        Key::Named(NamedKey::ArrowRight) => {
                            cached_editor.editor.action(font_system, Action::Motion(Motion::Right))
                        }
                        Key::Named(NamedKey::ArrowUp) => cached_editor.editor.action(font_system, Action::Motion(Motion::Up)),
                        Key::Named(NamedKey::ArrowDown) => {
                            cached_editor.editor.action(font_system, Action::Motion(Motion::Down))
                        }
                        Key::Named(NamedKey::Home) => cached_editor.editor.action(font_system, Action::Motion(Motion::Home)),
                        Key::Named(NamedKey::End) => cached_editor.editor.action(font_system, Action::Motion(Motion::End)),
                        Key::Named(NamedKey::PageUp) => {
                            cached_editor.editor.action(font_system, Action::Motion(Motion::PageUp))
                        }
                        Key::Named(NamedKey::PageDown) => {
                            cached_editor.editor.action(font_system, Action::Motion(Motion::PageDown))
                        }
                        Key::Named(NamedKey::Escape) => cached_editor.editor.action(font_system, Action::Escape),
                        Key::Named(NamedKey::Enter) => cached_editor.editor.action(font_system, Action::Enter),
                        Key::Named(NamedKey::Backspace) => cached_editor.editor.action(font_system, Action::Backspace),
                        Key::Named(NamedKey::Delete) => cached_editor.editor.action(font_system, Action::Delete),
                        Key::Named(key) => {
                            if let Some(text) = key.to_text() {
                                for char in text.chars() {
                                    cached_editor.editor.action(font_system, Action::Insert(char));
                                }
                            }
                        }
                        Key::Character(text) => {
                            for c in text.chars() {
                                cached_editor.editor.action(font_system, Action::Insert(c));

                                //text_context.editor.set_selection(Selection::Line(Cursor::new(0, 0)));
                            }
                        }
                        _ => {}
                    }
                }
                cached_editor.editor.shape_as_needed(font_system, true);
                cached_editor.clear_cache();
                
                cached_editor.editor.with_buffer(|buffer| {

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
        }
    }

    fn initialize_state(&self, font_system: &mut FontSystem, scaling_factor: f64) -> ElementStateStoreItem {
        let cached_editor = CachedEditor::new(&self.text, &self.common_element_data.style, scaling_factor, font_system);
        let text_input_state = TextInputState {
            cached_editor,
            dragging: false,
        };

        ElementStateStoreItem {
            base: Default::default(),
            data: Box::new(text_input_state)
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

        state.cached_editor.update_state(&self.text, &self.common_element_data.style, scaling_factor, reload_fonts, font_system);
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
