//! Stores one or more elements.

use std::collections::VecDeque;
use std::sync::Arc;

use issho::{AccessEvent, IsshoError, Role};

use retgui_renderer::renderer::Renderer;

use retgui_resource_manager::ResourceManager;

use winit::keyboard::KeyCode;

use crate::App;
use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::{apply_generic_container_layout, draw_generic_container};
use crate::elements::traits::clone_element;
use crate::elements::{DynElement, Element, ElementIds, ElementInternals, ElementStates, RetGuiAccessTree, RetainedElements};
use crate::events::{ClickEvent, ClickTrigger, EventKind};
use crate::layout::GummyTree;
use crate::text::text_context::TextContext;

#[derive(Clone, Copy)]
pub struct Button {
    pub(crate) inner: DynElement,
}

/// Stores one or more elements.
///
/// If overflow is set to scroll, it will become scrollable.
#[derive(Clone)]
pub(crate) struct ButtonElement {
    element_data: ElementData,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ButtonState {
    Default,
    Hovered,
    Pressed,
    Focused,
}

impl Element for Button {
    fn as_dyn_element(&self) -> DynElement {
        self.inner
    }
}

impl crate::elements::HasElementData for ButtonElement {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementInternals for ButtonElement {
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
        if let EventKind::Click(_) = event {
            self.focus(elements, event_queue, focus, focus_outline_visible);
        } else if self.is_focused()
            && let EventKind::KeyDown(keyboard_event) = event
            && !keyboard_event.repeat
            && matches!(
                keyboard_event.code,
                KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::Space
            )
        {
            let target = self.element_data.me;
            event_queue.push_back(EventKind::Click(ClickEvent::new(
                target,
                ClickTrigger::Keyboard {
                    key: keyboard_event.key.clone(),
                },
            )));
        }
    }

    fn on_access_event(
        &mut self,
        _elements: &mut RetainedElements,
        event_queue: &mut VecDeque<EventKind>,
        _states: &mut ElementStates,
        event: AccessEvent,
    ) -> Result<(), IsshoError> {
        if let AccessEvent::Invoke = event {
            let target = self.element_data.me;
            event_queue.push_back(EventKind::Click(ClickEvent::new(target, ClickTrigger::Accessibility)));
        }
        Ok(())
    }
}

impl Button {
    pub fn new(app: &mut App) -> Self {
        Self {
            inner: ButtonElement::insert(
                &mut app.elements,
                &mut app.gummy_tree,
                &app.access_tree,
                &mut app.by_internal_id,
            ),
        }
    }
}

impl ButtonElement {
    pub(crate) fn insert(
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        access_tree: &RetGuiAccessTree,
        by_internal_id: &mut ElementIds,
    ) -> DynElement {
        let inner = elements.insert_with(access_tree, by_internal_id, |me, access_tree| {
            Box::new(ButtonElement {
                element_data: ElementData::new(me, true, access_tree),
            })
        });
        let inner_mut = elements.get_as_mut::<ButtonElement>(inner);
        inner_mut.element_data.create_layout_node(gummy_tree, None);
        inner_mut.element_data.set_accessibility_role(Role::Button);
        inner
    }
}
