use crate::components::component::ComponentSpecification;
use crate::components::props::Props;
use crate::elements::common_element_data::CommonElementData;
use crate::elements::element::Element;
use crate::elements::layout_context::{ImageContext, LayoutContext};
use crate::elements::ElementStyles;
use crate::geometry::Point;
use crate::reactive::element_state_store::ElementStateStore;
use crate::resource_manager::ResourceIdentifier;
use crate::style::Style;
use crate::{generate_component_methods_no_children, RendererBox};
use parley::FontContext;
use std::any::Any;
use taffy::{NodeId, TaffyTree};

#[derive(Clone, Debug)]
pub struct Image {
    pub(crate) resource_identifier: ResourceIdentifier,
    pub common_element_data: CommonElementData,
}

impl Image {
    pub fn new(resource_identifier: ResourceIdentifier) -> Image {
        Image {
            resource_identifier,
            common_element_data: Default::default(),
        }
    }

    pub fn name() -> &'static str {
        "Image"
    }
}

impl Element for Image {
    fn common_element_data(&self) -> &CommonElementData {
        &self.common_element_data
    }

    fn common_element_data_mut(&mut self) -> &mut CommonElementData {
        &mut self.common_element_data
    }

    fn name(&self) -> &'static str {
        "Image"
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
        
        renderer.draw_image(
            content_rectangle,
            self.resource_identifier.clone(),
        );

        self.draw_borders(renderer);
    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        _font_context: &mut FontContext,
        _element_state: &mut ElementStateStore,
        scale_factor: f64,
    ) -> Option<NodeId> {
        let style: taffy::Style = self.common_element_data.style.to_taffy_style_with_scale_factor(scale_factor);

        self.common_element_data_mut().taffy_node_id = Some(taffy_tree
            .new_leaf_with_context(
                style,
                LayoutContext::Image(ImageContext {
                    resource_identifier: self.resource_identifier.clone(),
                }),
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
        _font_context: &mut FontContext,
        _element_state: &mut ElementStateStore,
        _pointer: Option<Point>,
    ) {
        let result = taffy_tree.layout(root_node).unwrap();
        self.resolve_layer_rectangle(position, transform, result, z_index);
        
        self.finalize_borders();
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Image {
    generate_component_methods_no_children!();
}

impl ElementStyles for Image {
    fn styles_mut(&mut self) -> &mut Style {
        self.common_element_data.current_style_mut()
    }
}