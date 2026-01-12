mod text_input_state;

use std::any::Any;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::{Rc, Weak};

use craft_primitives::Color;
use craft_primitives::geometry::{Point, Rectangle, TrblRectangle};
use craft_renderer::renderer::{RenderList, TextScroll};
use craft_renderer::text_renderer_data::TextData;
use kurbo::Affine;
use parley::BoundingBox;
use ui_events::pointer::PointerButton;
use winit::event::Ime;

use crate::app::ELEMENTS;
use crate::elements::core::{ElementInternals, resolve_clip_for_scrollable};
use crate::elements::element::{AsElement, ElementImpl};
use crate::elements::element_data::ElementData;
#[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
use crate::elements::element_id::create_unique_element_id;
use crate::elements::text_input::text_input_state::TextInputState;
use crate::elements::{Element, scrollable};
use crate::events::{CraftMessage, Event};
use crate::layout::TaffyTree;
use crate::layout::layout_context::{LayoutContext, TaffyTextInputContext};
use crate::style::{Display, Style, Unit};
use crate::text::RangedStyles;
use crate::text::text_context::TextContext;
use crate::text::text_render_data::TextRender;
use crate::utils::cloneable_any::CloneableAny;

#[derive(Clone)]
pub struct TextInput {
    pub inner: Rc<RefCell<TextInputInner>>,
}

// A stateful element that shows text.
#[derive(Clone)]
pub struct TextInputInner {
    element_data: ElementData,
    /// Whether the text input will update the editor every update with the user provided text.
    /// NOTE: The editor will always use the user provided text on initialization.
    use_text_value_on_update: bool,
    pub text: Option<String>,
    pub ranged_styles: Option<RangedStyles>,
    pub disabled: bool,
    pub(crate) state: TextInputState,
    me: Weak<RefCell<Self>>,
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
        *inner.borrow_mut().element_data.style = *default_style;

        inner.borrow_mut().set_text(text);

        let context = Some(LayoutContext::TextInput(TaffyTextInputContext {
            element: inner.borrow().me.clone(),
        }));
        inner.borrow_mut().element_data.create_layout_node(context);

        let taffy_id = inner.borrow().element_data.layout_item.taffy_node_id;
        inner.borrow_mut().state.taffy_id = taffy_id;
        inner.borrow_mut().state.editor.taffy_id = taffy_id;

        ELEMENTS.with_borrow_mut(|elements| {
            elements.insert(inner.borrow().deref());
        });

        Self { inner }
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

impl AsElement for TextInput {
    fn as_element_rc(&self) -> Rc<RefCell<dyn ElementImpl>> {
        self.inner.clone()
    }
}

impl crate::elements::core::ElementData for TextInputInner {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementInternals for TextInputInner {
    fn apply_layout(
        &mut self,
        taffy_tree: &mut TaffyTree,
        position: Point,
        z_index: &mut u32,
        transform: Affine,
        _pointer: Option<Point>,
        text_context: &mut TextContext,
        clip_bounds: Option<Rectangle>,
        scale_factor: f64,
    ) {
        let node = self.element_data.layout_item.taffy_node_id.unwrap();
        let has_new_layout = taffy_tree.get_has_new_layout(node);

        let dirty = has_new_layout
            || transform != self.element_data.layout_item.get_transform()
            || position != self.element_data.layout_item.position;
        self.element_data.layout_item.has_new_layout = has_new_layout;

        if dirty {
            let result = taffy_tree.layout(node);
            self.resolve_box(position, transform, result, z_index);
            self.apply_clip(clip_bounds);
            self.apply_borders(scale_factor);

            self.element_data.apply_scroll(result);
            self.element_data.scroll_state.as_mut().unwrap().mark_old();

            let text_position = self.computed_box().content_rectangle();
            self.state.set_origin(&text_position.position());

            self.state.is_layout_dirty = false;
        }

        // For manual scroll updates.
        if !dirty
            && self
                .element_data
                .scroll_state
                .map(|scroll_state| scroll_state.is_new())
                .unwrap_or_default()
        {
            let result = taffy_tree.layout(node);
            self.element_data.apply_scroll(result);
            self.element_data.scroll_state.as_mut().unwrap().mark_old();
        }

        if has_new_layout {
            taffy_tree.mark_seen(node);
        }

        self.state.layout(
            self.state.last_requested_key.unwrap().known_dimensions(),
            self.state.last_requested_key.unwrap().available_space(),
            text_context,
            true,
        );

        self.state
            .render_text(self.is_focused(), self.element_data.current_style());
    }

