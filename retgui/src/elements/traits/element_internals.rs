use std::any::Any;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;

use issho::{AccessEvent, IsshoError};

use retgui_primitives::brush::Brush;
use retgui_primitives::geometry::{Affine, ElementBox, Point, TrblRectangle};

use retgui_renderer::renderer::Renderer;

use retgui_resource_manager::ResourceManager;

use crate::elements::scrollable::{ScrollState, draw_scrollbar};
use crate::elements::{DynElement, ElementIds, ElementStates, HasElementData, RetGuiAccessTree, RetainedElements, ScrollOptions, WindowElement};
use crate::events::pointer_capture::PointerCapture;
use crate::events::{CheckboxToggledHandler, ClickHandler, CustomHandler, EventCallback, EventCallbackKind, EventKind, EventListenerOptions, FocusEvent, FocusHandler, KeyboardInputHandler, PointerCaptureHandler, PointerEnterHandler, PointerEventHandler, PointerId, PointerLeaveHandler, PointerUpdateHandler, RadioValueChangedHandler, ScrollHandler, SliderValueChangedHandler, TextInputChangedHandler, UnfocusEvent, UnfocusHandler};
use crate::layout::GummyTree;
use crate::style::{AlignContent, AlignItems, AlignSelf, Animation, BoxShadow, BoxSizing, Display, FlexDirection, FlexWrap, FontFamily, FontStyle, FontWeight, JustifyContent, Overflow, Position, ScrollbarColor, Style, StyleVariant, TextAlign, Underline, Unit};
use crate::text::text_context::TextContext;
use crate::{Color, RetGuiError};

#[cfg(not(target_arch = "wasm32"))]
pub type AnimationInstant = std::time::Instant;

#[cfg(target_arch = "wasm32")]
pub type AnimationInstant = web_time::Instant;

/// When an element needs its next animation update.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AnimationSchedule {
    /// No further animation updates are needed.
    #[default]
    None,
    /// Update again on the next rendered frame.
    NextFrame,
    /// Update at or after the specified instant.
    At(AnimationInstant),
}

impl AnimationSchedule {
    /// Returns the schedule that requests the earlier update.
    pub fn merge(self, other: Self) -> Self {
        match (self, other) {
            (Self::NextFrame, _) | (_, Self::NextFrame) => Self::NextFrame,
            (Self::At(left), Self::At(right)) => Self::At(left.min(right)),
            (Self::None, schedule) | (schedule, Self::None) => schedule,
        }
    }
}

/// Internal element methods that should typically be ignored by users. Public for custom elements.
pub trait ElementInternals: HasElementData + Any {
    fn deep_clone(
        &self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        access_tree: &RetGuiAccessTree,
        by_internal_id: &mut ElementIds,
    ) -> DynElement;

    fn window_pointer_capture(&mut self) -> Option<&mut PointerCapture> {
        None
    }

    fn position_in_parent(&self, elements: &RetainedElements) -> Option<usize> {
        let parent = self.parent();

        // @OPTIMIZE: We are copying the vec here.
        if let Some(parent) = parent {
            let me = self.element_data().me;
            let children = &elements.get(parent).element_data().children;
            let self_position = children.iter().position(|child| *child == me).unwrap();

            Some(self_position)
        } else {
            None
        }
    }

    /// A helper to check if the element is visible.
    fn is_visible(&self) -> bool {
        let style = &self.element_data().style;
        style.get_visible() && style.get_display() != Display::None
    }

    fn is_keyboard_focusable(&self) -> bool {
        let data = self.element_data();
        data.access_key
            .and_then(|key| data.access_tree.get_node(key))
            .is_some_and(|node| node.enabled() && node.role().is_keyboard_focusable())
    }

    /// A helper to draw all children.
    fn draw_children(
        &self,
        elements: &RetainedElements,
        states: &ElementStates,
        renderer: &mut dyn Renderer,
        resource_manager: Arc<ResourceManager>,
        scale_factor: f64,
        text_context: &mut TextContext,
    ) {
        let parent_transform = renderer.get_transform();
        let scroll_y = self.element_data().scroll().scroll_y() as f64 * scale_factor;
        renderer.set_transform(parent_transform * Affine::translate((0.0, -scroll_y)));

        for child in &self.element_data().children {
            elements.get_for_draw(*child).draw_transformed(
                elements,
                states,
                renderer,
                resource_manager.clone(),
                scale_factor,
                text_context,
            );
        }
        renderer.set_transform(parent_transform);
    }

    /// Applies this element's local layout result from the [`GummyTree`].
    /// Ancestor position, scrolling, and clipping are deliberately excluded;
    /// those are composed by [`Self::draw_transformed`] during tree traversal.
    ///
    /// # Parameters
    /// - `gummy_tree`: The layout tree containing the computed results.
    /// - `z_index`: A mutable counter used to assign stacking order as elements
    ///   are processed.
    /// - `text_context`: Context used for text layout and measurement.
    /// - `scale_factor`: Scale used for local render caches such as border paths.
    ///
    /// # Effects
    /// This function mutates only element-local geometry and render caches.
    fn apply_layout(
        &mut self,
        gummy_tree: &mut GummyTree,
        z_index: &mut u32,
        _text_context: &mut TextContext,
        scale_factor: f64,
    ) {
        let node = self.element_data().layout.gummy_node_id.unwrap();
        let layout = gummy_tree.get_layout(node);
        let has_new_layout = gummy_tree.has_new_layout(node);

        self.element_data_mut().layout.has_new_layout.set(has_new_layout);
        if has_new_layout {
            self.resolve_box(layout, z_index);
            self.apply_borders(scale_factor);
            self.element_data_mut().apply_scroll(layout);
            gummy_tree.mark_seen(node);
        } else if self.element_data().layout.scroll_state.is_new() {
            self.element_data_mut().apply_scroll(layout);
            self.element_data_mut().layout.scroll_state.mark_old();
        }
    }

    fn draw_transformed(
        &self,
        elements: &RetainedElements,
        states: &ElementStates,
        renderer: &mut dyn Renderer,
        resource_manager: Arc<ResourceManager>,
        scale_factor: f64,
        text_context: &mut TextContext,
    ) {
        let parent_transform = renderer.get_transform();
        let local_transform = self.element_data().layout.local_transform(scale_factor);
        let render_transform = parent_transform * local_transform;
        renderer.set_transform(render_transform);

        let logical_transform = Affine::scale(1.0 / scale_factor) * render_transform * Affine::scale(scale_factor);
        let physical_clip = if self.style().get_overlay() {
            renderer.render_list().cull
        } else {
            renderer.get_clip()
        };
        let logical_clip = physical_clip.map(|clip| clip.scale(1.0 / scale_factor));
        let scale_changed = self.element_data().access_scale_factor.get() != scale_factor;
        let update_accessibility = {
            let layout = &self.element_data().layout;
            let changed = layout.update_render_state(logical_transform, logical_clip);
            let has_new_layout = layout.has_new_layout.replace(false);
            changed || has_new_layout || scale_changed
        };
        if update_accessibility {
            self.element_data().set_accessibility_bounds_from_layout(scale_factor);
        }

        self.draw(elements, states, renderer, resource_manager, scale_factor, text_context);
        renderer.set_transform(parent_transform);
    }

