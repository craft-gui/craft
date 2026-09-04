mod text_input_state;

use retgui_primitives::Color;
use retgui_primitives::geometry::{Rectangle, TrblRectangle};
use retgui_renderer::text_renderer_data::{TextData, TextScroll};
use std::sync::Arc;
use std::time::Duration;

use parley::BoundingBox;

use winit::keyboard::{Key, NamedKey};

use retgui_renderer::renderer::Renderer;

use retgui_resource_manager::ResourceManager;

use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event::{Ime, MouseButton as PointerButton};
use winit::window::{ImeCapabilities, ImeEnableRequest, ImeHint, ImePurpose, ImeRequest, ImeRequestData};

use crate::elements::element_data::ElementData;
use crate::elements::text_input::text_input_state::TextInputState;
use crate::elements::traits::clone_element;
use crate::elements::{DynElement, Element, ElementNode, Elements, WindowNode, scrollable};
use crate::events::{Event, EventKind};
use crate::layout::GummyTree;
use crate::layout::layout_context::{GummyTextInputContext, LayoutContext, TextHashKey};
use crate::style::{Animation, Display, Repeat, Style, TimingFunction, Unit};
use crate::text::RangedStyles;
use crate::text::text_context::TextContext;
use crate::text::text_render_data::TextRender;

/// Editable text area.
#[derive(Clone, Copy)]
pub struct TextInput {
    pub(crate) inner: DynElement,
}

// A stateful element that shows text.
#[derive(Clone)]
pub(crate) struct TextInputNode {
    pub(crate) element_data: ElementData,
    pub(crate) ranged_styles: Option<RangedStyles>,
    pub(crate) disabled: bool,
    pub(crate) state: TextInputState,
}

#[allow(dead_code)]
/// An external message that allows others to command the TextInput.
pub enum TextInputMessage {
    Copy,
    Paste,
    Cut,
    // TODO: Add more messages.
}

impl TextInput {
    /// Creates a single line editable text input.
    pub fn new(elements: &mut Elements, text: &str) -> Self {
        Self {
            inner: TextInputNode::new(elements, text),
        }
    }

    /// Disables the text input.
    pub fn set_disabled(&self, elements: &mut Elements, disabled: bool) {
        if let Some(input) = elements.try_get_as_mut::<TextInputNode>(self.inner) {
            input.disabled(disabled);
        }
    }

    /// Returns whether the element is disabled.
    pub fn is_disabled(&self, elements: &Elements) -> bool {
        elements
            .try_get_as::<TextInputNode>(self.inner)
            .is_some_and(|input| input.disabled)
    }

    /// Returns whether the text input is multiline.
    pub fn is_multiline(&self, elements: &Elements) -> bool {
        elements
            .try_get_as::<TextInputNode>(self.inner)
            .is_some_and(|input| input.state.multiline)
    }

    pub fn set_multiline(&self, elements: &mut Elements, multiline: bool) {
        if let Some(input) = elements.try_get_as_mut::<TextInputNode>(self.inner) {
            input.multiline(multiline);
        }
    }

    /// Returns the text in the text input.
    ///
    /// This does not include the ime preedit text.
    pub fn text(&self, elements: &Elements) -> String {
        elements
            .try_get_as::<TextInputNode>(self.inner)
            .map_or_else(String::new, |input| input.state.editor().text().chars().collect())
    }

    /// Set the text.
    ///
    /// Updates the text content immediately. Mark layout and render caches as dirty. Layout and
    /// render caches will be computed in the next layout/render pass.
    pub fn set_text(&self, elements: &mut Elements, text: &str) {
        if let Some(input) = elements.try_get_as_mut::<TextInputNode>(self.inner) {
            input.set_text(text);
        }
    }

    /// Styles the text along ranges.
    pub fn set_ranged_styles(&self, elements: &mut Elements, ranged_styles: RangedStyles) {
        if let Some(input) = elements.try_get_as_mut::<TextInputNode>(self.inner) {
            input.set_ranged_styles(ranged_styles);
        }
    }

    /// Returns the ranged styles.
    pub fn ranged_styles(&self, elements: &Elements) -> Option<RangedStyles> {
        elements
            .try_get_as::<TextInputNode>(self.inner)
            .and_then(|input| input.ranged_styles.clone())
    }
}

