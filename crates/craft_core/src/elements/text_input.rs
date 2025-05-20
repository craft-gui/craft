use crate::CraftMessage;
use crate::components::component::ComponentSpecification;
use crate::components::{ImeAction, Props};
use crate::components::Event;
use crate::elements::element::{resolve_clip_for_scrollable, Element, ElementBoxed};
use crate::elements::element_data::ElementData;
use crate::layout::layout_context::{LayoutContext, TaffyTextInputContext};
use crate::elements::scroll_state::ScrollState;
use crate::elements::ElementStyles;
use crate::geometry::{Point, Rectangle, Size, TrblRectangle};
use crate::reactive::element_state_store::{ElementStateStore, ElementStateStoreItem};
use crate::renderer::color::Color;
use crate::renderer::renderer::{RenderList, TextScroll};
use crate::style::{Display, Style, Unit};
use crate::{generate_component_methods_no_children};
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use parley::{PlainEditor, PlainEditorDriver};
use taffy::{AvailableSpace, NodeId, TaffyTree};

#[cfg(target_arch = "wasm32")]
use web_time as time;
#[cfg(not(target_arch = "wasm32"))]
use std::time as time;
use time::{Duration, Instant};

use winit::event::{Ime, Modifiers};
use winit::keyboard::{Key, NamedKey};
use winit::window::Window;
use crate::layout::layout_context::TextHashKey;
use crate::text::text_context::{ColorBrush, TextContext};
use crate::text::text_render_data;
use crate::text::text_render_data::TextRender;

// A stateful element that shows text.
#[derive(Clone, Default)]
pub struct TextInput {
    element_data: ElementData,
    /// Whether the text input will update the editor every update with the user provided text.
    /// NOTE: The editor will always use the user provided text on initialization.
    use_text_value_on_update: bool,
    pub text: Option<String>,
}

#[derive(Clone, Default, Debug)]
pub(crate) struct ImeState {
    #[allow(dead_code)]
    pub is_ime_active: bool,
}

pub struct TextInputState {
    pub is_active: bool,
    pub(crate) scroll_state: ScrollState,
    #[allow(dead_code)]
    pub(crate) ime_state: ImeState,
    pub(crate) editor: PlainEditor<ColorBrush>,

    cache: HashMap<TextHashKey, taffy::Size<f32>>,
    current_key: Option<TextHashKey>,
    last_requested_key: Option<TextHashKey>,
    text_render: Option<TextRender>,
    new_text: Option<String>,

    last_click_time: Option<Instant>,
    click_count: u32,
    pointer_down: bool,
    cursor_pos: (f32, f32),
    cursor_visible: bool,
    modifiers: Option<Modifiers>,
    start_time: Option<Instant>,
    blink_period: Duration,
}

impl TextInput {
    pub fn new(text: &str) -> Self {
        Self {
            text: Some(text.to_string()),
            element_data: ElementData::default(),
            use_text_value_on_update: true,
        }
    }

    #[allow(dead_code)]
    fn get_state<'a>(&self, element_state: &'a ElementStateStore) -> &'a TextInputState {
        element_state.storage.get(&self.element_data.component_id).unwrap().data.as_ref().downcast_ref().unwrap()
    }
}