    fn draw(
        &mut self,
        renderer: &mut RenderList,
        _text_context: &mut TextContext,
        _pointer: Option<Point>,
        scale_factor: f64,
    ) {
        if !self.is_visible() {
            return;
        }

        self.add_hit_testable(renderer, true, scale_factor);

        let computed_box_transformed = self.computed_box();
        let content_rectangle = computed_box_transformed.content_rectangle();

        self.draw_borders(renderer, scale_factor);

        let is_scrollable = self.element_data.is_scrollable();

        let element_data = &self.element_data;
        let padding_rectangle = element_data.layout_item.computed_box_transformed.padding_rectangle();
        renderer.push_layer(padding_rectangle.scale(scale_factor));

        let text_scroll = if is_scrollable {
            Some(TextScroll::new(
                self.element_data.scroll().map_or(0.0, |s| s.scroll_y()),
                self.element_data.layout_item.computed_scroll_track.height,
            ))
        } else {
            None
        };

        if self.state.text_render.as_ref().is_some() {
            renderer.draw_text(
                self.me.clone(),
                content_rectangle.scale(scale_factor),
                text_scroll,
                self.is_focused(),
            );
        }

        renderer.pop_layer();

        self.draw_scrollbar(renderer, scale_factor);
    }

    #[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
    fn compute_accessibility_tree(
        &mut self,
        tree: &mut accesskit::TreeUpdate,
        parent_index: Option<usize>,
        scale_factor: f64,
    ) {
        let state: &mut TextInputState = &mut self.state;

        if state.editor().try_layout().is_none() {
            return;
        }

        let current_node_id = accesskit::NodeId(self.element_data.internal_id);

        let mut current_node = accesskit::Node::new(accesskit::Role::TextInput);
        let padding_box = self
            .element_data
            .layout_item
            .computed_box_transformed
            .padding_rectangle()
            .scale(scale_factor);

        current_node.set_bounds(accesskit::Rect {
            x0: padding_box.left() as f64,
            y0: padding_box.top() as f64,
            x1: padding_box.right() as f64,
            y1: padding_box.bottom() as f64,
        });

        self.state.try_accessibility(
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

    fn on_event(
        &mut self,
        message: &CraftMessage,
        text_context: &mut TextContext,
        event: &mut Event,
        _target: Option<Rc<RefCell<dyn ElementInternals>>>,
    ) {
        self.state.is_active = true;

        scrollable::on_scroll_events(self, message, event);

        if !event.propagate {
            return;
        }

        let scroll_y = self.element_data.scroll().map_or(0.0, |s| s.scroll_y() as f64);

        let focused = self.is_focused();

        if let CraftMessage::ElementMessage(msg) = message
            && let Some(msg) = msg.as_any().downcast_ref::<TextInputMessage>()
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
                    //generate_text_changed_event(&mut self.state.editor);
                }
            }
        }

        match message {
            CraftMessage::KeyboardInputEvent(keyboard_event) if !self.state.editor().is_composing() => {
                if self.disabled || !keyboard_event.state.is_down() || !focused {
                    return;
                }
                self.state.key_press(text_context, keyboard_event);
            }
            CraftMessage::PointerButtonDown(pointer_button) => {
                if pointer_button.button == Some(PointerButton::Primary) {
                    self.focus();
                    self.state.pointer_down(text_context);
                }
            }
            CraftMessage::PointerButtonUp(pointer_button) => {
                if pointer_button.button == Some(PointerButton::Primary) {
                    self.state.pointer_up();
                }
            }
            CraftMessage::PointerMovedEvent(pointer_moved) => {
                self.state.move_pointer(text_context, pointer_moved, scroll_y);
            }
            CraftMessage::ImeEvent(Ime::Disabled) => {
                self.state.disable_ime(text_context);
            }
            CraftMessage::ImeEvent(Ime::Commit(text)) => {
                self.state.insert_or_replace_selection(text_context, text);
                //generate_text_changed_event(&mut self.state.editor);
            }
            CraftMessage::ImeEvent(Ime::Preedit(text, cursor)) => {
                self.state.ime_pre_edit(text_context, text, cursor);
            }
            _ => {}
        }

        if self.state.is_layout_dirty {
            self.mark_dirty();
        }
    }

    fn apply_clip(&mut self, clip_bounds: Option<Rectangle>) {
        resolve_clip_for_scrollable(self, clip_bounds);
    }

    fn get_default_style() -> Box<Style>
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
        self.apply_borders(scale_factor);
        self.state.set_scale_factor(scale_factor);
        self.mark_dirty();
    }
}

impl TextInputInner {
    /// Whether the text input will update the editor every update with the user provided text.
    /// NOTE: The editor will always use the user provided text on initialization.
    pub fn use_text_value_on_update(&mut self, use_initial_text_value: bool) {
        self.use_text_value_on_update = use_initial_text_value;
    }

    pub fn disable(&mut self) -> &mut Self {
        self.disabled = true;
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

impl ElementImpl for TextInputInner {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
