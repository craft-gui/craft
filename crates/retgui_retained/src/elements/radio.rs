//! A selectable circle.

use std::any::Any;
use std::cell::{Ref, RefCell, RefMut};
use std::ops::DerefMut;
use std::rc::{Rc, Weak};
use std::sync::Arc;

use issho::{AccessEvent, IsshoError, SelectionData, SelectionGroupItem};

use retgui_primitives::brush::Brush;
use retgui_primitives::geometry::{Affine, Circle, Point, Rectangle, TrblRectangle};

use retgui_renderer::renderer::Renderer;
use retgui_resource_manager::ResourceManager;

use crate::app::{GUMMY_TREE, queue_event};
use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::{apply_generic_container_layout, apply_generic_container_layout_non_dom, push_child_to_element};
use crate::elements::traits::DeepClone;
use crate::elements::{AsElement, Element, ElementInternals, resolve_clip_for_scrollable, scrollable};
use crate::events::{Event, EventKind};
use crate::layout::GummyTree;
use crate::style::{Overflow, Unit};
use crate::text::text_context::TextContext;
use crate::{auto, px, rgb};

#[derive(Clone)]
pub struct Radio {
    pub inner: Rc<RefCell<RadioInner>>,
}

/// Stores one or more elements.
///
/// If overflow is set to scroll, it will become scrollable.
#[derive(Clone)]
pub struct RadioInner {
    element_data: ElementData,
    circle_layout: ElementData,
    circle: Circle,
    value: String,
    label: String,
    hide_radio: bool,
    active_value: Rc<RefCell<String>>,
}

impl Default for Radio {
    fn default() -> Self {
        Self::new("", "radio item", Rc::new(RefCell::new("".to_string())))
    }
}

impl Element for Radio {}

impl Drop for RadioInner {
    fn drop(&mut self) {
        ElementInternals::drop(self)
    }
}

impl AsElement for Radio {
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

impl crate::elements::ElementData for RadioInner {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementInternals for RadioInner {
    fn deep_clone(&self) -> Rc<RefCell<dyn ElementInternals>> {
        self.deep_clone_internal()
    }

    fn apply_layout(
        &mut self,
        gummy_tree: &mut GummyTree,
        position: Point,
        z_index: &mut u32,
        transform: Affine,
        text_context: &mut TextContext,
        clip_bounds: Option<Rectangle>,
        scale_factor: f64,
    ) {
        apply_generic_container_layout(
            self,
            gummy_tree,
            position,
            z_index,
            transform,
            text_context,
            clip_bounds,
            scale_factor,
        );
        let p = self.element_data.layout.computed_box_transformed.position;
        let scroll_y = self.element_data.scroll().scroll_y() as f64;
        let child_transform = Affine::translate((0.0, -scroll_y));

        apply_generic_container_layout_non_dom(
            &mut self.circle_layout,
            gummy_tree,
            p,
            z_index,
            child_transform,
            clip_bounds,
            scale_factor,
        );
        self.circle.x = self.circle_layout.layout.computed_box_transformed.content_rectangle().x + self.circle.radius;
        self.circle.y = self.circle_layout.layout.computed_box_transformed.content_rectangle().y + self.circle.radius;
    }

    fn draw(
        &mut self,
        renderer: &mut dyn Renderer,
        resource_manager: Arc<ResourceManager>,
        _scale_factor: f64,
        _text_context: &mut TextContext,
    ) {
        if !self.is_visible() {
            return;
        }
        self.add_hit_testable(renderer, true, _scale_factor);
        self.draw_borders(renderer, _scale_factor);
        self.maybe_start_layer(renderer, _scale_factor);

        if !self.hide_radio {
            if self.is_selected() {
                renderer.draw_circle_outline(
                    self.circle.scale(_scale_factor),
                    Brush::Color(rgb(0, 100, 255)),
                    _scale_factor as f32,
                );
                renderer.draw_circle(
                    self.circle.expand(-4.0).scale(_scale_factor),
                    Brush::Color(rgb(0, 100, 255)),
                );
            } else {
                renderer.draw_circle_outline(
                    self.circle.scale(_scale_factor),
                    Brush::Color(rgb(150, 150, 150)),
                    _scale_factor as f32,
                );
            }
        }

        self.draw_children(renderer, resource_manager, _scale_factor, _text_context);
        self.maybe_end_layer(renderer);
        self.draw_scrollbar(renderer, _scale_factor);
    }