impl Element for TextInput {
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
        "TextInput"
    }

    fn draw(
        &mut self,
        renderer: &mut RenderList,
        _text_context: &mut TextContext,
        _taffy_tree: &mut TaffyTree<LayoutContext>,
        _root_node: NodeId,
        element_state: &mut ElementStateStore,
        _pointer: Option<Point>,
        _window: Option<Arc<dyn Window>>,
    ) {
        if !self.element_data.style.visible() {
            return;
        }
        let computed_box_transformed = self.element_data.layout_item.computed_box_transformed;
        let content_rectangle = computed_box_transformed.content_rectangle();

        self.draw_borders(renderer, element_state);

        let is_scrollable = self.element_data.is_scrollable();

        let element_data = self.element_data();
        let padding_rectangle = element_data.layout_item.computed_box_transformed.padding_rectangle();
        renderer.push_layer(padding_rectangle);

        let scroll_y = if let Some(state) =
            element_state.storage.get(&self.element_data.component_id).unwrap().data.downcast_ref::<TextInputState>()
        {
            state.scroll_state.scroll_y
        } else {
            0.0
        };

        let text_scroll = if is_scrollable {
            Some(TextScroll::new(scroll_y, self.element_data.layout_item.computed_scroll_track.height))
        } else {
            None
        };

        if let Some(state) =
            element_state.storage.get_mut(&self.element_data.component_id).unwrap().data.downcast_mut::<TextInputState>()
        {
            if let Some(text_render) = state.text_render.as_ref() {
                renderer.draw_text(text_render.clone(), content_rectangle, text_scroll, state.cursor_visible);
            }
        }

        renderer.pop_layer();

        self.draw_scrollbar(renderer);
    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        _element_state: &mut ElementStateStore,
        scale_factor: f64,
    ) -> Option<NodeId> {
        self.merge_default_style();
        self.element_data.style.scale(scale_factor);
        let style: taffy::Style = self.element_data.style.to_taffy_style();

        self.element_data.layout_item.build_tree_with_context(
            taffy_tree,
            style,
            LayoutContext::TextInput(TaffyTextInputContext::new(self.element_data.component_id))
        )
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
        text_context: &mut TextContext,
        clip_bounds: Option<Rectangle>,
    ) {
        let result = taffy_tree.layout(root_node).unwrap();
        self.resolve_box(position, transform, result, z_index);
        self.resolve_clip(clip_bounds);

        self.finalize_borders(element_state);

        let state: &mut TextInputState = element_state
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
                text_context,
            );
        }

        let _layout = state.editor.try_layout().as_ref().unwrap();
        let text_renderer = state.text_render.as_mut().unwrap();
        for line in text_renderer.lines.iter_mut() {
            line.selections.clear();
        }
        state.editor.selection_geometry_with( |rect, line| {
            text_renderer.lines[line].selections.push(rect.into());
        });
        text_renderer.cursor = state.editor.cursor_geometry(1.0).map(|r| r.into());

        self.element_data.layout_item.scrollbar_size = Size::new(result.scrollbar_size.width, result.scrollbar_size.height);
        self.element_data.layout_item.computed_scrollbar_size = Size::new(result.scroll_width(), result.scroll_height());

        let scroll_y = if let Some(state) =
            element_state.storage.get(&self.element_data.component_id).unwrap().data.downcast_ref::<TextInputState>()
        {
            state.scroll_state.scroll_y
        } else {
            0.0
        };

        self.finalize_scrollbar(scroll_y);
    }

    fn resolve_clip(&mut self, clip_bounds: Option<Rectangle>) {
        resolve_clip_for_scrollable(self, clip_bounds);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn on_event(
        &self,
        message: &CraftMessage,
        element_state: &mut ElementStateStore,
        _text_context: &mut TextContext,
        should_style: bool,
        event: &mut Event,
    ) {
        self.on_style_event(message, element_state, should_style, event);

        let base_state = self.get_base_state_mut(element_state);
        let state = base_state.data.as_mut().downcast_mut::<TextInputState>().unwrap();
        state.is_active = true;

        state.scroll_state.on_event(message, &self.element_data, &mut base_state.base, event);

        if !event.propagate {
            return;
        }

        let scroll_y = state.scroll_state.scroll_y;

        let text_position = self.element_data().layout_item.computed_box_transformed.content_rectangle();
        let text_x = text_position.x;
        let text_y = text_position.y;
        let state: &mut TextInputState = element_state
            .storage
            .get_mut(&self.element_data.component_id)
            .unwrap()
            .data
            .as_mut()
            .downcast_mut()
            .unwrap();
        match message {
            CraftMessage::ModifiersChangedEvent(modifiers) => {
                state.modifiers = Some(*modifiers);
            }
            CraftMessage::KeyboardInputEvent(keyboard_input) if !state.editor.is_composing() => {
                if !keyboard_input.event.state.is_pressed() {
                    return;
                }

                state.cursor_reset();
                #[allow(unused)]
                let (shift, action_mod) = state
                    .modifiers
                    .map(|mods| {
                        (
                            mods.state().shift_key(),
                            if cfg!(target_os = "macos") {
                                mods.state().meta_key()
                            } else {
                                mods.state().control_key()
                            },
                        )
                    })
                    .unwrap_or_default();

                let mut drv = state.driver(_text_context);
                match &keyboard_input.event.logical_key {
                    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
                    Key::Character(c) if action_mod && matches!(c.as_str(), "c" | "x" | "v") => {
                        use clipboard_rs::{Clipboard, ClipboardContext};
                        match c.to_lowercase().as_str() {
                            "c" => {
                                if let Some(text) = drv.editor.selected_text() {
                                    let cb = ClipboardContext::new().unwrap();
                                    cb.set_text(text.to_owned()).ok();
                                }
                            }
                            "x" => {
                                if let Some(text) = drv.editor.selected_text() {
                                    let cb = ClipboardContext::new().unwrap();
                                    cb.set_text(text.to_owned()).ok();
                                    drv.delete_selection();
                                    state.cache.clear();
                                }
                            }
                            "v" => {
                                let cb = ClipboardContext::new().unwrap();
                                let text = cb.get_text().unwrap_or_default();
                                drv.insert_or_replace_selection(&text);
                                state.cache.clear();
                            }
                            _ => (),
                        }
                    }
                    Key::Character(c) if action_mod && matches!(c.to_lowercase().as_str(), "a") => {
                        if shift {
                            drv.collapse_selection();
                        } else {
                            drv.select_all();
                        }
                    }
                    Key::Named(NamedKey::ArrowLeft) => {
                        if action_mod {
                            if shift {
                                drv.select_word_left();
                            } else {
                                drv.move_word_left();
                            }
                        } else if shift {
                            drv.select_left();
                        } else {
                            drv.move_left();
                        }
                    }
                    Key::Named(NamedKey::ArrowRight) => {
                        if action_mod {
                            if shift {
                                drv.select_word_right();
                            } else {
                                drv.move_word_right();
                            }
                        } else if shift {
                            drv.select_right();
                        } else {
                            drv.move_right();
                        }
                    }
                    Key::Named(NamedKey::ArrowUp) => {
                        if shift {
                            drv.select_up();
                        } else {
                            drv.move_up();
                        }
                    }
                    Key::Named(NamedKey::ArrowDown) => {
                        if shift {
                            drv.select_down();
                        } else {
                            drv.move_down();
                        }
                    }
                    Key::Named(NamedKey::Home) => {
                        if action_mod {
                            if shift {
                                drv.select_to_text_start();
                            } else {
                                drv.move_to_text_start();
                            }
                        } else if shift {
                            drv.select_to_line_start();
                        } else {
                            drv.move_to_line_start();
                        }
                    }
                    Key::Named(NamedKey::End) => {
                        let mut drv = state.driver(_text_context);

                        if action_mod {
                            if shift {
                                drv.select_to_text_end();
                            } else {
                                drv.move_to_text_end();
                            }
                        } else if shift {
                            drv.select_to_line_end();
                        } else {
                            drv.move_to_line_end();
                        }
                    }
                    Key::Named(NamedKey::Delete) => {
                        if action_mod {
                            drv.delete_word();
                            state.cache.clear();
                        } else {
                            drv.delete();
                            state.cache.clear();
                        }
                    }
                    Key::Named(NamedKey::Backspace) => {
                        if action_mod {
                            drv.backdelete_word();
                            state.cache.clear();
                        } else {
                            drv.backdelete();
                            state.cache.clear();
                        }
                    }
                    Key::Named(NamedKey::Enter) => {
                        drv.insert_or_replace_selection("\n");
                        state.cache.clear();
                    }
                    Key::Character(s) => {
                        drv.insert_or_replace_selection(s);
                        state.cache.clear();
                    }
                    _ => (),
                }


                // FIXME: This is more of a hack, we should be doing this somewhere else.
                event.prevent_defaults();
                event.prevent_propagate();
                event.result_message(CraftMessage::TextInputChanged(state.editor.text().to_string()))
            }
            // WindowEvent::Touch(Touch {
            //     phase, location, ..
            // }) if !self.editor.is_composing() => {
            //     let mut drv = self.editor.driver(&mut self.font_cx, &mut self.layout_cx);
            //     use winit::event::TouchPhase::*;
            //     match phase {
            //         Started => {
            //             // TODO: start a timer to convert to a SelectWordAtPoint
            //             drv.move_to_point(location.x as f32, location.y as f32);
            //         }
            //         Cancelled => {
            //             drv.collapse_selection();
            //         }
            //         Moved => {
            //             // TODO: cancel SelectWordAtPoint timer
            //             drv.extend_selection_to_point(
            //                 location.x as f32,
            //                 location.y as f32,
            //             );
            //         }
            //         Ended => (),
            //     }
            // }
            CraftMessage::PointerButtonEvent(pointer_button) => {
                if pointer_button.button.mouse_button() == winit::event::MouseButton::Left {
                    state.pointer_down = pointer_button.state.is_pressed();
                    state.cursor_reset();
                    if state.pointer_down && !state.editor.is_composing() {
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
                        let mut drv = state.driver(_text_context);
                        match click_count {
                            2 => drv.select_word_at_point(cursor_pos.0, cursor_pos.1),
                            3 => drv.select_line_at_point(cursor_pos.0, cursor_pos.1),
                            _ => drv.move_to_point(cursor_pos.0, cursor_pos.1),
                        }
                    }
                }
            }
            CraftMessage::PointerMovedEvent(pointer_moved) => {
                let prev_pos = state.cursor_pos;
                // NOTE: Cursor position should be relative to the top left of the text box.
                state.cursor_pos = (pointer_moved.position.x - text_x, pointer_moved.position.y - text_y + scroll_y);
                // macOS seems to generate a spurious move after selecting word?
                if state.pointer_down && prev_pos != state.cursor_pos && !state.editor.is_composing() {
                    state.cursor_reset();
                    let cursor_pos = state.cursor_pos;
                    state.driver(_text_context).extend_selection_to_point(cursor_pos.0, cursor_pos.1);
                }
            }
            CraftMessage::ImeEvent(Ime::Disabled) => {
                state.driver(_text_context).clear_compose();
                state.cache.clear();
            }
            CraftMessage::ImeEvent(Ime::Commit(text)) => {
                state.driver(_text_context).insert_or_replace_selection(text);
                state.cache.clear();
            }
            CraftMessage::ImeEvent(Ime::Preedit(text, cursor)) => {
                if text.is_empty() {
                    state.driver(_text_context).clear_compose();
                } else {
                    state.driver(_text_context).set_compose(text, *cursor);
                }
                state.cache.clear();
            }
            _ => {}
        }
        let ime = state.editor.ime_cursor_area();
        event.ime_action(ImeAction::Set(Rectangle::new(ime.x0 as f32, ime.y0 as f32, ime.width() as f32, ime.height() as f32)));
    }

    fn initialize_state(&mut self, scaling_factor: f64) -> ElementStateStoreItem {
        let mut editor = PlainEditor::new(self.style().font_size());
        editor.set_scale(scaling_factor as f32);
        let style_set = editor.edit_styles();
        self.style().add_styles_to_style_set(style_set);
        
        let text_input_state = TextInputState {
            ime_state: ImeState::default(),
            is_active: false,
            scroll_state: ScrollState::default(),
            editor,
            cache: Default::default(),
            current_key: None,
            last_requested_key: None,
            text_render: None,
            new_text: std::mem::take(&mut self.text),
            last_click_time: None,
            click_count: 0,
            pointer_down: false,
            cursor_pos: (0.0, 0.0),
            cursor_visible: false,
            modifiers: None,
            start_time: None,
            blink_period: Default::default(),
        };

        ElementStateStoreItem {
            base: Default::default(),
            data: Box::new(text_input_state),
        }
    }

    fn update_state(
        &mut self,
        element_state: &mut ElementStateStore,
        _reload_fonts: bool,
        scaling_factor: f64,
    ) {
        let state: &mut TextInputState = element_state
            .storage
            .get_mut(&self.element_data.component_id)
            .unwrap()
            .data
            .as_mut()
            .downcast_mut()
            .unwrap();

        if let Some(layout) = state.editor.try_layout() {
            if layout.scale() != scaling_factor as f32 {
                state.editor.set_scale(scaling_factor as f32);
                state.cache.clear();
                state.new_text = Some(state.editor.text().to_string());
            }
        }
    }

    fn default_style(&self) -> Style {
        let mut style = Style::default();
        *style.display_mut() = Display::Block;
        const BORDER_COLOR: Color = Color::from_rgb8(199, 199, 206);
        *style.border_color_mut() = TrblRectangle::new_all(BORDER_COLOR);
        *style.border_width_mut() = TrblRectangle::new_all(Unit::Px(1.0));
        *style.border_radius_mut() = [(5.0, 5.0); 4];
        let padding = Unit::Px(4.0);
        *style.padding_mut() = TrblRectangle::new_all(padding);

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
        self.element_data.current_style_mut()
    }
}