impl Element for TextInput {
    fn as_dyn_element(&self) -> DynElement {
        self.inner
    }
}

impl crate::elements::ElementNodeData for TextInputNode {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementNode for TextInputNode {
    fn deep_clone(&self, elements: &mut Elements) -> DynElement {
        DynElement::new(clone_element::<Self, _>(self, elements, |element, gummy_tree| {
            let gummy_id = element.element_data.layout.gummy_node_id();
            element.state.gummy_id = Some(gummy_id);
            element.state.editor.gummy_id = Some(gummy_id);
            let context = LayoutContext::TextInput(GummyTextInputContext {
                element: element.element_data.me,
            });
            gummy_tree.set_node_context(gummy_id, Some(context));
            Some(gummy_id)
        }))
    }

    fn apply_layout(
        &mut self,
        gummy_tree: &mut GummyTree,
        z_index: &mut u32,
        text_context: &mut TextContext,
        scale_factor: f64,
    ) {
        let node = self.element_data.layout.gummy_node_id.unwrap();
        let has_new_layout = gummy_tree.has_new_layout(node);

        self.element_data.layout.has_new_layout.set(has_new_layout);

        let result = gummy_tree.get_layout(node);
        let render_available_space = gummy::Size {
            width: gummy::AvailableSpace::Definite((result.content_box_width() - result.scrollbar_size.width).max(0.0)),
            height: gummy::AvailableSpace::Definite(
                (result.content_box_height() - result.scrollbar_size.height).max(0.0),
            ),
        };
        let render_key = TextHashKey::new(gummy::Size::NONE, render_available_space);
        let scroll_to_cursor_after_layout = self.is_focused() && !self.state.is_rendered_for(render_key);
        let needs_text_layout = self.state.needs_final_layout(render_key);
        if has_new_layout {
            self.resolve_box(result, z_index);
            self.apply_borders(scale_factor);
            self.element_data.apply_scroll(result);
            self.element_data.layout.scroll_state.mark_old();
        }

        // For manual scroll updates.
        if !has_new_layout && self.element_data.layout.scroll_state.is_new() {
            self.element_data.apply_scroll(result);
            self.element_data.layout.scroll_state.mark_old();
        }

        if has_new_layout {
            gummy_tree.mark_seen(node);
        }

        if needs_text_layout {
            self.state.finalize_layout(render_available_space, text_context);

            if scroll_to_cursor_after_layout {
                self.state.maybe_scroll_to_cursor(&mut self.element_data);
            }
        }

        self.state.render_text(self.element_data.style());
    }

    fn draw(
        &self,
        _elements: &Elements,
        _renderer: &mut dyn Renderer,
        _resource_manager: Arc<ResourceManager>,
        _scale_factor: f64,
        _text_context: &mut TextContext,
    ) {
        if !self.is_visible() {
            return;
        }

        let text_position = self.get_computed_box_transformed().content_rectangle();
        self.state.set_origin(&text_position.position());

        self.maybe_start_overlay(_renderer);

        self.add_hit_testable(_renderer, true, _scale_factor);

        let content_rectangle = self.element_data.layout.local_box().content_rectangle();

        self.draw_borders(_renderer, _scale_factor);

        let is_scrollable = self.element_data.is_scrollable();

        let element_data = &self.element_data;
        let padding_rectangle = element_data.layout.local_box().padding_rectangle();
        _renderer.push_layer(padding_rectangle.scale(_scale_factor));

        let text_scroll = if is_scrollable {
            Some(physical_text_scroll(
                self.element_data.scroll().scroll_y(),
                self.element_data.layout.computed_scroll_track.height,
                _scale_factor,
            ))
        } else {
            None
        };

        if self.state.text_render.as_ref().is_some() {
            let snapshot = self.state.text_snapshot.as_ref().expect("text snapshot not found");
            _renderer.draw_text_ref(
                snapshot,
                content_rectangle.scale(_scale_factor),
                text_scroll,
                self.is_focused() && self.state.editor().raw_selection().is_collapsed() && self.state.cursor_visible(),
            );
        }

        _renderer.pop_layer();

        self.draw_scrollbar(_renderer, _scale_factor);

        self.maybe_end_overlay(_renderer);
    }

