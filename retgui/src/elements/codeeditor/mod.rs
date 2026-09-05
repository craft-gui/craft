//! A basic code editor.

use std::sync::Arc;

use retgui_primitives::brush::Brush;

use retgui_renderer::renderer::Renderer;

use retgui_resource_manager::ResourceManager;

use crate::elements::codeeditor::highlighter::compute_code_editor_style;
use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::{apply_generic_container_layout, draw_generic_container};
use crate::elements::traits::clone_element;
use crate::elements::{DynElement, Element, ElementInternals, Elements, TextInput};
use crate::events::EventKind;
use crate::layout::GummyTree;
use crate::text::text_context::TextContext;

#[derive(Clone, Copy)]
pub struct CodeEditor {
    pub(crate) inner: DynElement,
}

pub mod highlighter;

/// Stores one or more elements.
///
/// If overflow is set to scroll, it will become scrollable.
#[derive(Clone)]
pub(crate) struct CodeEditorElement {
    element_data: ElementData,
    extension: String,
    theme: String,
    text_input: TextInput,
    // TODO: Retain syntax_set and theme set.
}

impl Element for CodeEditor {
    fn as_dyn_element(&self) -> DynElement {
        self.inner
    }
}

impl crate::elements::HasElementData for CodeEditorElement {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementInternals for CodeEditorElement {
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
        if let EventKind::TextInputChanged(_) = event {
            self.highlight(elements);
        }
    }
}

impl CodeEditor {
    pub fn new(elements: &mut Elements, code: &str, extension: &str, theme: &str) -> Self {
        let text_input = TextInput::new(elements, code);
        let inner = elements.insert_with(|me, access_tree| {
            Box::new(CodeEditorElement {
                element_data: ElementData::new(me, true, access_tree),
                extension: extension.to_string(),
                theme: theme.to_string(),
                text_input,
            })
        });
        elements.create_layout_node(inner, None);
        crate::elements::internal_helpers::push_child_to_element(elements, inner, text_input.inner);
        elements.dispatch_mut(inner, |element, elements| {
            (element as &mut dyn std::any::Any)
                .downcast_mut::<CodeEditorElement>()
                .expect("code editor handle changed type")
                .highlight(elements);
        });
        Self { inner }
    }
}

impl CodeEditorElement {
    fn highlight(&mut self, elements: &mut Elements) {
        let text = self.text_input.text(elements);
        let code_editor = compute_code_editor_style(&text, None, None, &self.extension, &self.theme);
        let (gummy_tree, elements) = elements.disjoint_borrow_layout_and_elements();
        let text = elements.get_as_mut::<crate::elements::TextInputElement>(self.text_input.inner);
        text.set_ranged_styles(gummy_tree, code_editor.ranged_styles);
        text.set_background_brush(Brush::Color(code_editor.background_color));
        text.set_text_brush(gummy_tree, Brush::Color(code_editor.foreground_color));
    }
}
