//! Stores one or more elements.

use std::any::Any;
use std::collections::VecDeque;
use std::sync::Arc;

use issho::{SelectionData, SelectionGroup};

use retgui_renderer::renderer::Renderer;

use retgui_resource_manager::ResourceManager;

use winit::keyboard::KeyCode;

use crate::App;
use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::{apply_generic_container_layout, draw_generic_container};
use crate::elements::traits::clone_element;
use crate::elements::{DynElement, Element, ElementIds, ElementInternals, ElementStates, RadioElement, RetGuiAccessTree, RetainedElements, scrollable};
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
pub(crate) struct RadioGroupElement {
    element_data: ElementData,
}

impl Element for RadioGroup {
    fn as_dyn_element(&self) -> DynElement {
        self.inner
    }
}

impl crate::elements::HasElementData for RadioGroupElement {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementInternals for RadioGroupElement {
    fn deep_clone(
        &self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        access_tree: &RetGuiAccessTree,
        by_internal_id: &mut ElementIds,
    ) -> DynElement {
        DynElement::new(clone_element::<Self, _>(
            self,
            elements,
            gummy_tree,
            access_tree,
            by_internal_id,
            |_, _| None,
        ))
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
        elements: &RetainedElements,
        states: &ElementStates,
        renderer: &mut dyn Renderer,
        resource_manager: Arc<ResourceManager>,
        scale_factor: f64,
        text_context: &mut TextContext,
    ) {
        draw_generic_container(
            self,
            elements,
            states,
            renderer,
            resource_manager,
            text_context,
            scale_factor,
        );
    }

    fn on_event(
        &mut self,
        elements: &mut RetainedElements,
        _gummy_tree: &mut GummyTree,
        _access_tree: &RetGuiAccessTree,
        _by_internal_id: &mut ElementIds,
        event_queue: &mut VecDeque<EventKind>,
        focus: &mut Option<DynElement>,
        focus_outline_visible: bool,
        _pending_animation_updates: &mut Vec<(DynElement, bool)>,
        states: &mut ElementStates,
        event: &mut EventKind,
        _text_context: &mut TextContext,
    ) {
        scrollable::handle_scroll_logic(elements, event_queue, focus, focus_outline_visible, self, event);

        if let EventKind::KeyDown(keyboard_event) = event {
            let direction = match keyboard_event.code {
                KeyCode::ArrowDown | KeyCode::ArrowRight => Some(1),
                KeyCode::ArrowUp | KeyCode::ArrowLeft => Some(-1),
                _ => None,
            };

            if direction.is_some_and(|direction| {
                self.move_selection(elements, event_queue, focus, focus_outline_visible, states, direction)
            }) {
                keyboard_event.stop_propagation();
                keyboard_event.prevent_default();
            }
        }
    }
}

impl RadioGroupElement {
    fn move_selection(
        &mut self,
        elements: &mut RetainedElements,
        event_queue: &mut VecDeque<EventKind>,
        focus: &mut Option<DynElement>,
        focus_outline_visible: bool,
        states: &mut ElementStates,
        direction: isize,
    ) -> bool {
        let radios = self
            .element_data
            .children
            .iter()
            .filter(|child| (elements.get(**child) as &dyn Any).is::<RadioElement>())
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
                let next = (next as &mut dyn Any).downcast_mut::<RadioElement>().unwrap();
                next.focus(elements, event_queue, focus, focus_outline_visible);
                next.set_value_from_group(elements.store_id(), event_queue, states);
            });
        }
        for radio in radios {
            let state = elements.get_as::<RadioElement>(radio).active_value;
            let selected = state.read_from(states, elements.store_id()).clone();
            elements
                .get_as_mut::<RadioElement>(radio)
                .set_accessibility_selection(&selected);
        }
        true
    }

    pub(crate) fn insert(
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        access_tree: &RetGuiAccessTree,
        by_internal_id: &mut ElementIds,
        label: &str,
    ) -> DynElement {
        let inner = elements.insert_with(access_tree, by_internal_id, |me, access_tree| {
            Box::new(RadioGroupElement {
                element_data: ElementData::new(me, true, access_tree),
            })
        });
        let inner_mut = elements.get_as_mut::<RadioGroupElement>(inner);
        inner_mut.element_data.create_layout_node(gummy_tree, None);
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
        inner
    }
}

impl RadioGroup {
    pub fn new(app: &mut App, label: &str) -> Self {
        Self {
            inner: RadioGroupElement::insert(
                &mut app.elements,
                &mut app.gummy_tree,
                &app.access_tree,
                &mut app.by_internal_id,
                label,
            ),
        }
    }
}
