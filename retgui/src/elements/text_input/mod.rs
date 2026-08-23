mod text_input_state;

use retgui_primitives::Color;
use retgui_primitives::geometry::{Rectangle, TrblRectangle};
use retgui_renderer::text_renderer_data::{TextData, TextScroll};
use std::any::Any;
use std::cell::{Ref, RefCell, RefMut};
use std::ops::Deref;
use std::rc::{Rc, Weak};
use std::sync::Arc;
use std::time::Duration;

use parley::BoundingBox;

use ui_events::keyboard::{Key, NamedKey};
use ui_events::pointer::PointerButton;

use retgui_renderer::renderer::Renderer;

use retgui_resource_manager::ResourceManager;

use winit::event::Ime;

use crate::app::{ELEMENTS, WINDOW_MANAGER, request_apply_layout};
use crate::elements::element_data::ElementData;
use crate::elements::text_input::text_input_state::TextInputState;
use crate::elements::traits::clone_element;
use crate::elements::{AsElement, Element, ElementInternals, scrollable};
use crate::events::{Event, EventKind};
use crate::layout::GummyTree;
use crate::layout::layout_context::{GummyTextInputContext, LayoutContext, TextHashKey};
use crate::style::{Animation, Display, Repeat, Style, TimingFunction, Unit};
use crate::text::RangedStyles;
use crate::text::text_context::TextContext;
use crate::text::text_render_data::TextRender;

#[derive(Clone)]
pub struct TextInput {
    pub inner: Rc<RefCell<TextInputInner>>,
}

// A stateful element that shows text.
#[derive(Clone)]
pub struct TextInputInner {
    pub(crate) element_data: ElementData,
    /// Whether the text input will update the editor every update with the user provided text.
    /// NOTE: The editor will always use the user provided text on initialization.
    pub(crate) use_text_value_on_update: bool,
    pub text: Option<String>,
    pub ranged_styles: Option<RangedStyles>,
    pub disabled: bool,
    pub(crate) state: TextInputState,
    pub(crate) me: Weak<RefCell<Self>>,
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
    pub fn new(text: &str) -> Self {
        Self {
            inner: TextInputInner::new(text),
        }
    }

    /// Whether the text input will update the editor every update with the user provided text.
    /// NOTE: The editor will always use the user provided text on initialization.
    pub fn use_text_value_on_update(self, use_initial_text_value: bool) -> Self {
        self.inner.borrow_mut().use_text_value_on_update(use_initial_text_value);
        self
    }

    pub fn disable(self) -> Self {
        self.inner.borrow_mut().disable();
        self
    }

    pub fn get_disabled(&self) -> bool {
        self.inner.borrow().disabled
    }

    pub fn get_text(&self) -> String {
        self.inner.borrow().state.editor().raw_text().to_owned()
    }

    /// Set the text.
    ///
    /// Updates the text content immediately. Mark layout and render caches as dirty. Layout and
    /// render caches will be computed in the next layout/render pass.
    pub fn set_text(self, text: &str) -> Self {
        self.inner.borrow_mut().set_text(text);
        self
    }

    pub fn ranged_styles(self, ranged_styles: RangedStyles) -> Self {
        self.inner.borrow_mut().set_ranged_styles(ranged_styles);
        self
    }
}

impl Element for TextInput {}

impl Drop for TextInputInner {
    fn drop(&mut self) {
        ElementInternals::drop(self)
    }
}

impl AsElement for TextInput {
    fn as_element_rc(&self) -> Rc<RefCell<dyn ElementInternals>> {
        self.inner.clone()
    }

    fn borrow(&self) -> Ref<'_, dyn ElementInternals> {
        self.inner.borrow()
    }

    fn borrow_mut(&self) -> RefMut<'_, dyn ElementInternals> {
        self.inner.borrow_mut()
    }
}

impl crate::elements::ElementData for TextInputInner {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementInternals for TextInputInner {
    fn deep_clone(&self) -> Rc<RefCell<dyn ElementInternals>> {
        clone_element::<Self, _>(self, |element, gummy_tree| {
            let me = Rc::downgrade(element);
            let mut element = element.borrow_mut();
            element.me = me;
            let gummy_id = element.element_data.layout.gummy_node_id();
            element.state.gummy_id = Some(gummy_id);
            element.state.editor.gummy_id = Some(gummy_id);
            let context = LayoutContext::TextInput(GummyTextInputContext {
                element: element.me.clone(),
            });
            gummy_tree.set_node_context(gummy_id, Some(context));
            Some(gummy_id)
        })
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

        self.element_data.layout.has_new_layout = has_new_layout;

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
        &mut self,
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
            _renderer.draw_text(
                self.me.clone(),
                content_rectangle.scale(_scale_factor),
                text_scroll,
                self.is_focused() && self.state.cursor_visible(),
            );
        }

        _renderer.pop_layer();

        self.draw_scrollbar(_renderer, _scale_factor);

        self.maybe_end_overlay(_renderer);
    }