    /// Draws this element in its own local coordinate space.
    fn draw(
        &self,
        _elements: &RetainedElements,
        _states: &ElementStates,
        _renderer: &mut dyn Renderer,
        _resource_manager: Arc<ResourceManager>,
        _scale_factor: f64,
        _text_context: &mut TextContext,
    ) {
    }

    fn sync_accessibility_children(&mut self, elements: &RetainedElements) {
        let data = self.element_data();
        let Some(node) = data.access_key else {
            return;
        };
        let children = data
            .children
            .iter()
            .filter_map(|child| elements.get(*child).element_data().access_key)
            .collect::<Vec<_>>();
        data.access_tree.set_children(node, &children);
    }

    /// Handles default events.
    fn on_event(
        &mut self,
        _elements: &mut RetainedElements,
        _gummy_tree: &mut GummyTree,
        _access_tree: &RetGuiAccessTree,
        _by_internal_id: &mut ElementIds,
        _event_queue: &mut VecDeque<EventKind>,
        _focus: &mut Option<DynElement>,
        _focus_outline_visible: bool,
        _pending_animation_updates: &mut Vec<(DynElement, bool)>,
        _states: &mut ElementStates,
        _event: &mut EventKind,
        _text_context: &mut TextContext,
    ) {
    }

    /// Computes this element's box model.
    fn resolve_box(&mut self, result: &gummy::Layout, layout_order: &mut u32) {
        self.element_data_mut().layout.resolve_box(result, layout_order);
    }

    fn apply_borders(&mut self, scale_factor: f64) {
        self.element_data_mut().apply_borders(scale_factor);
    }

    fn add_hit_testable(&self, renderer: &mut dyn Renderer, hit_testable: bool, scale_factor: f64) {
        if hit_testable {
            let id = self.element_data().internal_id;
            renderer.push_hit_testable(
                id,
                self.element_data()
                    .layout
                    .local_box()
                    .border_rectangle()
                    .scale(scale_factor),
            );
        }
    }

    fn draw_borders(&self, renderer: &mut dyn Renderer, scale_factor: f64) {
        let current_style = self.element_data().style();

        self.element_data()
            .layout
            .draw_borders(renderer, current_style, scale_factor);
    }

    fn maybe_start_overlay(&self, renderer: &mut dyn Renderer) {
        if self.style().get_overlay() {
            renderer.start_overlay();
        }
    }

    fn maybe_end_overlay(&self, renderer: &mut dyn Renderer) {
        if self.style().get_overlay() {
            renderer.end_overlay();
        }
    }

    fn maybe_start_layer(&self, renderer: &mut dyn Renderer, scale_factor: f64) {
        let element_data = self.element_data();
        let padding_rectangle = element_data.layout.local_box().padding_rectangle().scale(scale_factor);

        if self.should_start_new_layer() {
            renderer.push_layer(padding_rectangle);
        }
    }

    fn maybe_end_layer(&self, renderer: &mut dyn Renderer) {
        if self.should_start_new_layer() {
            renderer.pop_layer();
        }
    }

    fn draw_scrollbar(&self, renderer: &mut dyn Renderer, scale_factor: f64) {
        let element_data = self.element_data();
        draw_scrollbar(&element_data.style, &element_data.layout, renderer, scale_factor);
    }

    fn should_start_new_layer(&self) -> bool {
        let element_data = self.element_data();

        element_data.style().get_overflow()[1] == Overflow::Scroll
    }

    /// Returns the element's [`ElementBox`] without any transforms applied.
    fn computed_box(&self) -> ElementBox {
        self.element_data().layout.computed_box
    }

    /// Gets
    fn get_default_style() -> Style
    where
        Self: Sized,
    {
        Style::new()
    }

    /// Mark layout node dirty.
    fn mark_dirty(&mut self, gummy_tree: &mut GummyTree) {
        if let Some(node) = self.element_data().layout.gummy_node_id {
            gummy_tree.mark_dirty(node);
        }
        self.request_window_redraw();
    }

    /// Updates gummy's style to reflect retgui's style struct.
    fn update_gummy_style(&mut self, gummy_tree: &mut GummyTree) {
        let data = self.element_data();
        if let Some(node) = data.layout.gummy_node_id {
            gummy_tree.set_style(node, data.style.to_gummy_style());
        }
        self.request_window_redraw();
    }

    /// Set's this element's scale factor. This should not be used to scale individual elements.
    fn set_scale_factor(&mut self, elements: &mut RetainedElements, gummy_tree: &mut GummyTree, scale_factor: f64) {
        self.element_data_mut().applied_scale_factor = scale_factor;
        self.apply_borders(scale_factor);
        for child in self.element_data().children.clone() {
            elements.dispatch_mut(child, |child, elements| {
                child.set_scale_factor(elements, gummy_tree, scale_factor)
            });
        }
        self.mark_dirty(gummy_tree);
    }

    fn get_first_child(&self) -> Result<DynElement, RetGuiError> {
        self.element_data()
            .children
            .first()
            .cloned()
            .ok_or(RetGuiError::ElementNotFound)
    }

    fn get_last_child(&self) -> Result<DynElement, RetGuiError> {
        self.element_data()
            .children
            .last()
            .cloned()
            .ok_or(RetGuiError::ElementNotFound)
    }

    fn get_previous_sibling(&self, elements: &RetainedElements) -> Result<DynElement, RetGuiError> {
        let parent = self.parent();
        let position = self.position_in_parent(elements);

        if let Some(position) = position
            && let Some(parent) = parent
        {
            let previous_sibling = (position != 0)
                .then(|| elements.get(parent).element_data().children.get(position - 1).copied())
                .flatten();
            if let Some(previous_sibling) = previous_sibling {
                Ok(previous_sibling)
            } else {
                Err(RetGuiError::ElementNotFound)
            }
        } else {
            Err(RetGuiError::ElementNotFound)
        }
    }

    fn get_next_sibling(&self, elements: &RetainedElements) -> Result<DynElement, RetGuiError> {
        let parent = self.parent();
        let position = self.position_in_parent(elements);

        if let Some(position) = position
            && let Some(parent) = parent
        {
            let next_sibling = elements.get(parent).element_data().children.get(position + 1).copied();
            if let Some(next_sibling) = next_sibling {
                Ok(next_sibling)
            } else {
                Err(RetGuiError::ElementNotFound)
            }
        } else {
            Err(RetGuiError::ElementNotFound)
        }
    }

