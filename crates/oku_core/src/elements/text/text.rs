use crate::components::component::{ComponentId, ComponentSpecification};
use crate::elements::element::{Element, ElementBox};
use crate::elements::layout_context::{LayoutContext, TaffyTextContext};
use crate::elements::{ElementStyles, Span};
use crate::reactive::element_state_store::{ElementStateStore, ElementStateStoreItem};
use crate::style::Style;
use crate::{generate_component_methods_private_push, RendererBox};
use parley::{FontContext, Layout};
use peniko::Brush;
use std::any::Any;
use std::collections::HashMap;
use taffy::{NodeId, TaffyTree};

use crate::components::Props;
use crate::elements::common_element_data::CommonElementData;
use crate::elements::text::parley::{recompute_layout_from_cache_key, TextHashKey, TextHashValue};
use crate::geometry::Point;

#[derive(Clone, Debug)]
pub enum TextFragment {
    String(String),
    Span(u32),
    InlineComponentSpecification(u32),
}

/// An element for displaying text.
///
/// Text may consist of strings, spans, or inline elements.
#[derive(Clone, Default, Debug)]
pub struct Text {
    fragments: Vec<TextFragment>,
    common_element_data: CommonElementData,
}

pub struct TextState {
    pub id: ComponentId,
    pub fragments: Vec<TextFragment>,
    pub children: Vec<ComponentSpecification>,
    pub style: Style,
    pub layout: Layout<Brush>,
    pub cached_text_layout: HashMap<TextHashKey, TextHashValue>,
    pub last_cache_key: Option<TextHashKey>,
    /// We need to update the text layout in finalize_layout if the constraints or available space have changed since the last layout pass
    /// AND the last layout operation was a text size cache hit.
    ///
    /// This may be true because the cached text size that we retrieve does not map to the current layout which is computed during the last cache miss.
    pub should_recompute_final_text_layout: bool,
    pub reload_fonts: bool,
}

impl TextState {
    pub(crate) fn new(id: ComponentId) -> Self {
        Self {
            id,
            fragments: Vec::new(),
            children: Vec::new(),
            style: Default::default(),
            layout: Layout::default(),
            cached_text_layout: Default::default(),
            last_cache_key: None,
            should_recompute_final_text_layout: false,
            reload_fonts: false,
        }
    }
}

impl Text {
    pub fn new(text: &str) -> Text {
        Text {
            fragments: vec![TextFragment::String(text.to_string())],
            common_element_data: Default::default(),
        }
    }

    #[allow(dead_code)]
    fn get_state<'a>(&self, element_state: &'a ElementStateStore) -> &'a TextState {
        element_state.storage.get(&self.common_element_data.component_id).unwrap().data.as_ref().downcast_ref().unwrap()
    }

    fn get_state_mut<'a>(&self, element_state: &'a mut ElementStateStore) -> &'a mut TextState {
        element_state
            .storage
            .get_mut(&self.common_element_data.component_id)
            .unwrap()
            .data
            .as_mut()
            .downcast_mut()
            .unwrap()
    }
}

impl Element for Text {
    fn common_element_data(&self) -> &CommonElementData {
        &self.common_element_data
    }

    fn common_element_data_mut(&mut self) -> &mut CommonElementData {
        &mut self.common_element_data
    }

    fn children_mut(&mut self) -> &mut Vec<ElementBox> {
        &mut self.common_element_data.children
    }

    fn name(&self) -> &'static str {
        "Text"
    }

    fn draw(
        &mut self,
        renderer: &mut RendererBox,
        _font_context: &mut FontContext,
        _taffy_tree: &mut TaffyTree<LayoutContext>,
        _root_node: NodeId,
        _element_state: &ElementStateStore,
        _pointer: Option<Point>,
    ) {
        let computed_layer_rectangle_transformed = self.common_element_data.computed_layered_rectangle_transformed;
        let content_rectangle = computed_layer_rectangle_transformed.content_rectangle();

        self.draw_borders(renderer);

        renderer.draw_text(self.common_element_data.component_id, content_rectangle);
    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        _element_state: &mut ElementStateStore,
        scale_factor: f64,
    ) -> Option<NodeId> {
        self.merge_default_style();
        let style: taffy::Style = self.common_element_data.style.to_taffy_style_with_scale_factor(scale_factor);

        self.common_element_data_mut().taffy_node_id = Some(
            taffy_tree
                .new_leaf_with_context(
                    style,
                    LayoutContext::Text(TaffyTextContext::new(self.common_element_data.component_id)),
                )
                .unwrap(),
        );

        self.common_element_data().taffy_node_id
    }

    fn finalize_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        position: Point,
        z_index: &mut u32,
        transform: glam::Mat4,
        element_state: &mut ElementStateStore,
        _pointer: Option<Point>,
    ) {
        let state = self.get_state_mut(element_state);

        // We may need to recompute the final text layout, read the documentation for should_recompute_final_text_layout to find out more.
        if state.should_recompute_final_text_layout {
            if let Some(last_cache_key) = &state.last_cache_key {
                recompute_layout_from_cache_key(&mut state.layout, last_cache_key);
            }
        }

        let result = taffy_tree.layout(root_node).unwrap();
        self.resolve_layer_rectangle(position, transform, result, z_index);
        self.finalize_borders();

        state.reload_fonts = false;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn initialize_state(&self) -> ElementStateStoreItem {
        let mut state = TextState::new(self.common_element_data.component_id);

        self.update_state_fragments(&mut state);

        ElementStateStoreItem {
            base: Default::default(),
            data: Box::new(state),
        }
    }

    fn update_state(&self, element_state: &mut ElementStateStore, reload_fonts: bool) {
        let state = self.get_state_mut(element_state);
        self.update_state_fragments(state);
        state.reload_fonts = reload_fonts;
    }
}

impl Text {
    generate_component_methods_private_push!();

    fn update_state_fragments(&self, state: &mut TextState) {
        state.id = self.common_element_data.component_id;
        state.fragments = self.fragments.clone();
        state.children = self.common_element_data.child_specs.clone();
        state.style = *self.style();
    }

    pub fn push_text(mut self, text: String) -> Self {
        self.fragments.push(TextFragment::String(text));
        self
    }

    pub fn push_span(mut self, span: Span) -> Self {
        self = self.push(span);
        self.fragments.push(TextFragment::Span(self.common_element_data().child_specs.len() as u32 - 1));
        self
    }

    pub fn push_inline(mut self, inline_component: ComponentSpecification) -> Self {
        self = self.push(inline_component);
        self.fragments
            .push(TextFragment::InlineComponentSpecification(self.common_element_data().child_specs.len() as u32 - 1));
        self
    }
}

impl ElementStyles for Text {
    fn styles_mut(&mut self) -> &mut Style {
        self.common_element_data.current_style_mut()
    }
}
