use crate::components::component::ComponentSpecification;
use crate::components::Props;
use crate::components::UpdateResult;
use crate::elements::common_element_data::CommonElementData;
use crate::elements::element::{Element, ElementBox};
use crate::elements::layout_context::{LayoutContext, TaffyTextInputContext};
use crate::elements::scroll_state::ScrollState;
use crate::elements::ElementStyles;
use crate::events::OkuMessage;
use crate::geometry::{Point, Size};
use crate::reactive::element_state_store::{ElementStateStore, ElementStateStoreItem};
use crate::renderer::color::Color;
use crate::renderer::renderer::TextScroll;
use crate::style::{Display, Style, Unit};
use crate::text::cached_editor::CachedEditor;
use crate::{generate_component_methods_no_children, RendererBox};
use cosmic_text::FontSystem;
use cosmic_text::{Cursor, Edit};
use std::any::Any;
use std::sync::Arc;
use taffy::{NodeId, TaffyTree};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::Ime;
use winit::keyboard::{Key, NamedKey};
use winit::window::Window;

// A stateful element that shows text.
#[derive(Clone, Default, Debug)]
pub struct TextInput {
    text: String,
    common_element_data: CommonElementData,
    /// Whether the text input will update the editor every update with the user provided text.
    /// NOTE: The editor will always use the user provided text on initialization.
    use_text_value_on_update: bool,
}

#[derive(Clone, Default, Debug)]
pub(crate) struct ImeState {
    pub is_ime_active: bool,
    pub ime_starting_cursor: Option<Cursor>,
    pub ime_ending_cursor: Option<Cursor>,
}

pub struct TextInputState<'a> {
    pub cached_editor: CachedEditor<'a>,
    pub is_active: bool,
    pub(crate) scroll_state: ScrollState,
    pub(crate) ime_state: ImeState
}

impl TextInput {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            common_element_data: CommonElementData::default(),
            use_text_value_on_update: true,
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
        element_state: &mut ElementStateStore,
        _pointer: Option<Point>,
        window: Option<Arc<dyn Window>>
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
        
        let text_scroll = if is_scrollable {
            Some(TextScroll::new(scroll_y, self.common_element_data.computed_scroll_track.height))
        } else {
            None
        };

        renderer.draw_text(
            self.common_element_data.component_id,
            content_rectangle,
            self.common_element_data.style.color(),
            text_scroll
        );

        if let Some(state) = element_state
            .storage
            .get_mut(&self.common_element_data.component_id)
            .unwrap()
            .data
            .downcast_mut::<TextInputState>()
        {

            if let Some((cursor_x, cursor_y)) = state.cached_editor.editor.cursor_position() {
                if state.is_active {
                    if let Some(window) = window {
                        let content_position = self.common_element_data.computed_layered_rectangle_transformed.content_rectangle();
                        window.set_ime_cursor_area(
                            PhysicalPosition::new(content_position.x + cursor_x as f32, content_position.y + cursor_y as f32).into(),
                            PhysicalSize::new(20.0, 20.0).into(),
                        );
                    }
                }
            }

            state.is_active = false;
        }

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
        let state = base_state.data.as_mut().downcast_mut::<TextInputState>().unwrap();
        state.is_active = true;

        let scroll_result = state.scroll_state.on_event(&message, &self.common_element_data, &mut base_state.base);

        if !scroll_result.propagate {
            return scroll_result;
        }

        let cached_editor = &mut state.cached_editor;
        let scroll_y = state.scroll_state.scroll_y;
        let content_rect = self.common_element_data.computed_layered_rectangle.content_rectangle();
        