    fn swap_child(
        &mut self,
        elements: &RetainedElements,
        gummy_tree: &mut GummyTree,
        child_1: DynElement,
        child_2: DynElement,
    ) -> Result<(), RetGuiError> {
        let children = &mut self.element_data_mut().children;
        let position_1 = children
            .iter()
            .position(|x| *x == child_1)
            .ok_or(RetGuiError::ElementNotFound)?;

        let position_2 = children
            .iter()
            .position(|x| *x == child_2)
            .ok_or(RetGuiError::ElementNotFound)?;

        if position_1 == position_2 {
            return Ok(());
        }

        // Swap the children.
        self.element_data_mut().children.swap(position_1, position_2);

        // Swap the children's gummy nodes.
        let parent_id = self.element_data().layout.gummy_node_id;
        let child_1_id = elements.get(child_1).element_data().layout.gummy_node_id;
        let child_2_id = elements.get(child_2).element_data().layout.gummy_node_id;

        if let Some(parent_id) = parent_id
            && let Some(child_1_id) = child_1_id
            && let Some(child_2_id) = child_2_id
        {
            // There isn't a swap API in the gummy tree. Instead swap the children and call set_children.
            let mut tchildren = gummy_tree.children(parent_id).to_vec();

            let i1 = tchildren
                .iter()
                .position(|x| *x == child_1_id)
                .ok_or(RetGuiError::ElementNotFound)
                .expect("Failed to find gummy child");
            let i2 = tchildren
                .iter()
                .position(|x| *x == child_2_id)
                .ok_or(RetGuiError::ElementNotFound)
                .expect("Failed to find gummy child");

            tchildren.swap(i1, i2);

            gummy_tree.set_children(parent_id, &tchildren);
            gummy_tree.mark_dirty(parent_id);
            gummy_tree.request_layout();
        }

        // TODO: Fix. This is likely doing more work than required.
        self.sync_accessibility_children(elements);
        self.request_window_redraw();

        Ok(())
    }

    /// Removes a direct child of this element and returns the removed element.
    ///
    /// # Errors
    /// Returns [`RetGuiError::ElementNotFound`] if `child` is not an immediate child
    /// of this element.
    ///
    /// # Panics
    /// Panics if the corresponding Gummy layout nodes fail to be removed.
    fn remove_child(
        &mut self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        event_queue: &mut VecDeque<EventKind>,
        focus: &mut Option<DynElement>,
        child: DynElement,
    ) -> Result<DynElement, RetGuiError> {
        // Find the child element.
        let children = &mut self.element_data_mut().children;
        let position = children
            .iter()
            .position(|x| *x == child)
            .ok_or(RetGuiError::ElementNotFound)?;

        let child = children[position];

        // Remove the child from this element.

        children.remove(position);

        // Remove the parent reference.
        elements.get_mut(child).element_data_mut().parent = None;

        let child_id = elements.get(child).element_data().layout.gummy_node_id;
        if let Some(child_id) = child_id {
            gummy_tree.unparent_node(child_id);
        }

        let parent_id = self.element_data().layout.gummy_node_id;
        gummy_tree.mark_dirty(parent_id.unwrap());

        fn remove_element_from_document(element: DynElement, pointer_capture: &mut PointerCapture) {
            pointer_capture.remove_element(element);
        }

        if let Some(window) = self.element_data().window {
            let pointer_capture = if window == self.element_data().me {
                self.window_pointer_capture()
                    .expect("a window root must provide pointer capture state")
            } else {
                &mut elements.get_as_mut::<WindowElement>(window).pointer_capture
            };
            remove_element_from_document(child, pointer_capture);
        }

        elements.dispatch_mut(child, |child, elements| {
            child.unfocus(event_queue, focus);
            crate::accessibility::detach_subtree(elements, child);
            child.element_data_mut().window = None;
            child.element_data_mut().redraw_signal = None;
            child.propagate_window_down(elements);
        });
        self.request_window_redraw();

        Ok(child)
    }

    fn remove_all_children(
        &mut self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        event_queue: &mut VecDeque<EventKind>,
        focus: &mut Option<DynElement>,
    ) {
        // @OPTIMIZE: We are copying the vec here.
        for child in self.element_data().children.clone().iter().rev() {
            self.remove_child(elements, gummy_tree, event_queue, focus, *child)
                .unwrap();
        }
    }

    fn child_layout_parent(&self) -> Option<gummy::NodeId> {
        self.element_data().layout.gummy_node_id
    }

    /// Called after a node is added to the gummy tree.
    fn on_post_add_layout_tree(&mut self, _gummy_tree: &mut GummyTree) {}

    fn add_event_listener(&mut self, callback: EventCallbackKind, options: EventListenerOptions) {
        self.element_data_mut().event_callbacks.push(EventCallback {
            callback,
            capturing: options.capturing,
        });
    }

    fn on_pointer_enter(&mut self, on_pointer_enter: PointerEnterHandler) {
        self.add_event_listener(
            EventCallbackKind::PointerEnter(on_pointer_enter),
            EventListenerOptions::default(),
        );
    }

    fn on_slider_value_changed(&mut self, on_slider_value_changed: SliderValueChangedHandler) {
        self.add_event_listener(
            EventCallbackKind::SliderValueChanged(on_slider_value_changed),
            EventListenerOptions::default(),
        );
    }

    fn on_pointer_leave(&mut self, on_pointer_leave: PointerLeaveHandler) {
        self.add_event_listener(
            EventCallbackKind::PointerLeave(on_pointer_leave),
            EventListenerOptions::default(),
        );
    }

    fn on_radio_value_changed(&mut self, on_radio_value_changed: RadioValueChangedHandler) {
        self.add_event_listener(
            EventCallbackKind::RadioValueChanged(on_radio_value_changed),
            EventListenerOptions::default(),
        );
    }

    fn on_checkbox_toggled(&mut self, on_checkbox_toggled: CheckboxToggledHandler) {
        self.add_event_listener(
            EventCallbackKind::CheckboxToggled(on_checkbox_toggled),
            EventListenerOptions::default(),
        );
    }

    fn on_text_input_changed(&mut self, on_text_input_changed: TextInputChangedHandler) {
        self.add_event_listener(
            EventCallbackKind::TextInputChanged(on_text_input_changed),
            EventListenerOptions::default(),
        );
    }

    fn on_got_pointer_capture(&mut self, on_got_pointer_capture: PointerCaptureHandler) {
        self.add_event_listener(
            EventCallbackKind::GotPointerCapture(on_got_pointer_capture),
            EventListenerOptions::default(),
        );
    }

    fn on_lost_pointer_capture(&mut self, on_lost_pointer_capture: PointerCaptureHandler) {
        self.add_event_listener(
            EventCallbackKind::LostPointerCapture(on_lost_pointer_capture),
            EventListenerOptions::default(),
        );
    }

    /// Returns the element's user based id. This id is not used by RetGUI.
    fn get_id(&self) -> Option<smol_str::SmolStr> {
        self.element_data().id.clone()
    }

    /// Sets the element's user based id.
    fn set_id(&mut self, id: &str) {
        self.element_data_mut().id = Some(id.into());
    }

    /// Adds a pointer button down listener.
    fn on_pointer_button_down(&mut self, on_pointer_button_down: PointerEventHandler) {
        self.add_event_listener(
            EventCallbackKind::PointerButtonDown(on_pointer_button_down),
            EventListenerOptions::default(),
        );
    }

    /// Adds a pointer button up listener.
    fn on_pointer_button_up(&mut self, on_pointer_button_up: PointerEventHandler) {
        self.add_event_listener(
            EventCallbackKind::PointerButtonUp(on_pointer_button_up),
            EventListenerOptions::default(),
        );
    }