impl TextInputState {
    pub fn measure(
        &mut self,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<AvailableSpace>,
        text_context: &mut TextContext,
    ) -> taffy::Size<f32> {
        if self.editor.try_layout().is_none() || self.new_text.is_some() {
            let text = std::mem::take(&mut self.new_text).unwrap();
            self.editor.set_text(text.as_str());
            self.editor.refresh_layout(&mut text_context.font_context, &mut text_context.layout_context);
        }
        self.editor.refresh_layout(&mut text_context.font_context, &mut text_context.layout_context);

        let key = TextHashKey::new(known_dimensions, available_space);

        self.last_requested_key = Some(key);

        if let Some(value) = self.cache.get(&key) {
            return *value;
        }

        self.layout(known_dimensions, available_space, text_context)
    }

    pub fn layout(
        &mut self,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<AvailableSpace>,
        text_context: &mut TextContext,
    ) -> taffy::Size<f32> {
        let key = TextHashKey::new(known_dimensions, available_space);

        if let Some(value) = self.cache.get(&key) {
            return *value;
        }

        let width_constraint = known_dimensions.width.or(match available_space.width {
            AvailableSpace::MinContent => Some(self.editor.try_layout().unwrap().min_content_width()),
            AvailableSpace::MaxContent => Some(self.editor.try_layout().unwrap().max_content_width()),
            AvailableSpace::Definite(width) => Some(width),
        });
        // Some(self.text_style.font_size * self.text_style.line_height)
        let _height_constraint = known_dimensions.height.or(match available_space.height {
            AvailableSpace::MinContent => None,
            AvailableSpace::MaxContent => None,
            AvailableSpace::Definite(height) => Some(height),
        });

        self.editor.set_width(width_constraint);
        self.editor.refresh_layout(&mut text_context.font_context, &mut text_context.layout_context);

        let layout = self.editor.try_layout().unwrap();
        let width = layout.width();
        let height = layout.height();

        self.text_render = Some(text_render_data::from_editor(layout));

        let size = taffy::Size { width, height };

        self.cache.insert(key, size);
        self.current_key = Some(key);
        size
    }

    pub fn cursor_reset(&mut self) {
        self.start_time = Some(Instant::now());
        // TODO: for real world use, this should be reading from the system settings
        self.blink_period = Duration::from_millis(500);
        self.cursor_visible = true;
    }

    #[allow(dead_code)]
    pub fn disable_blink(&mut self) {
        self.start_time = None;
    }

    #[allow(dead_code)]
    pub fn next_blink_time(&self) -> Option<Instant> {
        self.start_time.map(|start_time| {
            let phase = Instant::now().duration_since(start_time);

            start_time
                + Duration::from_nanos(
                ((phase.as_nanos() / self.blink_period.as_nanos() + 1)
                    * self.blink_period.as_nanos()) as u64,
            )
        })
    }

    #[allow(dead_code)]
    pub fn cursor_blink(&mut self) {
        self.cursor_visible = self.start_time.is_some_and(|start_time| {
            let elapsed = Instant::now().duration_since(start_time);
            (elapsed.as_millis() / self.blink_period.as_millis()) % 2 == 0
        });
    }

    fn driver<'a>(&'a mut self, text_context: &'a mut TextContext) -> PlainEditorDriver<'a, ColorBrush> {
        self.editor.driver(&mut text_context.font_context, &mut text_context.layout_context)
    }

}