    fn on_event(&mut self, elements: &mut Elements, event: &mut EventKind, text_context: &mut TextContext) {
        self.state.is_active = true;
        let focused = self.is_focused();
        let ime_owns_keyboard_event = focused
            && matches!(
                &*event,
                EventKind::KeyDown(keyboard_event) | EventKind::KeyUp(keyboard_event)
                    if keyboard_event.is_composing || self.state.editor().is_composing()
            );

        let editor_owns_scroll_key = ime_owns_keyboard_event
            || matches!(
                &*event,
                EventKind::KeyDown(keyboard_event) | EventKind::KeyUp(keyboard_event)
                    if matches!(
                        &keyboard_event.key,
                        Key::Named(
                            NamedKey::ArrowUp | NamedKey::ArrowDown | NamedKey::Home | NamedKey::End
                        )
                    )
            );
        if !editor_owns_scroll_key {
            scrollable::handle_scroll_logic(elements, self, event);
        }

        if event.is_default_prevented() {
            return;
        }

        let editor_generation = self.state.editor().generation();
        let scroll_y = self.element_data.scroll().scroll_y() as f64;

        if let EventKind::Custom(custom_event) = &*event
            && let Some(msg) = custom_event.data::<TextInputMessage>()
        {
            match msg {
                TextInputMessage::Copy => {
                    self.state.copy(text_context);
                }
                TextInputMessage::Paste => {
                    if self.disabled {
                        return;
                    }
                    self.state.paste(text_context);
                    self.mark_dirty();
                    //generate_text_changed_event(&mut self.state.editor);
                }
                TextInputMessage::Cut => {
                    if self.disabled {
                        return;
                    }
                    self.state.cut(text_context);
                    self.mark_dirty();
                }
            }
        }

        match event {
            EventKind::Focus(_) => {
                self.start_cursor_blink(elements);
                if !self.disabled {
                    self.set_ime_enabled(elements, true);
                }
            }
            EventKind::Unfocus(_) => {
                self.stop_cursor_blink(elements);
                self.state.disable_ime(text_context);
                self.set_ime_enabled(elements, false);
            }
            EventKind::KeyDown(keyboard_event) | EventKind::KeyUp(keyboard_event) if ime_owns_keyboard_event => {
                // Candidate-list navigation belongs to the IME. Keep those keys
                // from reaching scrollable ancestors while composition is active.
                keyboard_event.stop_propagation();
            }
            EventKind::KeyDown(keyboard_event) | EventKind::KeyUp(keyboard_event)
                if !self.state.editor().is_composing() =>
            {
                if self.disabled || !keyboard_event.state.is_pressed() || !focused {
                    return;
                }
                self.state
                    .key_press(elements, text_context, keyboard_event, &mut self.element_data);
                keyboard_event.stop_propagation();
            }
            EventKind::PointerDown(pointer_button) if pointer_button.button == Some(PointerButton::Left) => {
                self.focus(elements);
                self.set_pointer_capture(elements, pointer_button.pointer.pointer_id.unwrap());
                self.state.pointer_down(text_context, pointer_button.position, scroll_y);
            }
            EventKind::PointerUp(pointer_button) if pointer_button.button == Some(PointerButton::Left) => {
                self.state.pointer_up();
            }
            EventKind::PointerMoved(pointer_moved) => {
                self.state
                    .move_pointer(text_context, pointer_moved.current.logical_point(), scroll_y);
            }
            EventKind::Ime(ime_event) => match &ime_event.ime {
                Ime::Disabled => {
                    self.state.ime_state.is_ime_active = false;
                    self.state.disable_ime(text_context);
                }
                Ime::Commit(text) => {
                    self.state.insert_or_replace_selection(text_context, text);
                    self.state.generate_text_changed_event(elements, &self.element_data);
                }
                Ime::Preedit(text, cursor) => self.state.ime_pre_edit(text_context, text, cursor),
                Ime::DeleteSurrounding {
                    before_bytes,
                    after_bytes,
                } => {
                    if self
                        .state
                        .ime_delete_surrounding(text_context, *before_bytes, *after_bytes)
                    {
                        self.state.generate_text_changed_event(elements, &self.element_data);
                    }
                }
                Ime::Enabled => self.state.ime_state.is_ime_active = true,
                _ => {}
            },
            _ => {}
        }

        if self.state.is_layout_dirty {
            self.mark_dirty();
        } else if self.state.editor().generation() != editor_generation {
            self.element_data.apply_layout_dirty = true;
            self.request_window_redraw();
        }
        {
            let value = self.state.editor().raw_text().to_owned();
            self.element_data.set_accessibility_value(value);
        }
        if self.state.ime_state.is_ime_active {
            self.update_ime(elements);
        }
    }