    fn on_click(&mut self, on_click: ClickHandler) {
        self.add_event_listener(EventCallbackKind::Click(on_click), EventListenerOptions::default());
    }

    fn on_custom_event(&mut self, on_custom_event: CustomHandler) {
        self.add_event_listener(
            EventCallbackKind::Custom(on_custom_event),
            EventListenerOptions::default(),
        );
    }

    fn on_focus(&mut self, on_focus: FocusHandler) {
        self.add_event_listener(EventCallbackKind::Focus(on_focus), EventListenerOptions::default());
    }

    fn on_unfocus(&mut self, on_unfocus: UnfocusHandler) {
        self.add_event_listener(EventCallbackKind::Unfocus(on_unfocus), EventListenerOptions::default());
    }

    fn on_pointer_moved(&mut self, on_pointer_moved: PointerUpdateHandler) {
        self.add_event_listener(
            EventCallbackKind::PointerMoved(on_pointer_moved),
            EventListenerOptions::default(),
        );
    }

    fn on_keyboard_input(&mut self, on_keyboard_input: KeyboardInputHandler) {
        self.add_event_listener(
            EventCallbackKind::KeyboardInput(on_keyboard_input),
            EventListenerOptions::default(),
        );
    }

    fn on_scroll(&mut self, on_scroll: ScrollHandler) {
        self.add_event_listener(EventCallbackKind::Scroll(on_scroll), EventListenerOptions::default());
    }

    fn scroll_to_child_by_id_with_options(
        &mut self,
        elements: &RetainedElements,
        event_queue: &mut VecDeque<EventKind>,
        id: &str,
        options: ScrollOptions,
    ) {
        if crate::elements::scrollable::scroll_to_child_by_id_with_options(
            elements,
            event_queue,
            self.element_data_mut(),
            id,
            options,
        ) {
            self.request_window_redraw();
        }
    }

    fn scroll_to(&mut self, event_queue: &mut VecDeque<EventKind>, y: f32) {
        if crate::elements::scrollable::scroll_to(event_queue, self.element_data_mut(), y) {
            self.request_window_redraw();
        }
    }

    fn scroll_to_top(&mut self, event_queue: &mut VecDeque<EventKind>) {
        if crate::elements::scrollable::scroll_to_top(event_queue, self.element_data_mut()) {
            self.request_window_redraw();
        }
    }

    fn scroll_to_bottom(&mut self, event_queue: &mut VecDeque<EventKind>) {
        if crate::elements::scrollable::scroll_to_bottom(event_queue, self.element_data_mut()) {
            self.request_window_redraw();
        }
    }

    fn scroll_by(&mut self, event_queue: &mut VecDeque<EventKind>, y: f32) {
        if crate::elements::scrollable::scroll_by(event_queue, self.element_data_mut(), y) {
            self.request_window_redraw();
        }
    }

    fn get_scroll_state(&self) -> ScrollState {
        self.element_data().layout.scroll_state
    }

    /// Returns the element's [`ElementBox`].
    fn get_computed_box_transformed(&self) -> ElementBox {
        self.element_data().layout.world_box()
    }

    /// Returns a shared reference to the element's [`Style`].
    fn style(&self) -> &Style {
        &self.element_data().style
    }

    /// Returns a mutable reference to the element's [`Style`].
    fn style_mut(&mut self) -> &mut Style {
        &mut self.element_data_mut().style
    }

    /// Determines if a point is within the bound of the element.
    ///
    /// Visual order and visibility shall not be accounted for.
    fn in_bounds(&self, point: Point) -> bool {
        let element_data = self.element_data();
        let rect = element_data.layout.world_box().border_rectangle();

        if let Some(clip) = element_data.layout.clip_bounds.get() {
            match rect.intersection(&clip) {
                Some(bounds) => bounds.contains(&point),
                None => false,
            }
        } else {
            false
        }
    }

    fn propagate_window_down(&mut self, elements: &mut RetainedElements) {
        let window = self.element_data().window;
        let redraw_signal = self.element_data().redraw_signal.clone();
        for child in self.element_data().children.clone() {
            elements.dispatch_mut(child, |child, elements| {
                child.element_data_mut().window = window;
                child.element_data_mut().redraw_signal = redraw_signal.clone();
                child.propagate_window_down(elements);
            });
        }
    }

    fn set_pointer_capture(&self, elements: &mut RetainedElements, pointer_id: PointerId) {
        // 9.2 Setting pointer capture
        // https://w3c.github.io/pointerevents/#setting-pointer-capture

        // 1. If the pointerId provided as the method's argument does not match any of the active pointers, then throw a "NotFoundError" DOMException.
        // TODO (POINTER CAPTURE)
        // 2. Let the pointer be the active pointer specified by the given pointerId.
        // 3. If the element is not connected [DOM], throw an "InvalidStateError" DOMException.
        // TODO (POINTER CAPTURE)
        // 4. If this method is invoked while the element's node document [DOM] has a locked element ([PointerLock] pointerLockElement), throw an "InvalidStateError" DOMException.
        // TODO (POINTER CAPTURE)
        // 5. If the pointer is not in the active buttons state or the element's node document is not the active document of the pointer, then terminate these steps.
        // TODO (POINTER CAPTURE)
        // 6. For the specified pointerId, set the pending pointer capture target override to the Element on which this method was invoked.
        if let Some(window) = self.element_data().window {
            elements
                .get_as_mut::<WindowElement>(window)
                .pointer_capture
                .pending_pointer_captures
                .insert(pointer_id, self.element_data().me);
        }
    }

    fn release_pointer_capture(&self, elements: &mut RetainedElements, pointer_id: PointerId) {
        // 9.3 Releasing pointer capture
        // https://w3c.github.io/pointerevents/#releasing-pointer-capture
        let has_pointer_capture = self.has_pointer_capture(elements, pointer_id);
        // 1. If the pointerId provided as the method's argument does not match any of the active pointers and these steps are not being invoked as a result of the implicit release of pointer capture, then throw a "NotFoundError" DOMException.
        // TODO (POINTER CAPTURE)
        // 2. If hasPointerCapture is false for the Element with the specified pointerId, then terminate these steps.
        if !has_pointer_capture {
            return;
        }
        // 3. For the specified pointerId, clear the pending pointer capture target override, if set.
        if let Some(window) = self.element_data().window {
            elements
                .get_as_mut::<WindowElement>(window)
                .pointer_capture
                .pending_pointer_captures
                .remove(&pointer_id);
        }
    }

    fn has_pointer_capture(&self, elements: &RetainedElements, pointer_id: PointerId) -> bool {
        // https://w3c.github.io/pointerevents/#dom-element-haspointercapture
        if let Some(window) = self.element_data().window {
            elements
                .get_as::<WindowElement>(window)
                .pointer_capture
                .pending_pointer_captures
                .get(&pointer_id)
                .copied()
                == Some(self.element_data().me)
        } else {
            false
        }
    }