    fn on_event(&mut self, message: &EventKind, _text_context: &mut TextContext, event: &mut Event) {
        scrollable::handle_scroll_logic(self, message, event);
        if let EventKind::PointerButtonUp(_) = message {
            self.set_value();
        }
    }

    fn apply_clip(&mut self, clip_bounds: Option<Rectangle>) {
        let overflow = self.style().get_overflow();
        if overflow[0] == Overflow::Scroll || overflow[1] == Overflow::Scroll {
            resolve_clip_for_scrollable(self, clip_bounds);
        } else {
            self.element_data.layout.apply_clip(clip_bounds);
        }
    }

    fn push(&mut self, child: Rc<RefCell<dyn ElementInternals>>) {
        push_child_to_element(self, child);
    }

    fn on_access_event(&mut self, event: AccessEvent) -> Result<(), IsshoError> {
        if matches!(event, AccessEvent::Select | AccessEvent::AddToSelection) && !self.is_selected() {
            self.set_value();
        }
        Ok(())
    }
}

impl RadioInner {
    fn set_value(&mut self) {
        let selection_changed = !self.is_selected();
        self.active_value.replace(self.value.clone());
        self.set_accessibility_selection();

        let me = self.element_data.me.upgrade();
        let parent = self.element_data.parent.as_ref().and_then(Weak::upgrade);
        if let Some(parent) = parent {
            for sibling in parent.borrow().children().to_vec() {
                if me.as_ref().is_some_and(|me| Rc::ptr_eq(me, &sibling)) {
                    continue;
                }
                if let Some(radio) = (sibling.borrow_mut().deref_mut() as &mut dyn Any).downcast_mut::<RadioInner>() {
                    radio.set_accessibility_selection();
                }
            }
        }
        let new_event = Event::new(self.element_data.me.upgrade().unwrap());
        queue_event(new_event, EventKind::RadioValueChanged(self.active_value.clone()));
        if selection_changed {
            self.request_window_redraw();
        }
    }

    fn is_selected(&self) -> bool {
        self.active_value.borrow().as_str() == self.value
    }

    fn set_accessibility_selection(&mut self) {
        let is_selected = self.is_selected();
        self.element_data
            .set_accessibility_selection_data(Some(SelectionData::SelectionGroupItem(SelectionGroupItem {
                is_selected,
            })));
    }
}

impl Radio {
    pub fn new(value: &str, label: &str, active_value: Rc<RefCell<String>>) -> Self {
        let radius = 7.0;
        let inner = Rc::new_cyclic(|me: &Weak<RefCell<RadioInner>>| {
            RefCell::new(RadioInner {
                element_data: ElementData::new(me.clone(), true),
                circle_layout: ElementData::new_pseudo(me.clone(), false),
                circle: Circle::new(0.0, 0.0, radius),
                value: value.to_string(),
                label: label.to_string(),
                hide_radio: false,
                active_value,
            })
        });
        let mut inner_mut = inner.borrow_mut();
        inner_mut.element_data.create_layout_node(None);

        inner_mut.circle_layout.style.set_min_width(Unit::Px(radius * 2.0));
        inner_mut.circle_layout.style.set_min_height(Unit::Px(radius * 2.0));
        inner_mut
            .circle_layout
            .style
            .set_margin(TrblRectangle::new(auto(), px(5), auto(), px(0)));
        inner_mut.circle_layout.create_layout_node(None);
        GUMMY_TREE.with_borrow_mut(|gummy_tree| {
            let node_id = inner_mut.circle_layout.layout.gummy_node_id();
            gummy_tree.add_child(inner_mut.element_data.layout.gummy_node_id(), node_id);
        });
        {
            inner_mut.element_data.set_accessibility_role(issho::Role::RadioButton);
            inner_mut.element_data.set_accessibility_name(label.to_string());
            inner_mut.set_accessibility_selection();
        }

        drop(inner_mut);
        Self { inner }
    }

    /// Hide the default circle radio button.
    pub fn set_hide_radio(&mut self, value: bool) {
        // TODO: Hide in gummy.
        let mut inner = self.inner.borrow_mut();
        inner.hide_radio = value;
        inner.request_window_redraw();
    }

