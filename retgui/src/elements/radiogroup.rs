//! Stores one or more elements.

use std::any::Any;
use std::sync::Arc;

use issho::{SelectionData, SelectionGroup};

use retgui_renderer::renderer::Renderer;

use retgui_resource_manager::ResourceManager;

use winit::keyboard::KeyCode;

use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::{apply_generic_container_layout, draw_generic_container};
use crate::elements::traits::clone_element;
use crate::elements::{DynElement, Element, ElementNode, Elements, RadioNode, scrollable};
use crate::events::{Event, EventKind};
use crate::layout::GummyTree;
use crate::text::text_context::TextContext;

#[derive(Clone, Copy)]
pub struct RadioGroup {
    pub(crate) inner: DynElement,
}

/// Stores one or more elements.
///
/// If overflow is set to scroll, it will become scrollable.
#[derive(Clone)]
pub(crate) struct RadioGroupNode {
    element_data: ElementData,
}

impl Element for RadioGroup {
    fn as_dyn_element(&self) -> DynElement {
        self.inner
    }
}

impl crate::elements::ElementNodeData for RadioGroupNode {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementNode for RadioGroupNode {
    fn deep_clone(&self, elements: &mut Elements) -> DynElement {
        DynElement::new(clone_element::<Self, _>(self, elements, |_, _| None))
    }

    fn apply_layout(
        &mut self,
        gummy_tree: &mut GummyTree,
        z_index: &mut u32,
        _text_context: &mut TextContext,
        scale_factor: f64,
    ) {
        apply_generic_container_layout(self, gummy_tree, z_index, scale_factor);
    }

    fn draw(
        &self,
        elements: &Elements,
        renderer: &mut dyn Renderer,
        resource_manager: Arc<ResourceManager>,
        scale_factor: f64,
        text_context: &mut TextContext,
    ) {
        draw_generic_container(self, elements, renderer, resource_manager, text_context, scale_factor);
    }

    fn on_event(&mut self, elements: &mut Elements, event: &mut EventKind, _text_context: &mut TextContext) {
        scrollable::handle_scroll_logic(elements, self, event);

        if let EventKind::KeyDown(keyboard_event) = event {
            let direction = match keyboard_event.code {
                KeyCode::ArrowDown | KeyCode::ArrowRight => Some(1),
                KeyCode::ArrowUp | KeyCode::ArrowLeft => Some(-1),
                _ => None,
            };

            if direction.is_some_and(|direction| self.move_selection(elements, direction)) {
                keyboard_event.stop_propagation();
                keyboard_event.prevent_default();
            }
        }
    }
}

impl RadioGroupNode {
    fn move_selection(&mut self, elements: &mut Elements, direction: isize) -> bool {
        let radios = self
            .element_data
            .children
            .iter()
            .filter(|child| (elements.get(**child) as &dyn Any).is::<RadioNode>())
            .copied()
            .collect::<Vec<_>>();
        let Some(current_index) = radios.iter().position(|radio| elements.get(*radio).is_focused()) else {
            return false;
        };

        let next_index = if direction < 0 {
            (current_index + radios.len() - 1) % radios.len()
        } else {
            (current_index + 1) % radios.len()
        };
        {
            let next_handle = radios[next_index];
            elements.dispatch_mut(next_handle, |next, elements| {
                let next = (next as &mut dyn Any).downcast_mut::<RadioNode>().unwrap();
                next.focus(elements);
                next.set_value_from_group(elements);
            });
        }
        for radio in radios {
            let state = elements.get_as::<RadioNode>(radio).active_value;
            let selected = elements.state(state).clone();
            elements
                .get_as_mut::<RadioNode>(radio)
                .set_accessibility_selection(&selected);
        }
        true
    }
}

impl RadioGroup {
    pub fn new(elements: &mut Elements, label: &str) -> Self {
        let inner = elements.insert_with(|me, access_tree| {
            Box::new(RadioGroupNode {
                element_data: ElementData::new(me, true, access_tree),
            })
        });
        elements.create_layout_node(inner, None);
        let inner_mut = elements.get_as_mut::<RadioGroupNode>(inner);
        {
            inner_mut.element_data.set_accessibility_role(issho::Role::Group);
            inner_mut.element_data.set_accessibility_name(label.to_string());
            inner_mut
                .element_data
                .set_accessibility_selection_data(Some(SelectionData::SelectionGroup(SelectionGroup {
                    is_mandatory: true,
                    multiple_selectable: false,
                })));
        }
        Self { inner }
    }
}
