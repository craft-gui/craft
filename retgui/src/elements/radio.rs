//! A selectable circle.

use std::any::Any;
use std::sync::Arc;

use issho::{AccessEvent, IsshoError, SelectionData, SelectionGroupItem};

use retgui_primitives::brush::Brush;
use retgui_primitives::geometry::{Affine, Circle, TrblRectangle};

use retgui_renderer::renderer::Renderer;

use retgui_resource_manager::ResourceManager;

use winit::keyboard::KeyCode;

use crate::elements::element_data::ElementData;
use crate::elements::element_id::create_unique_element_id;
use crate::elements::internal_helpers::{apply_generic_container_layout, apply_generic_container_layout_non_dom};
use crate::elements::traits::clone_element;
use crate::elements::{DynElement, Element, ElementNode, Elements, State, scrollable};
use crate::events::{Event, EventKind, RadioValueChangedEvent};
use crate::layout::GummyTree;
use crate::style::Unit;
use crate::text::text_context::TextContext;
use crate::{auto, px, rgb};

#[derive(Clone, Copy)]
pub struct Radio {
    pub(crate) inner: DynElement,
}

/// Stores one or more elements.
///
/// If overflow is set to scroll, it will become scrollable.
#[derive(Clone)]
pub(crate) struct RadioNode {
    element_data: ElementData,
    circle_layout: ElementData,
    circle: Circle,
    value: String,
    label: String,
    hide_radio: bool,
    pub(super) active_value: State<String>,
}

impl Element for Radio {
    fn as_dyn_element(&self) -> DynElement {
        self.inner
    }
}

