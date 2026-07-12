//! A basic code editor.

use std::any::Any;
use std::cell::{Ref, RefCell, RefMut};
use std::rc::{Rc, Weak};
use std::sync::Arc;

use craft_primitives::geometry::{Affine, Point, Rectangle};
use craft_renderer::renderer::Renderer;
use craft_resource_manager::ResourceManager;

use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::{apply_generic_container_layout, draw_generic_container, push_child_to_element};
use crate::elements::traits::DeepClone;
use crate::elements::{AsElement, Element, ElementInternals, resolve_clip_for_scrollable, TextInput};
use crate::elements::codeeditor::highlighter::compute_code_editor_style;
use crate::events::{Event, EventKind};
use crate::layout::TaffyTree;
use crate::style::Overflow;
use crate::text::text_context::TextContext;

#[derive(Clone)]
pub struct CodeEditor {
    pub inner: Rc<RefCell<CodeEditorInner>>,
}

pub mod highlighter;

/// Stores one or more elements.
///
/// If overflow is set to scroll, it will become scrollable.
#[derive(Clone)]
pub struct CodeEditorInner {
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
    fn as_element_rc(&self) -> Rc<RefCell<dyn ElementInternals>> {
        self.inner.clone()
    }

    fn borrow(&self) -> Ref<'_, dyn ElementInternals> {
        self.inner.borrow()
    }

    fn borrow_mut(&self) -> RefMut<'_, dyn ElementInternals> {
        self.inner.borrow_mut()
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
    fn deep_clone(&self) -> Rc<RefCell<dyn ElementInternals>> {
        self.deep_clone_internal()
    }

    fn apply_layout(
        &mut self,
        taffy_tree: &mut TaffyTree,
        position: Point,
        z_index: &mut u32,
        transform: Affine,
        text_context: &mut TextContext,
        clip_bounds: Option<Rectangle>,
        scale_factor: f64,
    ) {
        apply_generic_container_layout(
            self,
            taffy_tree,
            position,
            z_index,
            transform,
            text_context,
            clip_bounds,
            scale_factor,
        );
    }

    fn draw(&mut self, renderer: &mut dyn Renderer, resource_manager: Arc<ResourceManager>, scale_factor: f64, text_context: &mut TextContext) {
        draw_generic_container(self, renderer, resource_manager, text_context, scale_factor);
    }

    fn on_event(
        &mut self,
        message: &EventKind,
        _text_context: &mut TextContext,
        _event: &mut Event,
        _target: Option<Rc<RefCell<dyn ElementInternals>>>,
    ) {
        if let EventKind::TextInputChanged(_) = message {
            self.highlight();
        }
    }

    fn apply_clip(&mut self, clip_bounds: Option<Rectangle>) {
        let overflow = self.style().get_overflow();
        if overflow[0] == Overflow::Scroll || overflow[1] == Overflow::Scroll {
            resolve_clip_for_scrollable(self, clip_bounds);
        } else {
            self.element_data.layout.apply_clip(clip_bounds);
        }
    }

    fn push(&mut self, child: Rc<RefCell<dyn ElementInternals>>) {
        push_child_to_element(self, child);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
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
        inner_mut.push(text_input.inner);
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
        text.set_background_color(code_editor.background_color);
        text.set_color(code_editor.foreground_color);
    }

}