    fn set_style_variant(&mut self, gummy_tree: &mut GummyTree, style: StyleVariant) {
        match style {
            StyleVariant::BoxSizing(value) => self.set_box_sizing(gummy_tree, value),
            StyleVariant::Position(value) => self.set_position(gummy_tree, value),
            StyleVariant::Margin(value) => {
                self.set_margin(gummy_tree, value.top, value.right, value.bottom, value.left)
            }
            StyleVariant::Padding(value) => {
                self.set_padding(gummy_tree, value.top, value.right, value.bottom, value.left)
            }
            StyleVariant::Gap(value) => self.set_gap(gummy_tree, value[0], value[1]),
            StyleVariant::Inset(value) => self.set_inset(gummy_tree, value.top, value.right, value.bottom, value.left),
            StyleVariant::Width(value) => self.set_width(gummy_tree, value),
            StyleVariant::MinWidth(value) => self.set_min_width(gummy_tree, value),
            StyleVariant::MaxWidth(value) => self.set_max_width(gummy_tree, value),
            StyleVariant::Height(value) => self.set_height(gummy_tree, value),
            StyleVariant::MinHeight(value) => self.set_min_height(gummy_tree, value),
            StyleVariant::MaxHeight(value) => self.set_max_height(gummy_tree, value),
            StyleVariant::Display(value) => self.set_display(gummy_tree, value),
            StyleVariant::Wrap(value) => self.set_wrap(gummy_tree, value),
            StyleVariant::AlignItems(value) => self.set_align_items(gummy_tree, value),
            StyleVariant::AlignSelf(value) => self.set_align_self(gummy_tree, value),
            StyleVariant::AlignContent(value) => self.set_align_content(gummy_tree, value),
            StyleVariant::JustifyContent(value) => self.set_justify_content(gummy_tree, value),
            StyleVariant::FlexDirection(value) => self.set_flex_direction(gummy_tree, value),
            StyleVariant::FlexGrow(value) => self.set_flex_grow(gummy_tree, value),
            StyleVariant::FlexShrink(value) => self.set_flex_shrink(gummy_tree, value),
            StyleVariant::FlexBasis(value) => self.set_flex_basis(gummy_tree, value),
            StyleVariant::Order(value) => self.set_order(gummy_tree, value),
            StyleVariant::FontFamily(value) => self.set_font_family(gummy_tree, value),
            StyleVariant::BackgroundBrush(value) => self.set_background_brush(value),
            StyleVariant::TextBrush(value) => self.set_text_brush(gummy_tree, value),
            StyleVariant::LineHeight(value) => self.set_line_height(gummy_tree, value),
            StyleVariant::FontSize(value) => self.set_font_size(gummy_tree, value),
            StyleVariant::FontWeight(value) => self.set_font_weight(gummy_tree, value),
            StyleVariant::FontStyle(value) => self.set_font_style(gummy_tree, value),
            StyleVariant::TextAlign(value) => self.set_text_align(gummy_tree, value),
            StyleVariant::Underline(value) => self.set_underline(gummy_tree, value),
            StyleVariant::Overflow(value) => self.set_overflow(gummy_tree, value[0], value[1]),
            StyleVariant::BorderColor(value) => self.set_border_color(value.top, value.right, value.bottom, value.left),
            StyleVariant::BorderWidth(value) => {
                self.set_border_width(gummy_tree, value.top, value.right, value.bottom, value.left)
            }
            StyleVariant::BorderRadius(value) => self.set_border_radius(value[0], value[1], value[2], value[3]),
            StyleVariant::OutlineColor(value) => {
                self.set_outline_color(value.top, value.right, value.bottom, value.left)
            }
            StyleVariant::OutlineWidth(value) => {
                self.set_outline_width(value.top, value.right, value.bottom, value.left)
            }
            StyleVariant::ScrollbarBrush(value) => self.set_scrollbar_color(value),
            StyleVariant::ScrollbarThumbMargin(value) => {
                self.set_scrollbar_thumb_margin(gummy_tree, value.top, value.right, value.bottom, value.left)
            }
            StyleVariant::ScrollbarThumbRadius(value) => {
                self.set_scrollbar_thumb_radius(gummy_tree, value[0], value[1], value[2], value[3])
            }
            StyleVariant::ScrollbarWidth(value) => self.set_scrollbar_width(gummy_tree, value),
            StyleVariant::Overlay(value) => self.set_overlay(value),
            StyleVariant::Visible(value) => self.set_visible(value),
            StyleVariant::SelectionBrush(value) => self.set_selection_brush(gummy_tree, value),
            StyleVariant::CursorBrush(value) => self.set_cursor_brush(value),
            StyleVariant::BoxShadows(value) => self.set_box_shadows(value),
        }
    }

    fn set_display(&mut self, gummy_tree: &mut GummyTree, display: Display) {
        self.style_mut().set_display(display);
        self.update_gummy_style(gummy_tree);
    }

    fn set_box_sizing(&mut self, gummy_tree: &mut GummyTree, box_sizing: BoxSizing) {
        self.style_mut().set_box_sizing(box_sizing);
        self.update_gummy_style(gummy_tree);
    }

    fn set_position(&mut self, gummy_tree: &mut GummyTree, position: Position) {
        self.style_mut().set_position(position);
        self.update_gummy_style(gummy_tree);
    }

    fn set_overlay(&mut self, overlay: bool) {
        self.style_mut().set_overlay(overlay);
    }

    fn set_visible(&mut self, visible: bool) {
        self.style_mut().set_visible(visible);
        self.request_window_redraw();
    }

    fn set_margin(&mut self, gummy_tree: &mut GummyTree, top: Unit, right: Unit, bottom: Unit, left: Unit) {
        self.style_mut()
            .set_margin(TrblRectangle::new(top, right, bottom, left));
        self.update_gummy_style(gummy_tree);
    }

    fn set_margin_all(&mut self, gummy_tree: &mut GummyTree, value: Unit) {
        self.set_margin(gummy_tree, value, value, value, value);
    }

    fn set_margin_horizontal(&mut self, gummy_tree: &mut GummyTree, value: Unit) {
        let margin = self.style().get_margin();
        self.set_margin(gummy_tree, margin.top, value, margin.bottom, value);
    }

    fn set_margin_vertical(&mut self, gummy_tree: &mut GummyTree, value: Unit) {
        let margin = self.style().get_margin();
        self.set_margin(gummy_tree, value, margin.right, value, margin.left);
    }

    fn set_padding(&mut self, gummy_tree: &mut GummyTree, top: Unit, right: Unit, bottom: Unit, left: Unit) {
        self.style_mut()
            .set_padding(TrblRectangle::new(top, right, bottom, left));
        self.update_gummy_style(gummy_tree);
    }

    fn set_padding_all(&mut self, gummy_tree: &mut GummyTree, value: Unit) {
        self.set_padding(gummy_tree, value, value, value, value);
    }

    fn set_padding_horizontal(&mut self, gummy_tree: &mut GummyTree, value: Unit) {
        let padding = self.style().get_padding();
        self.set_padding(gummy_tree, padding.top, value, padding.bottom, value);
    }

    fn set_padding_vertical(&mut self, gummy_tree: &mut GummyTree, value: Unit) {
        let padding = self.style().get_padding();
        self.set_padding(gummy_tree, value, padding.right, value, padding.left);
    }