    fn get_default_style() -> Style
    where
        Self: Sized,
    {
        let mut style = Style::new();

        style.set_display(Display::Block);

        const BORDER_COLOR: Color = Color::from_rgb8(199, 199, 206);
        style.set_border_color(TrblRectangle::new_all(BORDER_COLOR));
        style.set_border_width(TrblRectangle::new_all(Unit::Px(1.0)));
        style.set_border_radius([(5.0, 5.0); 4]);

        let padding = Unit::Px(4.0);
        style.set_padding(TrblRectangle::new_all(padding));

        style
    }

    fn set_scale_factor(&mut self, _elements: &mut Elements, scale_factor: f64) {
        self.element_data.applied_scale_factor = scale_factor;
        self.apply_borders(scale_factor);
        self.state.set_scale_factor(scale_factor);
        self.mark_dirty();
    }

    fn on_text_style_changed(&mut self) {
        let style = self.element_data.style.clone();
        self.state.set_style(&style);
    }

    fn animation_tick(&mut self, delta: Duration) {
        let mut animations = std::mem::take(&mut self.element_data.animations);
        for animation in &mut animations {
            animation.tick(delta);
            if animation.key_frames.len() >= 2 {
                animation.apply_styles(&mut |style| self.set_style_variant(style));
            }
        }
        self.element_data.animations = animations;

        if self.is_focused() && self.state.editor().raw_selection().is_collapsed() {
            if !self.state.is_blinking() {
                self.state.reset_blink();
            }
            self.state.cursor_blink();
        } else {
            self.state.disable_blink();
        }
    }
}

impl TextInputNode {
    fn set_ime_enabled(&mut self, elements: &Elements, enabled: bool) {
        self.state.ime_state.is_ime_active = enabled;

        let Some(window) = self
            .element_data
            .window
            .and_then(|window| elements.get_as::<WindowNode>(window).winit_window())
        else {
            return;
        };

        let request = if enabled {
            let capabilities = ImeCapabilities::new()
                .with_hint_and_purpose()
                .with_cursor_area()
                .with_surrounding_text();
            let cursor_area = self.ime_cursor_area();
            let request_data = self.ime_request_data(cursor_area);
            let enable_request = ImeEnableRequest::new(capabilities, request_data)
                .expect("IME capabilities and initial state must match");
            ImeRequest::Enable(enable_request)
        } else {
            ImeRequest::Disable
        };

        let _ = window.request_ime_update(request);
    }

    fn update_ime(&mut self, elements: &Elements) {
        let Some(window) = self
            .element_data
            .window
            .and_then(|window| elements.get_as::<WindowNode>(window).winit_window())
        else {
            return;
        };
        let cursor_area = self.ime_cursor_area();
        let request_data = self.ime_request_data(cursor_area);
        let _ = window.request_ime_update(ImeRequest::Update(request_data));
    }

    fn ime_cursor_area(&self) -> Rectangle {
        let fallback = self.get_computed_box_transformed().content_rectangle();
        self.state
            .ime_cursor_area(fallback, self.element_data.scroll().scroll_y())
    }

    fn ime_request_data(&self, cursor_area: Rectangle) -> ImeRequestData {
        ImeRequestData::default()
            .with_hint_and_purpose(ImeHint::NONE, ImePurpose::Normal)
            .with_cursor_area(
                LogicalPosition::new(cursor_area.x, cursor_area.y).into(),
                LogicalSize::new(cursor_area.width, cursor_area.height).into(),
            )
            .with_surrounding_text(
                self.state
                    .ime_surrounding_text()
                    .expect("text input must produce valid IME surrounding text"),
            )
    }