impl crate::elements::ElementNodeData for RadioNode {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementNode for RadioNode {
    fn deep_clone(&self, elements: &mut Elements) -> DynElement {
        DynElement::new(clone_element::<Self, _>(self, elements, |element, gummy_tree| {
            let owner_id = element.element_data.internal_id;
            let owner = element.element_data.me;
            let parent = element.element_data.layout.gummy_node_id();
            let circle_node = gummy_tree.clone_node(element.circle_layout.layout.gummy_node_id());
            element.circle_layout.layout.gummy_node_id = Some(circle_node);
            element.circle_layout.internal_id = create_unique_element_id();
            element.circle_layout.me = owner;
            gummy_tree.add_child(parent, circle_node);
            gummy_tree.register_owner(circle_node, owner_id, owner);
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
        apply_generic_container_layout_non_dom(&mut self.circle_layout, gummy_tree, z_index, scale_factor);
        let circle_rect = self.circle_layout.layout.local_box_in_parent().content_rectangle();
        self.circle.x = circle_rect.x + self.circle.radius;
        self.circle.y = circle_rect.y + self.circle.radius;
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

        if !self.hide_radio {
            if self.is_selected(elements) {
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

        renderer.set_transform(container_transform);

        self.draw_children(elements, renderer, resource_manager, _scale_factor, _text_context);
        self.maybe_end_layer(renderer);
        self.draw_scrollbar(renderer, _scale_factor);

        self.maybe_end_overlay(renderer);
    }

    fn on_event(&mut self, elements: &mut Elements, event: &mut EventKind, _text_context: &mut TextContext) {
        scrollable::handle_scroll_logic(elements, self, event);
        if let EventKind::PointerUp(_) = event {
            self.focus(elements);
            self.set_value(elements);
        } else if self.is_focused()
            && let EventKind::KeyDown(keyboard_event) = event
            && keyboard_event.code == KeyCode::Space
            && !keyboard_event.repeat
        {
            self.set_value(elements);
            keyboard_event.stop_propagation();
            keyboard_event.prevent_default();
        }
    }

    fn on_access_event(&mut self, elements: &mut Elements, event: AccessEvent) -> Result<(), IsshoError> {
        if matches!(event, AccessEvent::Select | AccessEvent::AddToSelection) && !self.is_selected(elements) {
            self.set_value(elements);
        }
        Ok(())
    }
}

impl RadioNode {
    fn set_value(&mut self, elements: &mut Elements) {
        self.set_value_from_group(elements);

        let me = self.element_data.me;
        let parent = self.element_data.parent;
        if let Some(parent) = parent {
            for sibling in elements.get(parent).element_data().children.clone() {
                if me == sibling {
                    continue;
                }
                if (elements.get(sibling) as &dyn Any).is::<RadioNode>() {
                    let selected = elements.state(self.active_value).clone();
                    elements
                        .get_as_mut::<RadioNode>(sibling)
                        .set_accessibility_selection(&selected);
                }
            }
        }
    }

    pub(super) fn set_value_from_group(&mut self, elements: &mut Elements) {
        let selection_changed = !self.is_selected(elements);
        *elements.state_mut(self.active_value) = self.value.clone();
        let selected = elements.state(self.active_value).clone();
        self.set_accessibility_selection(&selected);
        let target = self.element_data.me;
        elements.queue_event(EventKind::RadioValueChanged(RadioValueChangedEvent::new(
            target, selected,
        )));
        if selection_changed {
            self.request_window_redraw();
        }
    }

    fn is_selected(&self, elements: &Elements) -> bool {
        elements.state(self.active_value).as_str() == self.value
    }

    pub(super) fn set_accessibility_selection(&mut self, selected: &str) {
        let is_selected = selected == self.value;
        self.element_data
            .set_accessibility_selection_data(Some(SelectionData::SelectionGroupItem(SelectionGroupItem {
                is_selected,
            })));
    }
}

impl Radio {
    pub fn new(elements: &mut Elements, value: &str, label: &str, active_value: State<String>) -> Self {
        let radius = 7.0;
        let inner = elements.insert_with(|me, access_tree| {
            Box::new(RadioNode {
                element_data: ElementData::new(me, true, access_tree.clone()),
                circle_layout: ElementData::new_pseudo(me, false, access_tree),
                circle: Circle::new(0.0, 0.0, radius),
                value: value.to_string(),
                label: label.to_string(),
                hide_radio: false,
                active_value,
            })
        });
        let selected = elements.state(active_value).clone();
        {
            let inner_mut = elements.get_as_mut::<RadioNode>(inner);
            inner_mut.circle_layout.style.set_min_width(Unit::Px(radius * 2.0));
            inner_mut.circle_layout.style.set_min_height(Unit::Px(radius * 2.0));
            inner_mut
                .circle_layout
                .style
                .set_margin(TrblRectangle::new(auto(), px(5), auto(), px(0)));
            inner_mut.element_data.set_accessibility_role(issho::Role::RadioButton);
            inner_mut.element_data.set_accessibility_name(label.to_string());
            inner_mut.set_accessibility_selection(&selected);
        }
        elements.with_gummy_tree(|gummy_tree, elements| {
            let inner_mut = elements.get_as_mut::<RadioNode>(inner);
            inner_mut.element_data.create_layout_node(gummy_tree, None);
            inner_mut.circle_layout.create_layout_node(gummy_tree, None);
            let node_id = inner_mut.circle_layout.layout.gummy_node_id();
            gummy_tree.add_child(inner_mut.element_data.layout.gummy_node_id(), node_id);
            gummy_tree.register_owner(node_id, inner_mut.element_data.internal_id, inner);
        });

        Self { inner }
    }

    /// Hide the default circle radio button.
    pub fn set_hide_radio(&mut self, elements: &mut Elements, value: bool) {
        // TODO: Hide in gummy.
        let inner = elements.get_as_mut::<RadioNode>(self.inner);
        inner.hide_radio = value;
        inner.request_window_redraw();
    }

    /// Hide the default circle radio button.
    pub fn hide_radio(mut self, elements: &mut Elements) -> Self {
        self.set_hide_radio(elements, true);
        self
    }

    pub fn get_label(&self, elements: &Elements) -> String {
        elements.get_as::<RadioNode>(self.inner).label.clone()
    }

    pub fn get_value(&self, elements: &Elements) -> String {
        elements.get_as::<RadioNode>(self.inner).value.clone()
    }
}
