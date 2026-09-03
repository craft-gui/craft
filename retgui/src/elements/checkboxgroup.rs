//! Stores one or more elements.

use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::{apply_generic_container_layout, draw_generic_container};
use crate::elements::traits::clone_element;
use crate::elements::{DynElement, Element, ElementNode, Elements, scrollable};
use crate::events::EventKind;
use crate::layout::GummyTree;
use crate::text::text_context::TextContext;
use retgui_renderer::renderer::Renderer;
use retgui_resource_manager::ResourceManager;

use std::sync::Arc;

#[derive(Clone, Copy)]
pub struct CheckboxGroup {
    pub(crate) inner: DynElement,
}

/// Stores one or more elements.
///
/// If overflow is set to scroll, it will become scrollable.
#[derive(Clone)]
pub(crate) struct CheckboxGroupNode {
    element_data: ElementData,
}

impl Element for CheckboxGroup {
    fn as_dyn_element(&self) -> DynElement {
        self.inner
    }
}

impl crate::elements::ElementNodeData for CheckboxGroupNode {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementNode for CheckboxGroupNode {
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
    }
}

impl CheckboxGroup {
    pub fn new(elements: &mut Elements, label: &str) -> Self {
        let inner = elements.insert_with(|me, access_tree| {
            Box::new(CheckboxGroupNode {
                element_data: ElementData::new(me, true, access_tree),
            })
        });
        elements.create_layout_node(inner, None);
        let inner_mut = elements.get_as_mut::<CheckboxGroupNode>(inner);
        {
            inner_mut.element_data.set_accessibility_role(issho::Role::Group);
            inner_mut.element_data.set_accessibility_name(label.to_string());
        }
        Self { inner }
    }
}