    /// Hide the default circle radio button.
    pub fn hide_radio(mut self) -> Self {
        self.set_hide_radio(true);
        self
    }

    pub fn get_label(&self) -> String {
        self.inner.borrow().label.clone()
    }

    pub fn get_value(&self) -> String {
        self.inner.borrow().value.clone()
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use issho::{AccessEvent, SelectionData};

    use super::Radio;
    use crate::app::dequeue_event;
    use crate::elements::{ElementData as _, ElementInternals as _, RadioGroup};

    #[test]
    fn radio_group_and_items_retain_selection_state() {
        let active_value = Rc::new(RefCell::new("first".to_string()));
        let group = RadioGroup::new("Choices");
        let first = Radio::new("first", "First", active_value.clone());
        let second = Radio::new("second", "Second", active_value.clone());

        group.inner.borrow_mut().push(first.inner.clone());
        group.inner.borrow_mut().push(second.inner.clone());

        let (tree, group_key) = {
            let group = group.inner.borrow();
            let data = group.element_data();
            (data.access_tree.clone(), data.access_key.unwrap())
        };
        let first_key = first.inner.borrow().element_data().access_key.unwrap();
        let second_key = second.inner.borrow().element_data().access_key.unwrap();

        {
            let group = tree.get_node(group_key).unwrap();
            let Some(SelectionData::SelectionGroup(selection)) = group.selection_data() else {
                panic!("radio group was not exposed as a selection group");
            };
            assert!(selection.is_mandatory);
            assert!(!selection.multiple_selectable);
        }
        assert_eq!(tree.get_parent(first_key), Some(group_key));
        assert_eq!(tree.get_parent(second_key), Some(group_key));
        assert!(selection_item_state(&tree, first_key));
        assert!(!selection_item_state(&tree, second_key));
        assert!(!tree.get_node(first_key).unwrap().checked());
        assert_eq!(tree.get_node(first_key).unwrap().value(), "");

        assert!(tree.dispatch_access_event(second_key, AccessEvent::Select).is_ok());

        assert_eq!(active_value.borrow().as_str(), "second");
        assert!(!selection_item_state(&tree, first_key));
        assert!(selection_item_state(&tree, second_key));
        assert!(!tree.get_node(second_key).unwrap().checked());
        assert_eq!(tree.get_node(second_key).unwrap().value(), "");

        while dequeue_event().is_some() {}
    }

    #[test]
    fn radio_accessibility_uses_selection_actions_instead_of_toggle() {
        while dequeue_event().is_some() {}

        let active_value = Rc::new(RefCell::new("second".to_string()));
        let group = RadioGroup::new("Choices");
        let first = Radio::new("first", "First", active_value.clone());
        let second = Radio::new("second", "Second", active_value.clone());
        group.inner.borrow_mut().push(first.inner.clone());
        group.inner.borrow_mut().push(second.inner.clone());
        let tree = first.inner.borrow().element_data().access_tree.clone();
        let first_key = first.inner.borrow().element_data().access_key.unwrap();
        let second_key = second.inner.borrow().element_data().access_key.unwrap();

        assert!(tree.dispatch_access_event(second_key, AccessEvent::Select).is_ok());
        assert!(dequeue_event().is_none());

        assert!(tree.dispatch_access_event(first_key, AccessEvent::Toggle).is_ok());
        assert_eq!(active_value.borrow().as_str(), "second");
        assert!(dequeue_event().is_none());

        assert!(
            tree.dispatch_access_event(first_key, AccessEvent::AddToSelection)
                .is_ok()
        );
        assert_eq!(active_value.borrow().as_str(), "first");
        assert!(dequeue_event().is_some());
        while dequeue_event().is_some() {}

        assert!(tree.dispatch_access_event(first_key, AccessEvent::UnSelect).is_ok());
        assert_eq!(active_value.borrow().as_str(), "first");
        assert!(dequeue_event().is_none());

        while dequeue_event().is_some() {}
    }

    fn selection_item_state(tree: &crate::accessibility::RetGuiAccessTree, key: issho::AccessKey) -> bool {
        let node = tree.get_node(key).unwrap();
        let Some(SelectionData::SelectionGroupItem(item)) = node.selection_data() else {
            panic!("radio was not exposed as a selection group item");
        };
        item.is_selected
    }
}
