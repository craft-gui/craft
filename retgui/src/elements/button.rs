//! Stores one or more elements.

use std::sync::Arc;

use issho::{AccessEvent, IsshoError, Role};
use winit::keyboard::KeyCode;

use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::{apply_generic_container_layout, draw_generic_container};
use crate::elements::traits::clone_element;
use crate::elements::{DynElement, Element, ElementInternals, Elements};
use crate::events::{ClickEvent, ClickTrigger, EventKind};
use crate::layout::GummyTree;
use crate::text::text_context::TextContext;
use retgui_renderer::renderer::Renderer;
use retgui_resource_manager::ResourceManager;

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
        if let EventKind::Click(_) = event {
            self.focus(elements);
        } else if self.is_focused()
            && let EventKind::KeyDown(keyboard_event) = event
            && !keyboard_event.repeat
            && matches!(
                keyboard_event.code,
                KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::Space
            )
        {
            let target = self.element_data.me;
            elements.queue_event(EventKind::Click(ClickEvent::new(
                target,
                ClickTrigger::Keyboard {
                    key: keyboard_event.key.clone(),
                },
            )));
        }
    }

    fn on_access_event(&mut self, elements: &mut Elements, event: AccessEvent) -> Result<(), IsshoError> {
        if let AccessEvent::Invoke = event {
            let target = self.element_data.me;
            elements.queue_event(EventKind::Click(ClickEvent::new(target, ClickTrigger::Accessibility)));
        }
        Ok(())
    }
}

impl Button {
    pub fn new(elements: &mut Elements) -> Self {
        let inner = elements.insert_with(|me, access_tree| {
            Box::new(ButtonElement {
                element_data: ElementData::new(me, true, access_tree),
            })
        });
        elements.create_layout_node(inner, None);
        let inner_mut = elements.get_as_mut::<ButtonElement>(inner);
        inner_mut.element_data.set_accessibility_role(Role::Button);
        Self { inner }
    }
}
