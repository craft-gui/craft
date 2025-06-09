use crate::components::component::ComponentSpecification;
use crate::components::{Event, FocusAction};
use crate::components::{ImeAction, Props};
use crate::elements::element::{resolve_clip_for_scrollable, Element, ElementBoxed};
use crate::elements::element_data::ElementData;
use crate::elements::scroll_state::ScrollState;
use crate::elements::ElementStyles;
use crate::generate_component_methods_no_children;
use crate::geometry::{Point, Rectangle, Size, TrblRectangle};
use crate::layout::layout_context::{LayoutContext, TaffyTextInputContext};
use crate::reactive::element_state_store::{ElementStateStore, ElementStateStoreItem};
use crate::renderer::color::Color;
use crate::renderer::renderer::{RenderList, TextScroll};
use crate::style::{Display, Style, Unit};
use crate::CraftMessage;
use parley::{PlainEditor, PlainEditorDriver, StyleProperty};
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use taffy::{AvailableSpace, NodeId, TaffyTree};

use crate::layout::layout_context::TextHashKey;
use crate::text::text_context::{ColorBrush, TextContext};
use crate::text::text_render_data::TextRender;
use crate::text::{text_render_data, TextStyle};
#[cfg(not(target_arch = "wasm32"))]
use std::time;
use time::{Duration, Instant};
use kurbo::Affine;
use ui_events::keyboard::{Key, Modifiers, NamedKey};
use winit::dpi;
#[cfg(target_arch = "wasm32")]
use web_time as time;
use winit::event::Ime;
use winit::window::Window;
use crate::elements::text::TextState;
use crate::reactive::element_id::create_unique_element_id;

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

/// An external message that allows others to command the TextInput.
pub enum TextInputMessage {
    Copy,
    Paste,
    Cut,
    // TODO: Add more messages.
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
    new_style: TextStyle,