    fn set_gap(&mut self, gummy_tree: &mut GummyTree, column_gap: Unit, row_gap: Unit) {
        self.style_mut().set_gap([column_gap, row_gap]);
        self.update_gummy_style(gummy_tree);
    }

    fn set_row_gap(&mut self, gummy_tree: &mut GummyTree, value: Unit) {
        let column_gap = self.style().get_gap()[0];
        self.set_gap(gummy_tree, column_gap, value);
    }

    fn set_column_gap(&mut self, gummy_tree: &mut GummyTree, value: Unit) {
        let row_gap = self.style().get_gap()[1];
        self.set_gap(gummy_tree, value, row_gap);
    }

    fn set_inset(&mut self, gummy_tree: &mut GummyTree, top: Unit, right: Unit, bottom: Unit, left: Unit) {
        self.style_mut().set_inset(TrblRectangle::new(top, right, bottom, left));
        self.update_gummy_style(gummy_tree);
    }

    fn set_min_width(&mut self, gummy_tree: &mut GummyTree, min_width: Unit) {
        self.style_mut().set_min_width(min_width);
        self.update_gummy_style(gummy_tree);
    }

    fn set_min_height(&mut self, gummy_tree: &mut GummyTree, min_height: Unit) {
        self.style_mut().set_min_height(min_height);
        self.update_gummy_style(gummy_tree);
    }

    fn set_width(&mut self, gummy_tree: &mut GummyTree, width: Unit) {
        self.style_mut().set_width(width);
        self.update_gummy_style(gummy_tree);
    }

    fn set_height(&mut self, gummy_tree: &mut GummyTree, height: Unit) {
        self.style_mut().set_height(height);
        self.update_gummy_style(gummy_tree);
    }

    fn set_max_width(&mut self, gummy_tree: &mut GummyTree, max_width: Unit) {
        self.style_mut().set_max_width(max_width);
        self.update_gummy_style(gummy_tree);
    }

    fn set_max_height(&mut self, gummy_tree: &mut GummyTree, max_height: Unit) {
        self.style_mut().set_max_height(max_height);
        self.update_gummy_style(gummy_tree);
    }

    fn set_wrap(&mut self, gummy_tree: &mut GummyTree, wrap: FlexWrap) {
        self.style_mut().set_wrap(wrap);
        self.update_gummy_style(gummy_tree);
    }

    fn set_align_items(&mut self, gummy_tree: &mut GummyTree, align_items: AlignItems) {
        self.style_mut().set_align_items(align_items);
        self.update_gummy_style(gummy_tree);
    }

    fn set_align_self(&mut self, gummy_tree: &mut GummyTree, align_self: AlignSelf) {
        self.style_mut().set_align_self(align_self);
        self.update_gummy_style(gummy_tree);
    }

    fn set_align_content(&mut self, gummy_tree: &mut GummyTree, align_content: AlignContent) {
        self.style_mut().set_align_content(align_content);
        self.update_gummy_style(gummy_tree);
    }

    fn set_justify_content(&mut self, gummy_tree: &mut GummyTree, justify_content: JustifyContent) {
        self.style_mut().set_justify_content(justify_content);
        self.update_gummy_style(gummy_tree);
    }

    fn set_flex_direction(&mut self, gummy_tree: &mut GummyTree, flex_direction: FlexDirection) {
        self.style_mut().set_flex_direction(flex_direction);
        self.update_gummy_style(gummy_tree);
    }

    fn set_flex_grow(&mut self, gummy_tree: &mut GummyTree, flex_grow: f32) {
        self.style_mut().set_flex_grow(flex_grow);
        self.update_gummy_style(gummy_tree);
    }

    fn set_flex_shrink(&mut self, gummy_tree: &mut GummyTree, flex_shrink: f32) {
        self.style_mut().set_flex_shrink(flex_shrink);
        self.update_gummy_style(gummy_tree);
    }

    fn set_flex_basis(&mut self, gummy_tree: &mut GummyTree, flex_basis: Unit) {
        self.style_mut().set_flex_basis(flex_basis);
        self.update_gummy_style(gummy_tree);
    }

    fn set_order(&mut self, gummy_tree: &mut GummyTree, order: i32) {
        self.style_mut().set_order(order);
        self.update_gummy_style(gummy_tree);
    }

    fn set_font_family(&mut self, gummy_tree: &mut GummyTree, font_family: FontFamily) {
        self.style_mut().set_font_family(font_family);
        self.on_text_style_changed();
        self.update_gummy_style(gummy_tree);
    }

    fn set_text_brush(&mut self, gummy_tree: &mut GummyTree, brush: Brush) {
        self.style_mut().set_text_brush(brush);
        self.on_text_style_changed();
        self.update_gummy_style(gummy_tree);
    }

    fn set_background_brush(&mut self, brush: Brush) {
        self.style_mut().set_background_brush(brush);
        self.request_window_redraw();
    }

    fn set_font_size(&mut self, gummy_tree: &mut GummyTree, font_size: f32) {
        self.style_mut().set_font_size(font_size);
        self.on_text_style_changed();
        self.update_gummy_style(gummy_tree);
    }

    fn set_line_height(&mut self, gummy_tree: &mut GummyTree, line_height: f32) {
        self.style_mut().set_line_height(line_height);
        self.on_text_style_changed();
        self.update_gummy_style(gummy_tree);
    }

    fn set_font_weight(&mut self, gummy_tree: &mut GummyTree, font_weight: FontWeight) {
        self.style_mut().set_font_weight(font_weight);
        self.on_text_style_changed();
        self.update_gummy_style(gummy_tree);
    }

    fn set_font_style(&mut self, gummy_tree: &mut GummyTree, font_style: FontStyle) {
        self.style_mut().set_font_style(font_style);
        self.on_text_style_changed();
        self.update_gummy_style(gummy_tree);
    }

    fn set_text_align(&mut self, gummy_tree: &mut GummyTree, text_align: TextAlign) {
        self.style_mut().set_text_align(text_align);
        self.on_text_style_changed();
        self.update_gummy_style(gummy_tree);
    }

    fn set_underline(&mut self, gummy_tree: &mut GummyTree, underline: Option<Underline>) {
        self.style_mut().set_underline(underline);
        self.on_text_style_changed();
        self.update_gummy_style(gummy_tree);
    }

    fn on_text_style_changed(&mut self) {}

    fn set_overflow(&mut self, gummy_tree: &mut GummyTree, overflow_x: Overflow, overflow_y: Overflow) {
        self.style_mut().set_overflow([overflow_x, overflow_y]);
        self.update_gummy_style(gummy_tree);
    }

    fn set_overflow_x(&mut self, gummy_tree: &mut GummyTree, overflow: Overflow) {
        let overflow_y = self.style().get_overflow()[1];
        self.set_overflow(gummy_tree, overflow, overflow_y);
    }

    fn set_overflow_y(&mut self, gummy_tree: &mut GummyTree, overflow: Overflow) {
        let overflow_x = self.style().get_overflow()[0];
        self.set_overflow(gummy_tree, overflow_x, overflow);
    }

