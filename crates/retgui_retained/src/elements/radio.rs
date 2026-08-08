//! A toggleable circle.

use std::any::Any;
use std::cell::{Ref, RefCell, RefMut};
use std::ops::DerefMut;
use std::rc::{Rc, Weak};
use std::sync::Arc;

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
            self.active_value.replace(self.value.clone());
            {
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
            }
            let new_event = Event::new(self.element_data.me.upgrade().unwrap());
            queue_event(new_event, EventKind::RadioValueChanged(self.active_value.clone()));
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
}

impl RadioInner {
    fn is_selected(&self) -> bool {
        self.active_value.borrow().as_str() == self.value
    }

    fn set_accessibility_selection(&mut self) {
        self.element_data
            .set_accessibility_value(if self.is_selected() { "selected" } else { "not selected" });
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
        self.inner.borrow_mut().hide_radio = value;
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