    last_click_time: Option<Instant>,
    click_count: u32,
    pointer_down: bool,
    cursor_pos: Point,
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
        element_state: &mut ElementStateStore,
        _pointer: Option<Point>,
        _window: Option<Arc<Window>>,
        scale_factor: f64,
    ) {
        if !self.element_data.style.visible() {
            return;
        }
        let computed_box_transformed = self.computed_box_transformed();
        let content_rectangle = computed_box_transformed.content_rectangle();

        self.draw_borders(renderer, element_state, scale_factor);

        let is_scrollable = self.element_data.is_scrollable();

        let element_data = self.element_data();
        let padding_rectangle = element_data.layout_item.computed_box_transformed.padding_rectangle();
        renderer.push_layer(padding_rectangle.scale(scale_factor));

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

        if let Some(state) = element_state
            .storage
            .get_mut(&self.element_data.component_id)
            .unwrap()
            .data
            .downcast_mut::<TextInputState>()
        {
            if let Some(text_render) = state.text_render.as_ref() {
                renderer.draw_text(text_render.clone(), content_rectangle.scale(scale_factor), text_scroll, state.cursor_visible);
            }
        }

        renderer.pop_layer();

        self.draw_scrollbar(renderer, scale_factor);
    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        _element_state: &mut ElementStateStore,
        _scale_factor: f64,
    ) -> Option<NodeId> {
        self.merge_default_style();
        let style: taffy::Style = self.element_data.style.to_taffy_style();

        self.element_data.layout_item.build_tree_with_context(
            taffy_tree,
            style,
            LayoutContext::TextInput(TaffyTextInputContext::new(self.element_data.component_id)),
        )
    }

    fn finalize_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        position: Point,
        z_index: &mut u32,
        transform: Affine,
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
        state.editor.selection_geometry_with(|rect, line| {
            text_renderer.lines[line].selections.push(rect.into());
        });
        text_renderer.cursor = state.editor.cursor_geometry(1.0).map(|r| r.into());

        self.element_data.layout_item.scrollbar_size =
            Size::new(result.scrollbar_size.width, result.scrollbar_size.height);
        self.element_data.layout_item.computed_scrollbar_size =
            Size::new(result.scroll_width(), result.scroll_height());

        if let Some(state) = element_state
            .storage
            .get_mut(&self.element_data.component_id)
            .unwrap()
            .data
            .downcast_mut::<TextInputState>()
        {
            self.finalize_scrollbar(&mut state.scroll_state);
        }
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
        self.maybe_unset_focus(message, event);

        let base_state = self.get_base_state_mut(element_state);
        let state = base_state.data.as_mut().downcast_mut::<TextInputState>().unwrap();
        state.is_active = true;

        state.scroll_state.on_event(message, &self.element_data, &mut base_state.base, event);

        if !event.propagate {
            return;
        }

        let scroll_y = state.scroll_state.scroll_y;

        let scale_factor = state.editor.try_layout().unwrap().scale() as f64;
        let text_position = self.computed_box_transformed().content_rectangle();
        let text_x = text_position.x;
        let text_y = text_position.y;
        let focused = element_state
            .storage
            .get(&self.element_data.component_id)
            .unwrap().base.focused;
        let state: &mut TextInputState = element_state
            .storage
            .get_mut(&self.element_data.component_id)
            .unwrap()
            .data
            .as_mut()
            .downcast_mut()
            .unwrap();

        fn copy(drv: &mut PlainEditorDriver<ColorBrush>) {
            #[cfg(all(any(target_os = "windows", target_os = "macos", target_os = "linux"), feature = "clipboard"))]
            {
                use clipboard_rs::{Clipboard, ClipboardContext};
                if let Some(text) = drv.editor.selected_text() {
                    let cb = ClipboardContext::new().unwrap();
                    cb.set_text(text.to_owned()).ok();
                }
            }
        }

        fn paste(drv: &mut PlainEditorDriver<ColorBrush>) {
            #[cfg(all(any(target_os = "windows", target_os = "macos", target_os = "linux"), feature = "clipboard"))]
            {
                use clipboard_rs::{Clipboard, ClipboardContext};
                let cb = ClipboardContext::new().unwrap();
                let text = cb.get_text().unwrap_or_default();
                drv.insert_or_replace_selection(&text);
            }
        }

        fn cut(drv: &mut PlainEditorDriver<ColorBrush>) {
            #[cfg(all(any(target_os = "windows", target_os = "macos", target_os = "linux"), feature = "clipboard"))]
            {
                use clipboard_rs::{Clipboard, ClipboardContext};
                if let Some(text) = drv.editor.selected_text() {
                    let cb = ClipboardContext::new().unwrap();
                    cb.set_text(text.to_owned()).ok();
                    drv.delete_selection();
                }
            }
        }

        let mut generate_text_changed_event = |editor: &mut PlainEditor<ColorBrush>| {
            event.prevent_defaults();
            event.prevent_propagate();
            event.result_message(CraftMessage::TextInputChanged(editor.text().to_string()));
        };

        if let CraftMessage::ElementMessage(msg) = message {
            if let Some(msg) = msg.downcast_ref::<TextInputMessage>() {
                let mut drv = state.driver(_text_context);
                match msg {
                    TextInputMessage::Copy => {
                        copy(&mut drv);
                    }
                    TextInputMessage::Paste => {
                        paste(&mut drv);
                        state.cache.clear();
                        generate_text_changed_event(&mut state.editor);
                    }
                    TextInputMessage::Cut => {
                        cut(&mut drv);
                        state.cache.clear();
                        generate_text_changed_event(&mut state.editor);
                    }
                }
            }
        }

        match message {
            CraftMessage::KeyboardInputEvent(keyboard_input) if !state.editor.is_composing() => {
                state.modifiers = Some(keyboard_input.modifiers);
                if !keyboard_input.state.is_down() {
                    return;
                }

                if !focused {
                    return;
                }

                state.cursor_reset();
                #[allow(unused)]
                let (shift, action_mod) = state
                    .modifiers
                    .map(|mods| (mods.shift(), if cfg!(target_os = "macos") { mods.meta() } else { mods.ctrl() }))
                    .unwrap_or_default();

                let mut drv = state.driver(_text_context);

                match &keyboard_input.key {
                    Key::Character(c) if action_mod && matches!(c.as_str(), "c" | "x" | "v") => {
                        match c.to_lowercase().as_str() {
                            "c" => copy(&mut drv),
                            "x" => {
                                cut(&mut drv);
                                state.cache.clear();
                                generate_text_changed_event(&mut state.editor);
                            }
                            "v" => {
                                paste(&mut drv);
                                state.cache.clear();
                                generate_text_changed_event(&mut state.editor);
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
                        generate_text_changed_event(&mut state.editor);
                    }
                    Key::Named(NamedKey::Backspace) => {
                        if action_mod {
                            drv.backdelete_word();
                            state.cache.clear();
                        } else {
                            drv.backdelete();
                            state.cache.clear();
                        }
                        generate_text_changed_event(&mut state.editor);
                    }
                    Key::Named(NamedKey::Enter) => {
                        drv.insert_or_replace_selection("\n");
                        state.cache.clear();
                        generate_text_changed_event(&mut state.editor);
                    }
                    Key::Character(s) => {
                        drv.insert_or_replace_selection(s);
                        state.cache.clear();
                        generate_text_changed_event(&mut state.editor);
                    }
                    _ => (),
                }
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
            CraftMessage::PointerButtonDown(pointer_button) => {
                if pointer_button.is_primary() {
                    event.focus_action(FocusAction::Set(self.component_id()));
                    state.pointer_down = true;
                    state.cursor_reset();
                    if !state.editor.is_composing() {
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
                        let cursor_x = cursor_pos.x as f32;
                        let cursor_y = cursor_pos.y as f32;
                        match click_count {
                            2 => drv.select_word_at_point(cursor_x, cursor_y),
                            3 => drv.select_line_at_point(cursor_x, cursor_y),
                            _ => drv.move_to_point(cursor_x, cursor_y),
                        }
                    }
                }
            }
            CraftMessage::PointerButtonUp(pointer_button) => {
                if pointer_button.is_primary() {
                    state.pointer_down = false;
                    state.cursor_reset();
                }
            }
            CraftMessage::PointerMovedEvent(pointer_moved) => {
                let prev_pos = state.cursor_pos;
                // NOTE: Cursor position should be relative to the top left of the text box.
                let cursor_pos = pointer_moved.current.position;
                let cursor_pos: Point = (
                    cursor_pos.x as f32 - text_x,
                    cursor_pos.y as f32 - text_y,
                ).into();
                let mut cursor_pos = Point::new(cursor_pos.x * scale_factor, cursor_pos.y * scale_factor);
                cursor_pos.y += scroll_y as f64;
                state.cursor_pos = cursor_pos;
                // macOS seems to generate a spurious move after selecting word?
                if state.pointer_down && prev_pos != state.cursor_pos && !state.editor.is_composing() {
                    state.cursor_reset();
                    let cursor_pos = state.cursor_pos;
                    state.driver(_text_context).extend_selection_to_point(cursor_pos.x as f32, cursor_pos.y as f32);
                }
            }
            CraftMessage::ImeEvent(Ime::Disabled) => {
                state.driver(_text_context).clear_compose();
                state.cache.clear();
            }
            CraftMessage::ImeEvent(Ime::Commit(text)) => {
                state.driver(_text_context).insert_or_replace_selection(text);
                state.cache.clear();
                generate_text_changed_event(&mut state.editor);
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
        event.ime_action(ImeAction::Set(Rectangle::new(
            ime.x0 as f32,
            ime.y0 as f32,
            ime.width() as f32,
            ime.height() as f32,
        )));
    }

    fn resolve_clip(&mut self, clip_bounds: Option<Rectangle>) {
        resolve_clip_for_scrollable(self, clip_bounds);
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
            new_style: TextStyle::from(self.style()),
            last_click_time: None,
            click_count: 0,
            pointer_down: false,
            cursor_pos: Point::default(),
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

    #[cfg(feature = "accesskit")]
    fn compute_accessibility_tree(
        &mut self,
        tree: &mut accesskit::TreeUpdate,
        parent_index: Option<usize>,
        element_state: &mut ElementStateStore,
        scale_factor: f64,
    ) {
        let state: &mut TextInputState = element_state
            .storage
            .get_mut(&self.element_data.component_id)
            .unwrap()
            .data
            .as_mut()
            .downcast_mut()
            .unwrap();

        if state.editor.try_layout().is_none() {
            return;
        }

        let editor = &mut state.editor;

        let current_node_id = accesskit::NodeId(self.element_data().component_id);

        let mut current_node = accesskit::Node::new(accesskit::Role::TextInput);
        let padding_box = self.element_data().layout_item.computed_box_transformed.padding_rectangle().scale(scale_factor);

        current_node.set_bounds(accesskit::Rect {
            x0: padding_box.left() as f64,
            y0: padding_box.top() as f64,
            x1: padding_box.right() as f64,
            y1: padding_box.bottom() as f64,
        });

        editor.try_accessibility(
            tree,
            &mut current_node,
            || accesskit::NodeId(create_unique_element_id()),
            padding_box.x as f64,
            padding_box.y as f64,
        );

        if let Some(parent_index) = parent_index {
            let parent_node = tree.nodes.get_mut(parent_index).unwrap();
            parent_node.1.push_child(current_node_id);
        }

        tree.nodes.push((current_node_id, current_node));
    }

    fn update_state(&mut self, element_state: &mut ElementStateStore, _reload_fonts: bool, scaling_factor: f64) {
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

        if TextStyle::from(self.style()) != state.new_style {
            state.new_style = TextStyle::from(self.style());
            state.cache.clear();
            state.new_text = Some(state.editor.text().to_string());
            let styles = state.editor.edit_styles();
            styles.insert(StyleProperty::FontSize(state.new_style.font_size));
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


        let scale_factor = {
            self.editor.try_layout().unwrap().scale() as f64
        };

        let width_constraint = known_dimensions.width.or(match available_space.width {
            AvailableSpace::MinContent => Some(self.editor.try_layout().unwrap().calculate_content_widths().min),
            AvailableSpace::MaxContent => Some(self.editor.try_layout().unwrap().calculate_content_widths().max),
            AvailableSpace::Definite(width) => {
                let scaled_width = dpi::PhysicalUnit::from_logical::<f32, f32>(width, scale_factor).0;
                Some(scaled_width)
            },
        });
        // Some(self.text_style.font_size * self.text_style.line_height)
        let _height_constraint = known_dimensions.height.or(match available_space.height {
            AvailableSpace::MinContent => None,
            AvailableSpace::MaxContent => None,
            AvailableSpace::Definite(height) => {
                let scaled_height = dpi::PhysicalUnit::from_logical::<f32, f32>(height, scale_factor).0;
                Some(scaled_height)
            },
        });

        self.editor.set_width(width_constraint);
        self.editor.refresh_layout(&mut text_context.font_context, &mut text_context.layout_context);

        let layout = self.editor.try_layout().unwrap();
        let width = layout.width();
        let height = layout.height();

        self.text_render = Some(text_render_data::from_editor(layout));

        let sw = dpi::LogicalUnit::from_physical::<f32, f32>(width, scale_factor).0;
        let sh = dpi::LogicalUnit::from_physical::<f32, f32>(height, scale_factor).0;

        let size = taffy::Size {
            width: sw,
            height: sh,
        };

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
                    ((phase.as_nanos() / self.blink_period.as_nanos() + 1) * self.blink_period.as_nanos()) as u64,
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
