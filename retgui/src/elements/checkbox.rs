//! A toggleable checkbox.

use std::sync::Arc;

use issho::{AccessEvent, IsshoError};

use peniko::kurbo;

use retgui_primitives::geometry::{Affine, Rectangle, TrblRectangle};

use retgui_renderer::Brush;
use retgui_renderer::renderer::Renderer;

use retgui_resource_manager::ResourceManager;

use winit::event::ElementState;
use winit::keyboard::KeyCode;

use crate::elements::element_data::ElementData;
use crate::elements::element_id::create_unique_element_id;
use crate::elements::internal_helpers::{apply_generic_container_layout, apply_generic_container_layout_non_dom};
use crate::elements::traits::clone_element;
use crate::elements::{DynElement, Element, ElementInternals, Elements, scrollable};
use crate::events::{CheckboxToggledEvent, EventKind};
use crate::layout::GummyTree;
use crate::style::Unit;
use crate::text::text_context::TextContext;
use crate::{auto, px, rgb};

#[derive(Clone, Copy)]
pub struct Checkbox {
    pub(crate) inner: DynElement,
}

#[derive(Clone)]
pub(crate) struct CheckboxElement {
    element_data: ElementData,
    box_layout: ElementData,
    box_rect: Rectangle,
    label: String,
    checked: bool,
}

impl Element for Checkbox {
    fn as_dyn_element(&self) -> DynElement {
        self.inner
    }
}

impl crate::elements::HasElementData for CheckboxElement {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementInternals for CheckboxElement {
    fn deep_clone(&self, elements: &mut Elements) -> DynElement {
        DynElement::new(clone_element::<Self, _>(self, elements, |element, gummy_tree| {
            let owner_id = element.element_data.internal_id;
            let owner = element.element_data.me;
            let parent = element.element_data.layout.gummy_node_id();
            let box_node = gummy_tree.clone_node(element.box_layout.layout.gummy_node_id());
            element.box_layout.layout.gummy_node_id = Some(box_node);
            element.box_layout.internal_id = create_unique_element_id();
            element.box_layout.me = owner;
            gummy_tree.add_child(parent, box_node);
            gummy_tree.register_owner(box_node, owner_id, owner);
            Some(parent)
        }))
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
        &self,
        elements: &Elements,
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

        self.draw_children(
            elements,
            renderer,
            resource_manager.clone(),
            _scale_factor,
            _text_context,
        );
        self.maybe_end_layer(renderer);
        self.draw_scrollbar(renderer, _scale_factor);

        self.maybe_end_overlay(renderer);
    }

    fn on_event(&mut self, elements: &mut Elements, event: &mut EventKind, _text_context: &mut TextContext) {
        scrollable::handle_scroll_logic(elements, self, event);
        if let EventKind::Click(_) = event {
            self.toggle(elements);
            self.focus(elements);
        } else if self.is_focused()
            && let EventKind::KeyDown(key) = event
            && key.code == KeyCode::Space
            && key.state == ElementState::Pressed
        {
            self.toggle(elements);
        }
    }

    fn on_access_event(&mut self, elements: &mut Elements, event: AccessEvent) -> Result<(), IsshoError> {
        if let AccessEvent::Toggle = event {
            self.toggle(elements);
        }
        Ok(())
    }
}

impl Checkbox {
    pub fn new(elements: &mut Elements, label: &str, checked: bool) -> Self {
        let size = 16.0;
        let inner = elements.insert_with(|me, access_tree| {
            Box::new(CheckboxElement {
                element_data: ElementData::new(me, true, access_tree.clone()),
                box_layout: ElementData::new_pseudo(me, false, access_tree),
                box_rect: Rectangle::new(0.0, 0.0, size, size),
                label: label.to_string(),
                checked,
            })
        });

        {
            let inner_mut = elements.get_as_mut::<CheckboxElement>(inner);
            inner_mut.box_layout.style.set_min_width(Unit::Px(size));
            inner_mut.box_layout.style.set_min_height(Unit::Px(size));
            inner_mut
                .box_layout
                .style
                .set_margin(TrblRectangle::new(auto(), px(5), auto(), px(0)));
            inner_mut.element_data.set_accessibility_role(issho::Role::CheckBox);
            inner_mut.element_data.set_accessibility_name(label.to_string());
            inner_mut.element_data.set_accessibility_checked(checked);
        }
        {
            let (gummy_tree, elements) = elements.disjoint_borrow_layout_and_elements();
            let inner_mut = elements.get_as_mut::<CheckboxElement>(inner);
            inner_mut.element_data.create_layout_node(gummy_tree, None);
            inner_mut.box_layout.create_layout_node(gummy_tree, None);
            let box_node = inner_mut.box_layout.layout.gummy_node_id();
            gummy_tree.add_child(inner_mut.element_data.layout.gummy_node_id(), box_node);
            gummy_tree.register_owner(box_node, inner_mut.element_data.internal_id, inner);
        }

        Self { inner }
    }
}

impl CheckboxElement {
    fn toggle(&mut self, elements: &mut Elements) {
        self.checked = !self.checked;
        self.element_data.set_accessibility_checked(self.checked);
        let target = self.element_data.me;
        elements.queue_event(EventKind::CheckboxToggled(CheckboxToggledEvent::new(
            target,
            self.label.clone(),
            self.checked,
        )));
        self.request_window_redraw();
    }
}
