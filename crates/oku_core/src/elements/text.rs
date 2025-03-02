use crate::components::component::{ComponentId, ComponentOrElement, ComponentSpecification};
use crate::elements::element::{Element, ElementBox};
use crate::elements::layout_context::{AvailableSpace, LayoutContext, TaffyTextContext};
use crate::elements::{ElementStyles, Span};
use crate::reactive::element_state_store::{ElementStateStore, ElementStateStoreItem};
use crate::style::Style;
use crate::{generate_component_methods_private_push, RendererBox};
use parley::{Alignment, AlignmentOptions, FontContext, FontStack, Layout, TextStyle, TreeBuilder};
use peniko::Brush;
use std::any::Any;
use taffy::{NodeId, TaffyTree};

use crate::components::props::Props;
use crate::elements::common_element_data::CommonElementData;
use crate::geometry::Point;

#[derive(Clone, Debug)]
pub enum TextFragment {
    String(String),
    Span(u32),
    InlineComponentSpecification(u32),
}

// A stateful element that shows text.
#[derive(Clone, Default, Debug)]
pub struct Text {
    fragments: Vec<TextFragment>,
    common_element_data: CommonElementData,
}

#[derive(Copy, Clone)]
pub struct TextHashValue {
    pub computed_width: f32,
    pub computed_height: f32,
}

pub struct TextState {
    pub id: ComponentId,
    pub fragments: Vec<TextFragment>,
    pub children: Vec<ComponentSpecification>,
    pub style: Style,
    pub layout: Layout<Brush>,
}

impl TextState {
    pub(crate) fn new(
        id: ComponentId,
    ) -> Self {
        Self {
            id,
            fragments: Vec::new(),
            children: Vec::new(),
            style: Default::default(),
            layout: Layout::default()
        }
    }

    pub fn font_family(&self) -> Option<&str> {
        None
    }

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

        fn style_to_parley_style<'a>(style: &Style) -> TextStyle<'a, Brush> {
            let text_brush = Brush::Solid(style.color());
            let font_stack = FontStack::from("system-ui");
            TextStyle {
                brush: text_brush,
                font_stack,
                line_height: 1.3,
                font_size: style.font_size(),
                ..Default::default()
            }
        }

        let root_style = style_to_parley_style(&self.style);
        let mut builder: TreeBuilder<Brush> = font_layout_context.tree_builder(font_context, 1.0, &root_style);

        for fragment in self.fragments.iter() {
            match fragment {
                TextFragment::String(str) => {
                    builder.push_text(str);
                }
                TextFragment::Span(span_index) => {
                    let span = self.children.get(*span_index as usize).unwrap();

                    match &span.component {
                        ComponentOrElement::Element(ele) => {
                            let ele = &*ele.internal;

                            if let Some(span) = ele.as_any().downcast_ref::<Span>() {
                                builder.push_style_span(style_to_parley_style(span.style()));
                                builder.push_text(&span.text);
                                builder.pop_style_span();
                            }
                        }
                        _ => {}
                    }
                }
                TextFragment::InlineComponentSpecification(inline) => {}
            }
        }


        // Build the builder into a Layout
        let (mut layout, _text): (Layout<Brush>, String) = builder.build();
        layout.break_all_lines(width_constraint);
        layout.align(width_constraint, Alignment::Start, AlignmentOptions::default());

        let width = layout.width().ceil() as u32;
        let height = layout.height().ceil() as u32;

        self.layout = layout;

        taffy::Size {
            width: width as f32,
            height: height as f32,
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
        element_state.storage.get_mut(&self.common_element_data.component_id).unwrap().data.as_mut().downcast_mut().unwrap()
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
        let computed_layer_rectangle_transformed =
            self.common_element_data.computed_layered_rectangle_transformed.clone();
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
                LayoutContext::Text(TaffyTextContext::new(self.common_element_data.component_id)),
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

        let result = taffy_tree.layout(root_node).unwrap();
        self.resolve_layer_rectangle(position, transform, result, z_index);
        
        self.finalize_borders();
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn initialize_state(&self, font_context: &mut FontContext) -> ElementStateStoreItem {
        let mut state = TextState::new(
            self.common_element_data.component_id,
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
}

impl Text {
    generate_component_methods_private_push!();

    fn update_state_fragments(&self, state: &mut TextState) {
        state.id = self.common_element_data.component_id;
        state.fragments = self.fragments.clone();
        state.children = self.common_element_data.child_specs.clone();
        state.style = self.style().clone();
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
        self.fragments.push(TextFragment::InlineComponentSpecification(self.common_element_data().child_specs.len() as u32 - 1));
        self
    }
}

impl ElementStyles for Text {
    fn styles_mut(&mut self) -> &mut Style {
        self.common_element_data.current_style_mut()
    }
}
