use crate::components::component::{ComponentId, ComponentSpecification};
use crate::elements::element::{Element, ElementBox};
use crate::elements::layout_context::{AvailableSpace, LayoutContext, TaffyTextContext, TaffyTextInputContext};
use crate::elements::ElementStyles;
use crate::reactive::element_state_store::{ElementStateStore, ElementStateStoreItem};
use crate::style::Style;
use crate::{generate_component_methods_private_push, RendererBox};
use parley::{FontContext, FontFamily, FontStack, Layout};
use std::any::Any;
use peniko::Brush;
use taffy::{NodeId, TaffyTree};

use crate::components::props::Props;
use crate::components::UpdateResult;
use crate::elements::common_element_data::CommonElementData;
use crate::elements::text::parley::{recompute_layout_from_cache_key, style_to_parley_style, TextHashKey, TextHashValue};
use crate::elements::text::TextState;
use crate::elements::text_input::editor::Editor;
use crate::events::OkuMessage;
use crate::geometry::Point;

// A stateful element that shows a text input.
#[derive(Clone, Default, Debug)]
pub struct TextInput {
    text: String,
    common_element_data: CommonElementData,
}

pub struct TextInputState {
    pub id: ComponentId,
    pub text: String,
    pub editor: Editor,
    pub children: Vec<ComponentSpecification>,
    pub style: Style,
}

impl TextInputState {
    pub(crate) fn new(
        id: ComponentId,
        text: &str,
        style: Style,
    ) -> Self {
        Self {
            id,
            text: String::new(),
            editor: Editor::new(text, style),
            children: Vec::new(),
            style: Default::default(),
        }
    }

    pub fn font_family(&self) -> Option<&str> {
        None
    }
}

impl TextInputState {

    /// Measure the width and height of the text given layout constraints.
    pub(crate) fn measure(
        &mut self,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        font_context: &mut FontContext,
        font_layout_context: &mut parley::LayoutContext<Brush>,
    ) -> taffy::Size<f32> {

        // Set width constraint
        let width_constraint = known_dimensions.width.or(match available_space.width {
            taffy::AvailableSpace::MinContent => Some(0.0),
            taffy::AvailableSpace::MaxContent => None,
            taffy::AvailableSpace::Definite(width) => Some(width),
        });

        let height_constraint = known_dimensions.height;

        let available_space_width_u32: AvailableSpace = match available_space.width {
            taffy::AvailableSpace::MinContent => AvailableSpace::MinContent,
            taffy::AvailableSpace::MaxContent => AvailableSpace::MaxContent,
            taffy::AvailableSpace::Definite(width) => AvailableSpace::Definite(width.to_bits()),
        };
        let available_space_height_u32: AvailableSpace = match available_space.height {
            taffy::AvailableSpace::MinContent => AvailableSpace::MinContent,
            taffy::AvailableSpace::MaxContent => AvailableSpace::MaxContent,
            taffy::AvailableSpace::Definite(height) => AvailableSpace::Definite(height.to_bits()),
        };

        self.editor.editor.set_width(width_constraint);
        self.editor.editor.update_layout(font_context, font_layout_context);
        let width = self.editor.editor.width.unwrap_or(0.0);
        let height = self.editor.editor.width.unwrap_or(0.0);

        taffy::Size {
            width: width,
            height: height,
        }

    }
}

impl TextInput {
    pub fn new(text: &str) -> TextInput {
        TextInput {
            text: String::from(text),
            common_element_data: Default::default(),
        }
    }

    #[allow(dead_code)]
    fn get_state<'a>(&self, element_state: &'a ElementStateStore) -> &'a TextInputState {
        element_state.storage.get(&self.common_element_data.component_id).unwrap().data.as_ref().downcast_ref().unwrap()
    }

    fn get_state_mut<'a>(&self, element_state: &'a mut ElementStateStore) -> &'a mut TextInputState {
        element_state.storage.get_mut(&self.common_element_data.component_id).unwrap().data.as_mut().downcast_mut().unwrap()
    }
}

impl Element for TextInput {
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
        "Text Input"
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
        let computed_layer_rectangle_transformed =
            self.common_element_data.computed_layered_rectangle_transformed;
        let content_rectangle = computed_layer_rectangle_transformed.content_rectangle();

        self.draw_borders(renderer);

        renderer.draw_text(
            self.common_element_data.component_id,
            content_rectangle,
            self.common_element_data.style.color(),
        );
    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        font_context: &mut FontContext,
        element_state: &mut ElementStateStore,
        scale_factor: f64,
    ) -> Option<NodeId> {
        let style: taffy::Style = self.common_element_data.style.to_taffy_style_with_scale_factor(scale_factor);

        self.common_element_data_mut().taffy_node_id = Some(taffy_tree
            .new_leaf_with_context(
                style,
                LayoutContext::TextInput(TaffyTextInputContext::new(self.common_element_data.component_id)),
            )
            .unwrap());

        self.common_element_data().taffy_node_id
    }

    fn finalize_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        position: Point,
        z_index: &mut u32,
        transform: glam::Mat4,
        font_context: &mut FontContext,
        element_state: &mut ElementStateStore,
        _pointer: Option<Point>,
    ) {
        let state = self.get_state_mut(element_state);

        let result = taffy_tree.layout(root_node).unwrap();
        self.resolve_layer_rectangle(position, transform, result, z_index);
        self.finalize_borders();
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn initialize_state(&self, font_context: &mut FontContext) -> ElementStateStoreItem {
        let mut state = TextInputState::new(
            self.common_element_data.component_id,
            &self.text,
            *self.style()
        );

        self.update_state_fragments(&mut state);
        
        ElementStateStoreItem {
            base: Default::default(),
            data: Box::new(state)
        }
    }

    fn update_state(&self, font_context: &mut FontContext, element_state: &mut ElementStateStore, reload_fonts: bool) {
        let state = self.get_state_mut(element_state);
        self.update_state_fragments(state);
    }

    fn on_event(&self, message: OkuMessage, element_state: &mut ElementStateStore, font_context: &mut FontContext) -> UpdateResult {
        let state = self.get_state_mut(element_state);
        
        let text_y: f32 = self.common_element_data().computed_layered_rectangle_transformed.content_rectangle().y;
        state.editor.handle_event(message, text_y);

        UpdateResult::default()
    }
}

impl TextInput {
    generate_component_methods_private_push!();

    fn update_state_fragments(&self, state: &mut TextInputState) {
        state.id = self.common_element_data.component_id;
        state.text = self.text.clone();
        state.children = self.common_element_data.child_specs.clone();
        state.style = *self.style();
    }
}

impl ElementStyles for TextInput {
    fn styles_mut(&mut self) -> &mut Style {
        self.common_element_data.current_style_mut()
    }
}
