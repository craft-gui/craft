use crate::components::component::ComponentSpecification;
use crate::components::Props;
use crate::elements::element::Element;
use crate::elements::element_data::ElementData;
use crate::elements::ElementStyles;
use crate::generate_component_methods_no_children;
use craft_primitives::geometry::{Point, Rectangle};
use crate::layout::layout_context::{ImageContext, LayoutContext};
use crate::reactive::element_state_store::ElementStateStore;
use craft_renderer::RenderList;
use craft_resource_manager::ResourceIdentifier;
use crate::style::Style;
use crate::text::text_context::TextContext;
use std::any::Any;
use std::sync::Arc;
use kurbo::Affine;
use taffy::{NodeId, TaffyTree};
use winit::window::Window;

#[derive(Clone)]
pub struct Image {
    pub(crate) resource_identifier: ResourceIdentifier,
    pub element_data: ElementData,
}

impl Image {
    pub fn new(resource_identifier: ResourceIdentifier) -> Image {
        Image {
            resource_identifier,
            element_data: Default::default(),
        }
    }

    pub fn name() -> &'static str {
        "Image"
    }
}

impl Element for Image {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }

    fn name(&self) -> &'static str {
        "Image"
    }

    fn draw(
        &mut self,
        renderer: &mut RenderList,
        _text_context: &mut TextContext,
        element_state: &mut ElementStateStore,
        _pointer: Option<Point>,
        _window: Option<Arc<Window>>,
        scale_factor: f64,
    ) {
        if !self.element_data.style.visible() {
            return;
        }
        let computed_box_transformed = self.computed_box_transformed();
        let content_rectangle = computed_box_transformed.content_rectangle();
        self.draw_borders(renderer, element_state, scale_factor);

        renderer.draw_image(content_rectangle.scale(scale_factor), self.resource_identifier.clone());
    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        _element_state: &mut ElementStateStore,
        _scale_factor: f64,
    ) -> Option<NodeId> {
        self.merge_default_style();
        let style: taffy::Style = self.element_data.style.to_taffy_style();

        self.element_data.layout_item.build_tree_with_context(
            taffy_tree,
            style,
            LayoutContext::Image(ImageContext {
                resource_identifier: self.resource_identifier.clone(),
            }),
        )
    }

    fn finalize_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        position: Point,
        z_index: &mut u32,
        transform: Affine,
        element_state: &mut ElementStateStore,
        _pointer: Option<Point>,
        _text_context: &mut TextContext,
        clip_bounds: Option<Rectangle>,
    ) {
        let result = taffy_tree.layout(root_node).unwrap();
        self.resolve_box(position, transform, result, z_index);
        self.resolve_clip(clip_bounds);

        self.finalize_borders(element_state);
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
        self.element_data.current_style_mut()
    }
}
