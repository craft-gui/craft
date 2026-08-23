//! A toggleable checkbox.

use std::cell::{Ref, RefCell, RefMut};
use std::rc::{Rc, Weak};
use std::sync::Arc;

use issho::{AccessEvent, IsshoError};

use peniko::kurbo;

use retgui_primitives::geometry::{Affine, Rectangle, TrblRectangle};

use retgui_renderer::Brush;
use retgui_renderer::renderer::Renderer;

use retgui_resource_manager::ResourceManager;

use ui_events::keyboard::{Code, KeyState};

use crate::app::{GUMMY_TREE, queue_event};
use crate::elements::element_data::ElementData;
use crate::elements::element_id::create_unique_element_id;
use crate::elements::internal_helpers::{apply_generic_container_layout, apply_generic_container_layout_non_dom, push_child_to_element};
use crate::elements::traits::clone_element;
use crate::elements::{AsElement, Element, ElementInternals, scrollable};
use crate::events::{CheckboxToggledEvent, EventKind};
use crate::layout::GummyTree;
use crate::style::Unit;
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
        clone_element::<Self, _>(self, |element, gummy_tree| {
            let mut element = element.borrow_mut();
            let owner_id = element.element_data.internal_id;
            let owner = element.element_data.me.clone();
            let parent = element.element_data.layout.gummy_node_id();
            let box_node = gummy_tree.clone_node(element.box_layout.layout.gummy_node_id());
            element.box_layout.layout.gummy_node_id = Some(box_node);
            element.box_layout.internal_id = create_unique_element_id();
            element.box_layout.me = owner.clone();
            gummy_tree.add_child(parent, box_node);
            gummy_tree.register_owner(box_node, owner_id, owner);
            Some(parent)
        })
    }

    fn apply_layout(
        &mut self,
        gummy_tree: &mut GummyTree,
        z_index: &mut u32,
        _text_context: &mut TextContext,
        scale_factor: f64,
    ) {
        apply_generic_container_layout(self, gummy_tree, z_index, scale_factor);
        apply_generic_container_layout_non_dom(&mut self.box_layout, gummy_tree, z_index, scale_factor);
        self.box_rect = self.box_layout.layout.local_box_in_parent().content_rectangle();
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

        self.maybe_start_overlay(renderer);

        self.add_hit_testable(renderer, true, _scale_factor);
        self.draw_borders(renderer, _scale_factor);
        self.maybe_start_layer(renderer, _scale_factor);

        let container_transform = renderer.get_transform();
        let scroll_y = self.element_data.scroll().scroll_y() as f64 * _scale_factor;
        renderer.set_transform(container_transform * Affine::translate((0.0, -scroll_y)));

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

        renderer.set_transform(container_transform);

        self.draw_children(renderer, resource_manager.clone(), _scale_factor, _text_context);
        self.maybe_end_layer(renderer);
        self.draw_scrollbar(renderer, _scale_factor);

        self.maybe_end_overlay(renderer);
    }

    fn on_event(&mut self, event: &mut EventKind, _text_context: &mut TextContext) {
        scrollable::handle_scroll_logic(self, event);
        if let EventKind::Click(_) = event {
            self.toggle();
            self.focus();
        } else if self.is_focused()
            && let EventKind::KeyDown(key) = event
            && key.code == Code::Space
            && key.state == KeyState::Down
        {
            self.toggle();
        }
    }

    fn push(&mut self, child: Rc<RefCell<dyn ElementInternals>>) {
        push_child_to_element(self, child);
    }

    fn on_access_event(&mut self, event: AccessEvent) -> Result<(), IsshoError> {
        if let AccessEvent::Toggle = event {
            self.toggle();
        }
        Ok(())
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
            let box_node = inner_mut.box_layout.layout.gummy_node_id();
            gummy_tree.add_child(inner_mut.element_data.layout.gummy_node_id(), box_node);
            let owner: Rc<RefCell<dyn ElementInternals>> = inner.clone();
            gummy_tree.register_owner(box_node, inner_mut.element_data.internal_id, Rc::downgrade(&owner));
        });
        {
            inner_mut.element_data.set_accessibility_role(issho::Role::CheckBox);
            inner_mut.element_data.set_accessibility_name(label.to_string());
            inner_mut.element_data.set_accessibility_checked(checked);
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
        queue_event(EventKind::CheckboxToggled(CheckboxToggledEvent::new(
            target,
            self.label.clone(),
            self.checked,
        )));
        self.request_window_redraw();
    }
}