    fn set_border_color(&mut self, top: Color, right: Color, bottom: Color, left: Color) {
        self.style_mut()
            .set_border_color(TrblRectangle::new(top, right, bottom, left));
        self.apply_borders(self.element_data().applied_scale_factor);
        self.request_window_redraw();
    }

    fn set_border_color_all(&mut self, value: Color) {
        self.set_border_color(value, value, value, value);
    }

    fn set_border_color_vertical(&mut self, value: Color) {
        let border_color = self.style().get_border_color();
        self.set_border_color(value, border_color.right, value, border_color.left);
    }

    fn set_border_color_horizontal(&mut self, value: Color) {
        let border_color = self.style().get_border_color();
        self.set_border_color(border_color.top, value, border_color.bottom, value);
    }

    fn set_border_width(&mut self, gummy_tree: &mut GummyTree, top: Unit, right: Unit, bottom: Unit, left: Unit) {
        self.style_mut()
            .set_border_width(TrblRectangle::new(top, right, bottom, left));
        self.update_gummy_style(gummy_tree);
    }

    fn set_border_width_all(&mut self, gummy_tree: &mut GummyTree, value: Unit) {
        self.set_border_width(gummy_tree, value, value, value, value);
    }

    fn set_border_width_vertical(&mut self, gummy_tree: &mut GummyTree, value: Unit) {
        let border_width = self.style().get_border_width();
        self.set_border_width(gummy_tree, value, border_width.right, value, border_width.left);
    }

    fn set_border_width_horizontal(&mut self, gummy_tree: &mut GummyTree, value: Unit) {
        let border_width = self.style().get_border_width();
        self.set_border_width(gummy_tree, border_width.top, value, border_width.bottom, value);
    }

    fn set_outline_color(&mut self, top: Color, right: Color, bottom: Color, left: Color) {
        let outline_color = TrblRectangle::new(top, right, bottom, left);
        let data = self.element_data_mut();
        if data.unfocused_outline_color.is_some() {
            data.unfocused_outline_color = Some(outline_color);
        } else {
            data.style.set_outline_color(outline_color);
            data.apply_borders(data.applied_scale_factor);
        }
        self.request_window_redraw();
    }

    fn set_outline_color_all(&mut self, value: Color) {
        self.set_outline_color(value, value, value, value);
    }

    fn set_outline_color_vertical(&mut self, value: Color) {
        let outline_color = self
            .element_data()
            .unfocused_outline_color
            .unwrap_or_else(|| self.style().get_outline_color());
        self.set_outline_color(value, outline_color.right, value, outline_color.left);
    }

    fn set_outline_color_horizontal(&mut self, value: Color) {
        let outline_color = self
            .element_data()
            .unfocused_outline_color
            .unwrap_or_else(|| self.style().get_outline_color());
        self.set_outline_color(outline_color.top, value, outline_color.bottom, value);
    }

    fn set_outline_width(&mut self, top: Unit, right: Unit, bottom: Unit, left: Unit) {
        let outline_width = TrblRectangle::new(top, right, bottom, left);
        let data = self.element_data_mut();
        if data.unfocused_outline_width.is_some() {
            data.unfocused_outline_width = Some(outline_width);
        } else {
            data.style.set_outline_width(outline_width);
            data.apply_borders(data.applied_scale_factor);
        }
        self.request_window_redraw();
    }

    fn set_outline_width_all(&mut self, value: Unit) {
        self.set_outline_width(value, value, value, value);
    }

    fn set_outline_width_vertical(&mut self, value: Unit) {
        let outline_width = self
            .element_data()
            .unfocused_outline_width
            .unwrap_or_else(|| self.style().get_outline_width());
        self.set_outline_width(value, outline_width.right, value, outline_width.left);
    }

    fn set_outline_width_horizontal(&mut self, value: Unit) {
        let outline_width = self
            .element_data()
            .unfocused_outline_width
            .unwrap_or_else(|| self.style().get_outline_width());
        self.set_outline_width(outline_width.top, value, outline_width.bottom, value);
    }

    fn set_border_radius(&mut self, top: (f32, f32), right: (f32, f32), bottom: (f32, f32), left: (f32, f32)) {
        self.style_mut().set_border_radius([top, right, bottom, left]);
        self.apply_borders(self.element_data().applied_scale_factor);
        self.request_window_redraw();
    }

    fn set_border_radius_all(&mut self, value: (f32, f32)) {
        self.set_border_radius(value, value, value, value);
    }

    fn set_border_radius_vertical(&mut self, value: (f32, f32)) {
        let border_radius = self.style().get_border_radius();
        self.set_border_radius(value, border_radius[1], value, border_radius[3]);
    }

    fn set_border_radius_horizontal(&mut self, value: (f32, f32)) {
        let border_radius = self.style().get_border_radius();
        self.set_border_radius(border_radius[0], value, border_radius[2], value);
    }

    fn set_scrollbar_color(&mut self, scrollbar_color: ScrollbarColor) {
        self.style_mut().set_scrollbar_brush(scrollbar_color);
        self.request_window_redraw();
    }

    fn set_scrollbar_thumb_margin(&mut self, gummy_tree: &mut GummyTree, top: f32, right: f32, bottom: f32, left: f32) {
        self.style_mut()
            .set_scrollbar_thumb_margin(TrblRectangle::new(top, right, bottom, left));
        self.refresh_scroll_layout(gummy_tree);
        self.request_window_redraw();
    }

    fn set_scrollbar_thumb_radius(
        &mut self,
        gummy_tree: &mut GummyTree,
        top: (f32, f32),
        right: (f32, f32),
        bottom: (f32, f32),
        left: (f32, f32),
    ) {
        self.style_mut().set_scrollbar_thumb_radius([top, right, bottom, left]);
        self.refresh_scroll_layout(gummy_tree);
        self.request_window_redraw();
    }

    fn set_scrollbar_width(&mut self, gummy_tree: &mut GummyTree, scrollbar_width: f32) {
        self.style_mut().set_scrollbar_width(scrollbar_width);
        self.update_gummy_style(gummy_tree);
    }

    /// Sets the list of animations.
    fn set_animations(&mut self, pending_animation_updates: &mut Vec<(DynElement, bool)>, animations: Vec<Animation>) {
        let element_data = self.element_data_mut();
        element_data.animations = animations;
        crate::elements::internal_helpers::queue_animation_update(pending_animation_updates, element_data.me, true);
        self.request_window_redraw();
    }

    /// Sets the selection color.
    fn set_selection_brush(&mut self, gummy_tree: &mut GummyTree, selection_brush: Brush) {
        self.style_mut().set_selection_brush(selection_brush);
        self.mark_dirty(gummy_tree);
    }

    /// Sets the text cursor color.
    fn set_cursor_brush(&mut self, cursor_brush: Option<Brush>) {
        self.style_mut().set_cursor_brush(cursor_brush);
        self.request_window_redraw();
    }

    /// Sets the box shadows on this element.
    fn set_box_shadows(&mut self, box_shadows: Vec<BoxShadow>) {
        self.style_mut().set_box_shadows(box_shadows);
        self.apply_borders(self.element_data().applied_scale_factor);
        self.request_window_redraw();
    }