        match message {
            OkuMessage::PointerButtonEvent(pointer_button) => {
                let pointer_position = pointer_button.position;
                let pointer_content_position = pointer_position - content_rect.position();
                
                if pointer_button.state.is_pressed() && content_rect.contains(&pointer_button.position) {
                    cached_editor.action_start_drag(font_system, Point::new(pointer_content_position.x, pointer_content_position.y + scroll_y));
                } else {
                    cached_editor.action_end_drag();
                }
                UpdateResult::new().prevent_defaults().prevent_propagate()
            }
            OkuMessage::PointerMovedEvent(moved) => {
                if cached_editor.dragging {
                    let pointer_position = moved.position;
                    let pointer_content_position = pointer_position - content_rect.position();
                    cached_editor.action_drag(font_system, Point::new(pointer_content_position.x, pointer_content_position.y + scroll_y));
                }
                UpdateResult::new().prevent_defaults().prevent_propagate()
            }
            OkuMessage::ModifiersChangedEvent(modifiers_changed) => {
                cached_editor.action_modifiers_changed(modifiers_changed);
                UpdateResult::new().prevent_defaults().prevent_propagate()
            }
            OkuMessage::KeyboardInputEvent(keyboard_input) => {
                let logical_key = keyboard_input.event.logical_key;
                let key_state = keyboard_input.event.state;

                if key_state.is_pressed() {
                    match logical_key {
                        Key::Named(NamedKey::ArrowLeft) => cached_editor.move_left(font_system),
                        Key::Named(NamedKey::ArrowRight) => cached_editor.move_right(font_system),
                        Key::Named(NamedKey::ArrowUp) => cached_editor.move_up(font_system),
                        Key::Named(NamedKey::ArrowDown) => cached_editor.move_down(font_system),
                        Key::Named(NamedKey::Home) => cached_editor.move_to_start(font_system),
                        Key::Named(NamedKey::End) => cached_editor.move_to_end(font_system),
                        Key::Named(NamedKey::PageUp) => cached_editor.move_page_up(font_system),
                        Key::Named(NamedKey::PageDown) => cached_editor.move_page_down(font_system),
                        
                        Key::Named(NamedKey::Escape) => cached_editor.action_escape(font_system),
                        Key::Named(NamedKey::Enter) => cached_editor.action_enter(font_system),
                        Key::Named(NamedKey::Backspace) => cached_editor.action_backspace(font_system),
                        Key::Named(NamedKey::Delete) => cached_editor.action_delete(font_system),
                        Key::Named(key) => {
                            if let Some(text) = key.to_text() {
                                cached_editor.action_insert(font_system, text.chars());
                            }
                        }
                        Key::Character(text) => {
                            if cached_editor.is_control_or_super_modifier_pressed() && matches!(text.as_str(), "c" | "v" | "x") {
                                match text.to_lowercase().as_str() {
                                    "c" => cached_editor.action_copy_to_clipboard(),
                                    "v" => cached_editor.action_paste_from_clipboard(font_system),
                                    "x" => cached_editor.action_cut_from_clipboard(),
                                    _ => (),
                                }
                            } else {
                                cached_editor.action_insert(font_system, text.chars());
                            }
                        }
                        _ => {}
                    }
                }

                let event_text = cached_editor.get_text();
                UpdateResult::new()
                    .prevent_defaults()
                    .prevent_propagate()
                    .result_message(OkuMessage::TextInputChanged(event_text))
            }

            // This is all a bit hacky and needs some improvement:
            OkuMessage::ImeEvent(ime) => {
                // FIXME: This shouldn't be possible, we need to close the ime window when a text input loses focus.
                if state.ime_state.ime_starting_cursor.is_none() && !matches!(ime, Ime::Enabled){
                    // state.ime_starting_cursor = Some(cached_editor.editor.cursor());
                    return Default::default();
                }

                match ime {
                    Ime::Enabled => {
                        state.ime_state = cached_editor.action_ime_enabled();
                    }
                    Ime::Preedit(str, cursor_info) => {
                        state.ime_state = cached_editor.action_ime_preedit(&state.ime_state, &str, cursor_info);
                    }
                    Ime::Commit(str) => {
                       state.ime_state = cached_editor.action_ime_commit(&state.ime_state, &str);
                    }
                    Ime::Disabled => {
                        state.ime_state = cached_editor.action_ime_disabled();
                    }
                };

                let event_text = cached_editor.get_text();
                UpdateResult::new()
                    .prevent_defaults()
                    .prevent_propagate()
                    .result_message(OkuMessage::TextInputChanged(event_text))
            }
            
            _ => UpdateResult::new(),
        }
    }

    fn initialize_state(&self, font_system: &mut FontSystem, scaling_factor: f64) -> ElementStateStoreItem {
        let cached_editor = CachedEditor::new(&self.text, &self.common_element_data.style, scaling_factor, font_system);
        let text_input_state = TextInputState {
            cached_editor,
            ime_state: ImeState::default(),
            is_active: false,
            scroll_state: ScrollState::default(),
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

        if self.use_text_value_on_update {
            state.cached_editor.update_state(Some(&self.text), &self.common_element_data.style, scaling_factor, reload_fonts, font_system);   
        } else {
            state.cached_editor.update_state(None, &self.common_element_data.style, scaling_factor, reload_fonts, font_system);
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

    /// Whether the text input will update the editor every update with the user provided text.
    /// NOTE: The editor will always use the user provided text on initialization.
    pub fn use_text_value_on_update(mut self, use_initial_text_value: bool) -> Self {
        self.use_text_value_on_update = use_initial_text_value;
        self
    }
}

impl ElementStyles for TextInput {
    fn styles_mut(&mut self) -> &mut Style {
        self.common_element_data.current_style_mut()
    }
}
