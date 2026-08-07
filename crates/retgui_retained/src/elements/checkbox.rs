//! A toggleable checkbox.

use crate::app::{GUMMY_TREE, queue_event};
use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::{apply_generic_container_layout, apply_generic_container_layout_non_dom, push_child_to_element};
use crate::elements::traits::DeepClone;
use crate::elements::{AsElement, Element, ElementInternals, resolve_clip_for_scrollable, scrollable};
use crate::events::{CheckboxToggled, Event, EventKind};
use crate::layout::GummyTree;
use crate::style::{Overflow, Unit};
use crate::text::text_context::TextContext;
use crate::{auto, px, rgb};
use retgui_primitives::geometry::{Affine, Point, Rectangle, TrblRectangle};
use retgui_renderer::Brush;
use retgui_renderer::renderer::Renderer;
use retgui_resource_manager::ResourceManager;
use peniko::kurbo;
use std::any::Any;
use std::cell::{Ref, RefCell, RefMut};
use std::rc::{Rc, Weak};
use std::sync::Arc;

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
            &mut self.box_layout,
            gummy_tree,
            p,
            z_index,
            child_transform,
            clip_bounds,
            scale_factor,
        );
        self.box_rect = self.box_layout.layout.computed_box_transformed.content_rectangle();
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

        let color = rgb(0, 100, 255);
        let border_color = if self.checked { color } else { rgb(150, 150, 150) };
        renderer.draw_rect_outline(
            self.box_rect.scale(_scale_factor),
            Brush::Color(border_color),
            2.0 * _scale_factor,
        );

        let s = self.box_rect;
        let blue = rgb(0, 100, 255);
        let grey = rgb(150, 150, 150);
        if self.checked {
            renderer.draw_rect(s.scale(_scale_factor), Brush::Color(blue));

            let scale_factor = _scale_factor as f32;
            let mut path = kurbo::BezPath::new();
            path.move_to((
                ((s.x + s.width * 0.25) * scale_factor) as f64,
                ((s.y + s.height * 0.5) * scale_factor) as f64,
            ));
            path.line_to((
                ((s.x + s.width * 0.45) * scale_factor) as f64,
                ((s.y + s.height * 0.7) * scale_factor) as f64,
            ));
            path.line_to((
                ((s.x + s.width * 0.75) * scale_factor) as f64,
                ((s.y + s.height * 0.3) * scale_factor) as f64,
            ));

            renderer.stroke_bez_path(path, Brush::Color(rgb(255, 255, 255)));
        } else {
            renderer.draw_rect_outline(s.scale(_scale_factor), Brush::Color(grey), 1.5 * _scale_factor);
        }

        self.draw_children(renderer, resource_manager.clone(), _scale_factor, _text_context);
        self.maybe_end_layer(renderer);
        self.draw_scrollbar(renderer, _scale_factor);
    }

    fn on_event(&mut self, message: &EventKind, _text_context: &mut TextContext, event: &mut Event) {
        scrollable::handle_scroll_logic(self, message, event);
        if let EventKind::PointerButtonUp(_) = message {
            self.toggle();
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
                box_layout: ElementData::new_pseudo(me.clone(), false),
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

        GUMMY_TREE.with_borrow_mut(|gummy_tree| {
            gummy_tree.add_child(
                inner_mut.element_data.layout.gummy_node_id(),
                inner_mut.box_layout.layout.gummy_node_id(),
            );
        });
        {
            inner_mut.element_data.set_accessibility_role(issho::Role::CheckBox);
            inner_mut.element_data.set_accessibility_name(label.to_string());
            inner_mut.element_data.set_accessibility_checked(checked);
            let inner = Rc::downgrade(&inner);
            inner_mut.element_data.set_accessibility_toggle_action(move || {
                if let Some(inner) = inner.upgrade() {
                    inner.borrow_mut().toggle();
                }
            });
        }

        drop(inner_mut);
        Self { inner }
    }
}

impl CheckboxInner {
    fn toggle(&mut self) {
        self.checked = !self.checked;
        self.element_data.set_accessibility_checked(self.checked);
        let target = self
            .element_data
            .me
            .upgrade()
            .expect("checkbox was detached while handling its toggle action");
        queue_event(
            Event::new(target),
            EventKind::CheckboxToggled(CheckboxToggled {
                label: self.label.clone(),
                status: self.checked,
            }),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::Checkbox;
    use crate::app::dequeue_event;
    use crate::elements::ElementData as _;

    #[test]
    fn toggle_updates_the_retained_checked_state_immediately() {
        let checkbox = Checkbox::new("Choice", true);
        let (tree, key) = {
            let checkbox = checkbox.inner.borrow();
            let data = checkbox.element_data();
            (data.access_tree.clone(), data.access_key.unwrap())
        };

        assert!(tree.get_node(key).unwrap().checked());

        checkbox.inner.borrow_mut().toggle();

        assert!(!tree.get_node(key).unwrap().checked());
        while dequeue_event().is_some() {}
    }
}