    fn refresh_scroll_layout(&mut self, gummy_tree: &mut GummyTree) {
        self.mark_dirty(gummy_tree);
    }

    /// Sets focus on the specified element, if it can be focused.
    ///
    /// The focused element is the element that will receive keyboard and similar events by default.
    fn focus(
        &mut self,
        elements: &mut RetainedElements,
        event_queue: &mut VecDeque<EventKind>,
        focus: &mut Option<DynElement>,
        focus_outline_visible: bool,
    ) {
        // Todo: check if the element is focusable. Should we return a result?
        let me = self.element_data().me;
        let previous_focus = focus.replace(me);
        let focus_changed = previous_focus != Some(me);
        {
            if let Some(previous) = previous_focus.filter(|previous| *previous != me) {
                let previous = elements.get(previous);
                let data = previous.element_data();
                if let Some(root) = data.access_root {
                    data.access_tree.set_focus(root, Some(root));
                }
                previous.request_window_redraw();
            }

            let data = self.element_data();
            if let Some((root, node)) = data.access_root.zip(data.access_key) {
                data.access_tree.set_focus(root, Some(node));
            }
        }
        if focus_changed {
            if let Some(previous) = previous_focus.filter(|previous| elements.contains(*previous)) {
                restore_unfocused_outline(elements.get_mut(previous).element_data_mut());
                elements.get_mut(previous).element_data_mut().focused = false;
                event_queue.push_back(EventKind::Unfocus(UnfocusEvent::new(previous)));
            }
            if focus_outline_visible {
                apply_focused_outline(self.element_data_mut());
            }
            self.element_data_mut().focused = true;
            event_queue.push_back(EventKind::Focus(FocusEvent::new(me)));
            self.request_window_redraw();
        }
    }

    /// Returns true if the element has focus.
    fn is_focused(&self) -> bool {
        self.element_data().focused
    }

    /// Removes focus if the element has focus.
    fn unfocus(&mut self, event_queue: &mut VecDeque<EventKind>, focus: &mut Option<DynElement>) {
        if self.is_focused() {
            let me = self.element_data().me;
            *focus = None;
            {
                let data = self.element_data();
                if let Some(root) = data.access_root {
                    data.access_tree.set_focus(root, Some(root));
                }
            }
            restore_unfocused_outline(self.element_data_mut());
            self.element_data_mut().focused = false;
            event_queue.push_back(EventKind::Unfocus(UnfocusEvent::new(me)));
            self.request_window_redraw();
        }
    }
    fn to_dyn_element(&self) -> DynElement {
        self.element_data().me
    }

    /// Use the element's window to request a redraw.
    fn request_window_redraw(&self) {
        if let Some(signal) = &self.element_data().redraw_signal {
            signal.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }

    fn on_access_event(
        &mut self,
        _elements: &mut RetainedElements,
        _event_queue: &mut VecDeque<EventKind>,
        _states: &mut ElementStates,
        _event: AccessEvent,
    ) -> Result<(), IsshoError> {
        Ok(())
    }

    /// Advances this element's animations and returns when it needs another update.
    fn animation_tick(
        &mut self,
        _elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        _states: &mut ElementStates,
        _pending_resources: &mut VecDeque<(
            retgui_resource_manager::ResourceId,
            retgui_resource_manager::resource_type::ResourceType,
        )>,
        delta: Duration,
    ) -> AnimationSchedule {
        self.tick_style_animations(gummy_tree, delta)
    }

    /// Advances the style animations stored in this element's data.
    fn tick_style_animations(&mut self, gummy_tree: &mut GummyTree, delta: Duration) -> AnimationSchedule {
        let mut animations = std::mem::take(&mut self.element_data_mut().animations);
        for animation in &mut animations {
            animation.tick(delta);
            if animation.key_frames.len() >= 2 {
                animation.apply_styles(&mut |style| self.set_style_variant(gummy_tree, style));
            }
        }
        let has_active_style_animation = animations
            .iter()
            .any(|animation| animation.key_frames.len() >= 2 && !animation.is_finished());
        self.element_data_mut().animations = animations;
        if has_active_style_animation {
            AnimationSchedule::NextFrame
        } else {
            AnimationSchedule::None
        }
    }
}

fn apply_focused_outline(data: &mut crate::elements::element_data::ElementData) {
    let outline_color = data.style.get_outline_color();
    let outline_width = data.style.get_outline_width();
    data.unfocused_outline_color.get_or_insert(outline_color);
    data.unfocused_outline_width.get_or_insert(outline_width);
    data.style
        .set_outline_color(TrblRectangle::new_all(crate::palette::css::DODGER_BLUE));
    data.style.set_outline_width(TrblRectangle::new_all(Unit::Px(2.0)));
    data.apply_borders(data.applied_scale_factor);
}

pub(crate) fn set_focus_outline_visible(
    elements: &mut RetainedElements,
    focus: Option<DynElement>,
    focus_outline_visible: &mut bool,
    visible: bool,
) {
    let changed = std::mem::replace(focus_outline_visible, visible) != visible;
    if !changed {
        return;
    }

    let focused = focus.filter(|focused| elements.contains(*focused));
    let Some(focused) = focused else {
        return;
    };

    let focused = elements.get_mut(focused);
    if visible {
        apply_focused_outline(focused.element_data_mut());
    } else {
        restore_unfocused_outline(focused.element_data_mut());
    }
    focused.request_window_redraw();
}

fn restore_unfocused_outline(data: &mut crate::elements::element_data::ElementData) {
    if let Some(outline_color) = data.unfocused_outline_color.take() {
        data.style.set_outline_color(outline_color);
    }
    if let Some(outline_width) = data.unfocused_outline_width.take() {
        data.style.set_outline_width(outline_width);
    }
    data.apply_borders(data.applied_scale_factor);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::App;
    use crate::elements::{Button, Element};

    #[test]
    fn outline_changes_while_focused_are_restored_on_unfocus() {
        let original_color = Color::from_rgb8(10, 20, 30);
        let updated_color = Color::from_rgb8(40, 50, 60);
        let mut app = App::new();
        let button = Button::new(&mut app);
        button.set_outline_color_all(&mut app, original_color);
        button.set_outline_width_all(&mut app, Unit::Px(1.0));

        button.focus(&mut app);
        button.set_outline_color_all(&mut app, updated_color);
        button.set_outline_width_all(&mut app, Unit::Px(3.0));

        assert_eq!(
            app.get(button.as_dyn_element()).style().get_outline_color(),
            TrblRectangle::new_all(crate::palette::css::DODGER_BLUE)
        );
        assert_eq!(
            app.get(button.as_dyn_element()).style().get_outline_width(),
            TrblRectangle::new_all(Unit::Px(2.0))
        );

        button.unfocus(&mut app);

        assert_eq!(
            app.get(button.as_dyn_element()).style().get_outline_color(),
            TrblRectangle::new_all(updated_color)
        );
        assert_eq!(
            app.get(button.as_dyn_element()).style().get_outline_width(),
            TrblRectangle::new_all(Unit::Px(3.0))
        );
    }
}