    /// Starts the cursor blink animation.
    fn start_cursor_blink(&mut self, elements: &mut Elements) {
        self.state.reset_blink();

        if self
            .element_data
            .animations
            .iter()
            .any(|animation| animation.key_frames.is_empty())
        {
            return;
        }

        let should_schedule = self.element_data.animations.is_empty();
        self.element_data.animations.push(Animation::new(
            Duration::from_millis(500),
            Repeat::Forever,
            TimingFunction::Linear,
        ));
        if should_schedule {
            elements.with_window_manager(|window_manager, _| {
                window_manager.schedule_element_animations(self.element_data.me.clone());
            });
        }
        self.request_window_redraw();
    }

    /// Stops the cursor blink animation.
    fn stop_cursor_blink(&mut self, elements: &mut Elements) {
        self.state.disable_blink();

        let had_animations = !self.element_data.animations.is_empty();
        self.element_data
            .animations
            .retain(|animation| !animation.key_frames.is_empty());
        if had_animations && self.element_data.animations.is_empty() {
            elements.with_window_manager(|window_manager, _| {
                window_manager.cancel_element_animations(&self.element_data.me);
            });
        }
        self.request_window_redraw();
    }

    pub fn new(elements: &mut Elements, text: &str) -> DynElement {
        let default_style = TextInputNode::get_default_style();

        let text_input_state = TextInputState::default();

        let inner = elements.insert_with(|me, access_tree| {
            Box::new(TextInputNode {
                element_data: ElementData::new(me, true, access_tree),
                ranged_styles: Some(RangedStyles::new(vec![])),
                disabled: false,
                state: text_input_state,
            })
        });
        let inner_mut = elements.get_as_mut::<TextInputNode>(inner);
        inner_mut.element_data.style = default_style;

        {
            inner_mut.element_data.set_accessibility_role(issho::Role::TextInput);
            inner_mut.element_data.set_accessibility_enabled(true);
        }
        inner_mut.set_text(text);

        let context = Some(LayoutContext::TextInput(GummyTextInputContext {
            element: inner_mut.element_data.me,
        }));
        let _ = inner_mut;
        elements.create_layout_node(inner, context);

        let inner_mut = elements.get_as_mut::<TextInputNode>(inner);
        let gummy_id = inner_mut.element_data.layout.gummy_node_id;
        inner_mut.state.gummy_id = gummy_id;
        inner_mut.state.editor.gummy_id = gummy_id;

        inner
    }

    pub fn disabled(&mut self, disabled: bool) -> &mut Self {
        self.disabled = disabled;
        self.element_data.set_accessibility_enabled(!disabled);
        self
    }

    pub fn multiline(&mut self, multiline: bool) -> &mut Self {
        self.state.multiline = multiline;
        self
    }

    /// Set the text.
    ///
    /// Updates the text content immediately. Mark layout and render caches as dirty. Layout and
    /// render caches will be computed in the next layout/render pass.
    pub fn set_text(&mut self, text: &str) -> &mut Self {
        self.state.set_text(text);
        self.mark_dirty();
        self.element_data.set_accessibility_value(text.to_owned());
        self
    }

    pub fn set_ranged_styles(&mut self, ranged_styles: RangedStyles) -> &mut Self {
        self.state.set_ranged_styles(ranged_styles);
        self.mark_dirty();
        self
    }
}

impl TextData for TextInputNode {
    fn get_text_renderer(&self) -> Option<&TextRender> {
        self.state.text_render.as_ref()
    }
}

fn parley_box_to_rect(bounding_box: BoundingBox) -> Rectangle {
    Rectangle::new(
        bounding_box.x0 as f32,
        bounding_box.y0 as f32,
        bounding_box.width() as f32,
        bounding_box.height() as f32,
    )
}

fn physical_text_scroll(scroll_y: f32, scroll_height: f32, scale_factor: f64) -> TextScroll {
    TextScroll::new(scroll_y * scale_factor as f32, scroll_height * scale_factor as f32)
}
