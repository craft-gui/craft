use crate::components::component::ComponentSpecification;
use crate::components::{Props, UpdateResult};
use crate::elements::element::{Element, ElementBoxed};
use crate::elements::element_data::ElementData;
use crate::elements::layout_context::{LayoutContext, TaffyTextContext};
use crate::elements::ElementStyles;
use crate::events::CraftMessage;
use crate::geometry::Point;
use crate::reactive::element_state_store::{ElementStateStore, ElementStateStoreItem};
use crate::renderer::renderer::RenderList;
use crate::style::Style;
use crate::text::cached_editor::CachedEditor;
use crate::generate_component_methods_no_children;
use cosmic_text::FontSystem;
use peniko::Color;
use std::any::Any;
use std::sync::Arc;
use taffy::{NodeId, TaffyTree};
use winit::keyboard::Key;
use winit::window::Window;

// A stateful element that shows text.
#[derive(Clone, Default, Debug)]
pub struct Text {
    text: String,
    element_data: ElementData,
    selectable: bool,
}

pub struct TextState<'a> {
    pub cached_editor: CachedEditor<'a>,
}

impl Text {
    pub fn new(text: &str) -> Text {
        Text {
            text: text.to_string(),
            element_data: Default::default(),
            selectable: true,
        }
    }

    pub fn disable_selection(mut self) -> Self {
        self.selectable = false;
        self
    }

    #[allow(dead_code)]
    fn get_state<'a>(&self, element_state: &'a ElementStateStore) -> &'a TextState {
        element_state.storage.get(&self.element_data.component_id).unwrap().data.as_ref().downcast_ref().unwrap()
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
        _font_system: &mut FontSystem,
        _taffy_tree: &mut TaffyTree<LayoutContext>,
        _root_node: NodeId,
        element_state: &mut ElementStateStore,
        _pointer: Option<Point>,
        _window: Option<Arc<dyn Window>>,
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

        let cached_editor = &mut state.cached_editor;

        let editor = &cached_editor.editor;
        let buffer = &cached_editor.get_last_cache_entry().buffer;

        let fill_color = self.element_data.style.color();
        let text_scroll = None;
        
        let buffer_glyphs = crate::renderer::text::create_glyphs_for_editor(
            buffer,
            editor,
            fill_color,
            Color::from_rgb8(0, 0, 0),
            Color::from_rgb8(0, 120, 215),
            Color::from_rgb8(255, 255, 255),
            text_scroll,
        );
        
        renderer.draw_text(buffer_glyphs, content_rectangle, None, false);
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
        _element_state: &mut ElementStateStore,
        _pointer: Option<Point>,
        _font_system: &mut FontSystem,
    ) {
        let result = taffy_tree.layout(root_node).unwrap();
        self.resolve_box(position, transform, result, z_index);

        self.finalize_borders();
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn on_event(
        &self,
        message: &CraftMessage,
        element_state: &mut ElementStateStore,
        font_system: &mut FontSystem,
    ) -> UpdateResult {
        let state: &mut TextState = element_state
            .storage
            .get_mut(&self.element_data.component_id)
            .unwrap()
            .data
            .as_mut()
            .downcast_mut()
            .unwrap();

        let cached_editor = &mut state.cached_editor;
        let content_rect = self.element_data.computed_box.content_rectangle();
        let content_position = content_rect.position();

        // Handle selection.
        if self.selectable {
            match message {
                CraftMessage::PointerButtonEvent(pointer_button) => {
                    let pointer_position = pointer_button.position;
                    let pointer_content_position = pointer_position - content_position;
                    if pointer_button.state.is_pressed() && content_rect.contains(&pointer_button.position) {
                        cached_editor.action_start_drag(
                            font_system,
                            Point::new(pointer_content_position.x, pointer_content_position.y),
                        );
                    } else {
                        cached_editor.action_end_drag();
                    }
                    UpdateResult::new().prevent_defaults().prevent_propagate()
                }
                CraftMessage::PointerMovedEvent(moved) => {
                    if cached_editor.dragging {
                        let pointer_position = moved.position;
                        let pointer_content_position = pointer_position - content_position;
                        cached_editor.action_drag(
                            font_system,
                            Point::new(pointer_content_position.x, pointer_content_position.y),
                        );
                    }
                    UpdateResult::new().prevent_defaults().prevent_propagate()
                }
                CraftMessage::ModifiersChangedEvent(modifiers_changed) => {
                    cached_editor.action_modifiers_changed(*modifiers_changed);
                    UpdateResult::new().prevent_defaults().prevent_propagate()
                }
                CraftMessage::KeyboardInputEvent(keyboard_input) => {
                    let logical_key = keyboard_input.clone().event.logical_key;
                    let key_state = keyboard_input.event.state;

                    if !key_state.is_pressed() {
                        return UpdateResult::new();
                    }

                    if let Key::Character(text) = logical_key {
                        if cached_editor.is_control_or_super_modifier_pressed() && text == "c" {
                            cached_editor.action_copy_to_clipboard()
                        }
                    }

                    UpdateResult::new().prevent_defaults().prevent_propagate()
                }
                _ => UpdateResult::new(),
            }
        } else {
            UpdateResult::default()
        }
    }

    fn initialize_state(&self, font_system: &mut FontSystem, scaling_factor: f64) -> ElementStateStoreItem {
        let cached_editor = CachedEditor::new(&self.text, &self.element_data.style, scaling_factor, font_system);
        let text_state = TextState { cached_editor };

        ElementStateStoreItem {
            base: Default::default(),
            data: Box::new(text_state),
        }
    }

    fn update_state(
        &self,
        font_system: &mut FontSystem,
        element_state: &mut ElementStateStore,
        reload_fonts: bool,
        scaling_factor: f64,
    ) {
        let state: &mut TextState = element_state
            .storage
            .get_mut(&self.element_data.component_id)
            .unwrap()
            .data
            .as_mut()
            .downcast_mut()
            .unwrap();

        state.cached_editor.update_state(
            Some(&self.text),
            &self.element_data.style,
            scaling_factor,
            reload_fonts,
            font_system,
        );
    }
}

impl Text {
    generate_component_methods_no_children!();
}

impl ElementStyles for Text {
    fn styles_mut(&mut self) -> &mut Style {
        self.element_data.current_style_mut()
    }
}
