//! A basic code editor.

use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::Arc;

use retgui_primitives::brush::Brush;
use retgui_renderer::renderer::Renderer;
use retgui_resource_manager::ResourceManager;

use crate::elements::codeeditor::highlighter::compute_code_editor_style;
use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::{apply_generic_container_layout, draw_generic_container, push_child_to_element};
use crate::elements::traits::clone_element;
use crate::elements::{AsElement, DynElement, Element, ElementInternals, TextInput};
use crate::events::EventKind;
use crate::layout::GummyTree;
use crate::text::text_context::TextContext;

#[derive(Clone)]
pub struct CodeEditor {
    pub(crate) inner: Rc<RefCell<CodeEditorInner>>,
}

pub mod highlighter;

/// Stores one or more elements.
///
/// If overflow is set to scroll, it will become scrollable.
#[derive(Clone)]
pub(crate) struct CodeEditorInner {
    element_data: ElementData,
    extension: String,
    theme: String,
    text_input: TextInput,
    // TODO: Retain syntax_set and theme set.
}

impl Default for CodeEditor {
    fn default() -> Self {
        Self::new("", "rs", "base16-ocean.dark")
    }
}

impl Element for CodeEditor {}

impl Drop for CodeEditorInner {
    fn drop(&mut self) {
        ElementInternals::drop(self)
    }
}

impl AsElement for CodeEditor {
    fn with<R>(&self, callback: impl FnOnce(&dyn ElementInternals) -> R) -> R {
        callback(&*self.inner.borrow())
    }

    fn with_mut<R>(&self, callback: impl FnOnce(&mut dyn ElementInternals) -> R) -> R {
        callback(&mut *self.inner.borrow_mut())
    }
}

impl crate::elements::ElementData for CodeEditorInner {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementInternals for CodeEditorInner {
    fn deep_clone(&self) -> DynElement {
        DynElement::new(clone_element::<Self, _>(self, |_, _| None))
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
        &mut self,
        renderer: &mut dyn Renderer,
        resource_manager: Arc<ResourceManager>,
        scale_factor: f64,
        text_context: &mut TextContext,
    ) {
        draw_generic_container(self, renderer, resource_manager, text_context, scale_factor);
    }

    fn on_event(&mut self, event: &mut EventKind, _text_context: &mut TextContext) {
        if let EventKind::TextInputChanged(_) = event {
            self.highlight();
        }
    }

    fn push(&mut self, child: DynElement) {
        push_child_to_element(self, child.inner);
    }
}

impl CodeEditor {
    pub fn new(code: &str, extension: &str, theme: &str) -> Self {
        println!("Extension: {}", extension);
        let text_input = TextInput::new(code);
        let inner = Rc::new_cyclic(|me: &Weak<RefCell<CodeEditorInner>>| {
            RefCell::new(CodeEditorInner {
                element_data: ElementData::new(me.clone(), true),
                extension: extension.to_string(),
                theme: theme.to_string(),
                text_input: text_input.clone(),
            })
        });
        let mut inner_mut = inner.borrow_mut();
        inner_mut.element_data.create_layout_node(None);
        inner_mut.push(DynElement::new(text_input.inner));
        inner_mut.highlight();
        drop(inner_mut);
        Self { inner }
    }
}

impl CodeEditorInner {
    fn highlight(&mut self) {
        let mut text = self.text_input.inner.borrow_mut();
        let code_editor = compute_code_editor_style(text.get_text(), None, None, &self.extension, &self.theme);
        text.set_ranged_styles(code_editor.ranged_styles);
        text.set_background_brush(Brush::Color(code_editor.background_color));
        text.set_text_brush(Brush::Color(code_editor.foreground_color));
    }
}