    fn on_event(&mut self, message: &EventKind, text_context: &mut TextContext, event: &mut Event) {
        self.state.is_active = true;

        let editor_owns_scroll_key = matches!(
            message,
            EventKind::KeyboardInputEvent(keyboard_event)
                if matches!(
                    &keyboard_event.key,
                    Key::Named(
                        NamedKey::ArrowUp | NamedKey::ArrowDown | NamedKey::Home | NamedKey::End
                    )
                )
        );
        if !editor_owns_scroll_key {
            scrollable::handle_scroll_logic(self, message, event);
        }

        if event.prevent_defaults {
            return;
        }

        let editor_generation = self.state.editor().generation();
        let scroll_y = self.element_data.scroll().scroll_y() as f64;

        let focused = self.is_focused();

        if let EventKind::ElementMessage(msg) = message
            && let Some(msg) = (msg as &dyn Any).downcast_ref::<TextInputMessage>()
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

        match message {
            EventKind::Focus() => {
                self.start_cursor_blink();
            }
            EventKind::Unfocus() => {
                self.stop_cursor_blink();
            }
            EventKind::KeyboardInputEvent(keyboard_event) if !self.state.editor().is_composing() => {
                if self.disabled || !keyboard_event.state.is_down() || !focused {
                    return;
                }
                self.state
                    .key_press(text_context, keyboard_event, &mut self.element_data);
                event.prevent_propagate();
            }
            EventKind::PointerButtonDown(pointer_button) if pointer_button.button == Some(PointerButton::Primary) => {
                self.focus();
                self.set_pointer_capture(message.pointer_id().unwrap());
                self.state
                    .pointer_down(text_context, pointer_button.state.logical_point(), scroll_y);
            }
            EventKind::PointerButtonUp(pointer_button) if pointer_button.button == Some(PointerButton::Primary) => {
                self.state.pointer_up();
            }
            EventKind::PointerMovedEvent(pointer_moved) => {
                self.state.move_pointer(text_context, pointer_moved, scroll_y);
            }
            EventKind::ImeEvent(Ime::Disabled) => {
                self.state.disable_ime(text_context);
            }
            EventKind::ImeEvent(Ime::Commit(text)) => {
                self.state.insert_or_replace_selection(text_context, text);
            }
            EventKind::ImeEvent(Ime::Preedit(text, cursor)) => {
                self.state.ime_pre_edit(text_context, text, cursor);
            }
            _ => {}
        }

        if self.state.is_layout_dirty {
            self.mark_dirty();
        } else if self.state.editor().generation() != editor_generation {
            request_apply_layout(self.element_data.layout.gummy_node_id());
            self.request_window_redraw();
        }
        {
            let value = self.state.editor().raw_text().to_owned();
            self.element_data.set_accessibility_value(value);
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

    fn set_scale_factor(&mut self, scale_factor: f64) {
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

        if self.is_focused() {
            if !self.state.is_blinking() {
                self.state.reset_blink();
            }
            self.state.cursor_blink();
        } else {
            self.state.disable_blink();
        }
    }
}

impl TextInputInner {
    /// Starts the cursor blink animation.
    fn start_cursor_blink(&mut self) {
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
            WINDOW_MANAGER.with_borrow_mut(|window_manager| {
                window_manager.schedule_element_animations(self.element_data.me.clone());
            });
        }
        self.request_window_redraw();
    }

    /// Stops the cursor blink animation.
    fn stop_cursor_blink(&mut self) {
        self.state.disable_blink();

        let had_animations = !self.element_data.animations.is_empty();
        self.element_data
            .animations
            .retain(|animation| !animation.key_frames.is_empty());
        if had_animations && self.element_data.animations.is_empty() {
            WINDOW_MANAGER.with_borrow_mut(|window_manager| {
                window_manager.cancel_element_animations(&self.element_data.me);
            });
        }
        self.request_window_redraw();
    }

    pub fn new(text: &str) -> Rc<RefCell<Self>> {
        let default_style = TextInputInner::get_default_style();

        let text_input_state = TextInputState::default();

        let inner = Rc::new_cyclic(|me: &Weak<RefCell<TextInputInner>>| {
            RefCell::new(TextInputInner {
                text: Some(text.to_string()),
                element_data: ElementData::new(me.clone(), true),
                use_text_value_on_update: true,
                ranged_styles: Some(RangedStyles::new(vec![])),
                disabled: false,
                state: text_input_state,
                me: me.clone(),
            })
        });
        let mut inner_mut = inner.borrow_mut();
        inner_mut.element_data.style = default_style;

        {
            inner_mut.element_data.set_accessibility_role(issho::Role::TextInput);
            inner_mut.element_data.set_accessibility_enabled(true);
        }
        inner_mut.set_text(text);

        let context = Some(LayoutContext::TextInput(GummyTextInputContext {
            element: inner_mut.me.clone(),
        }));
        inner_mut.element_data.create_layout_node(context);

        let gummy_id = inner_mut.element_data.layout.gummy_node_id;
        inner_mut.state.gummy_id = gummy_id;
        inner_mut.state.editor.gummy_id = gummy_id;

        ELEMENTS.with_borrow_mut(|elements| {
            elements.insert(inner_mut.deref());
        });
        drop(inner_mut);
        inner
    }

    /// Whether the text input will update the editor every update with the user provided text.
    /// NOTE: The editor will always use the user provided text on initialization.
    pub fn use_text_value_on_update(&mut self, use_initial_text_value: bool) {
        self.use_text_value_on_update = use_initial_text_value;
    }

    pub fn disable(&mut self) -> &mut Self {
        self.disabled = true;
        self.element_data.set_accessibility_enabled(false);
        self
    }

    pub fn get_disabled(&mut self) -> bool {
        self.disabled
    }

    pub fn get_text(&self) -> &str {
        self.state.editor().raw_text()
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

impl TextData for TextInputInner {
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
