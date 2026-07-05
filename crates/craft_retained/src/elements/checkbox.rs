//! A toggleable checkbox.

use std::any::Any;
use std::cell::{Ref, RefCell, RefMut};
use std::rc::{Rc, Weak};

#[cfg(feature = "accesskit")]
use accesskit::{Action, Role, Toggled, TreeUpdate};
use craft_primitives::geometry::{Affine, Point, Rectangle, TrblRectangle};
use craft_renderer::{Brush, RenderList};
use peniko::kurbo;

use crate::app::{TAFFY_TREE, queue_event};
use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::{apply_generic_container_layout, apply_generic_container_layout_non_dom, push_child_to_element};
use crate::elements::traits::DeepClone;
use crate::elements::{AsElement, Element, ElementData as ElementDataTrait, ElementInternals, resolve_clip_for_scrollable, scrollable};
use crate::events::{CheckboxToggled, Event, EventKind};
use crate::layout::TaffyTree;
use crate::style::{Overflow, Unit};
use crate::text::text_context::TextContext;
use crate::{auto, px, rgb};

#[derive(Clone)]
pub struct Checkbox {
    pub inner: Rc<RefCell<CheckboxInner>>,
}

#[derive(Clone)]
pub struct CheckboxInner {
    element_data: ElementData,
    box_layout: ElementData,
    box_rect: Rectangle,
    label: String,
    checked: bool,
}

impl Default for Checkbox {
    fn default() -> Self {
        Self::new("checkbox item", false)
    }
}

impl Element for Checkbox {}

impl Drop for CheckboxInner {
    fn drop(&mut self) {
        ElementInternals::drop(self)
    }
}

impl AsElement for Checkbox {
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

impl crate::elements::ElementData for CheckboxInner {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementInternals for CheckboxInner {
    fn deep_clone(&self) -> Rc<RefCell<dyn ElementInternals>> {
        self.deep_clone_internal()
    }

    fn apply_layout(
        &mut self,
        taffy_tree: &mut TaffyTree,
        position: Point,
        z_index: &mut u32,
        transform: Affine,
        text_context: &mut TextContext,
        clip_bounds: Option<Rectangle>,
        scale_factor: f64,
    ) {
        apply_generic_container_layout(
            self,
            taffy_tree,
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
            &mut self.box_layout,
            taffy_tree,
            p,
            z_index,
            child_transform,
            clip_bounds,
            scale_factor,
        );
        self.box_rect = self.box_layout.layout.computed_box_transformed.content_rectangle();
    }

    fn draw(&mut self, renderer: &mut RenderList, text_context: &mut TextContext, scale_factor: f64) {
        if !self.is_visible() {
            return;
        }
        self.add_hit_testable(renderer, true, scale_factor);
        self.draw_borders(renderer, scale_factor);
        self.maybe_start_layer(renderer, scale_factor);

        let color = rgb(0, 100, 255);
        let border_color = if self.checked { color } else { rgb(150, 150, 150) };
        renderer.draw_rect_outline(self.box_rect.scale(scale_factor), border_color, 2.0 * scale_factor);

        let s = self.box_rect;
        let blue = rgb(0, 100, 255);
        let grey = rgb(150, 150, 150);
        if self.checked {
            renderer.draw_rect(s.scale(scale_factor), blue);

            let mut path = kurbo::BezPath::new();
            path.move_to(((s.x + s.width * 0.25) as f64, (s.y + s.height * 0.5) as f64));
            path.line_to(((s.x + s.width * 0.45) as f64, (s.y + s.height * 0.7) as f64));
            path.line_to(((s.x + s.width * 0.75) as f64, (s.y + s.height * 0.3) as f64));

            renderer.stroke_bez_path(path, Brush::Color(rgb(255, 255, 255)));
        } else {
            renderer.draw_rect_outline(s.scale(scale_factor), grey, 1.5 * scale_factor);
        }

        self.draw_children(renderer, text_context, scale_factor);
        self.maybe_end_layer(renderer);
        self.draw_scrollbar(renderer, scale_factor);
    }

    #[cfg(feature = "accesskit")]
    fn compute_accessibility_tree(&mut self, tree: &mut TreeUpdate, parent_index: Option<usize>, scale_factor: f64) {
        let current_node_id = accesskit::NodeId(self.element_data().internal_id);
        let mut current_node = accesskit::Node::new(Role::CheckBox);
        current_node.set_label(self.label.clone());
        current_node.add_action(Action::Click);
        current_node.set_toggled(if self.checked { Toggled::True } else { Toggled::False });

        crate::elements::internal_helpers::add_generic_accesskit_data(
            &mut self.element_data,
            current_node,
            current_node_id,
            tree,
            parent_index,
            scale_factor,
        )
    }

    fn on_event(
        &mut self,
        message: &EventKind,
        _text_context: &mut TextContext,
        event: &mut Event,
        _target: Option<Rc<RefCell<dyn ElementInternals>>>,
    ) {
        scrollable::handle_scroll_logic(self, message, event);
        if let EventKind::PointerButtonUp(_) = message {
            self.checked = !self.checked;
            let new_event = Event::new(event.target.clone());
            queue_event(
                new_event,
                EventKind::CheckboxToggled(CheckboxToggled {
                    label: self.label.clone(),
                    status: self.checked,
                }),
            );
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

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Checkbox {
    pub fn new(label: &str, checked: bool) -> Self {
        let size = 16.0;
        let inner = Rc::new_cyclic(|me: &Weak<RefCell<CheckboxInner>>| {
            RefCell::new(CheckboxInner {
                element_data: ElementData::new(me.clone(), true),
                box_layout: ElementData::new(me.clone(), false),
                box_rect: Rectangle::new(0.0, 0.0, size, size),
                label: label.to_string(),
                checked,
            })
        });

        let mut inner_mut = inner.borrow_mut();
        inner_mut.element_data.create_layout_node(None);
        inner_mut.box_layout.style.set_min_width(Unit::Px(size));
        inner_mut.box_layout.style.set_min_height(Unit::Px(size));
        inner_mut
            .box_layout
            .style
            .set_margin(TrblRectangle::new(auto(), px(5), auto(), px(0)));
        inner_mut.box_layout.create_layout_node(None);

        TAFFY_TREE.with_borrow_mut(|taffy_tree| {
            taffy_tree.add_child(
                inner_mut.element_data.layout.taffy_node_id(),
                inner_mut.box_layout.layout.taffy_node_id(),
            );
        });

        drop(inner_mut);
        Self { inner }
    }
}
