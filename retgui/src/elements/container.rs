//! Stores one or more elements.

use std::collections::VecDeque;
use std::sync::Arc;

use retgui_renderer::renderer::Renderer;

use retgui_resource_manager::ResourceManager;

use crate::App;
use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::{apply_generic_container_layout, draw_generic_container};
use crate::elements::traits::clone_element;
use crate::elements::{DynElement, Element, ElementIds, ElementInternals, ElementStates, RetGuiAccessTree, RetainedElements, scrollable};
use crate::events::EventKind;
use crate::layout::GummyTree;
use crate::text::text_context::TextContext;

#[derive(Clone, Copy)]
pub struct Container {
    pub(crate) inner: DynElement,
}

/// Stores one or more elements.
///
/// If overflow is set to scroll, it will become scrollable.
#[derive(Clone)]
pub(crate) struct ContainerElement {
    element_data: ElementData,
}

impl Element for Container {
    fn as_dyn_element(&self) -> DynElement {
        self.inner
    }
}

impl crate::elements::HasElementData for ContainerElement {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementInternals for ContainerElement {
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
        _states: &mut ElementStates,
        event: &mut EventKind,
        _text_context: &mut TextContext,
    ) {
        scrollable::handle_scroll_logic(elements, event_queue, focus, focus_outline_visible, self, event);
    }
}

impl Container {
    pub fn new(app: &mut App) -> Self {
        Self {
            inner: ContainerElement::create(
                &mut app.elements,
                &mut app.gummy_tree,
                &app.access_tree,
                &mut app.by_internal_id,
            ),
        }
    }
}

impl ContainerElement {
    pub(crate) fn create(
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        access_tree: &RetGuiAccessTree,
        by_internal_id: &mut ElementIds,
    ) -> DynElement {
        let inner = elements.insert_with(access_tree, by_internal_id, |me, access_tree| {
            Box::new(ContainerElement {
                element_data: ElementData::new(me, true, access_tree),
            })
        });
        elements
            .get_mut(inner)
            .element_data_mut()
            .create_layout_node(gummy_tree, None);
        inner
    }
}
