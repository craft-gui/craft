use crate::components::component::ComponentSpecification;
use crate::components::Props;
use crate::elements::element::Element;
use crate::elements::element_data::ElementData;
use crate::layout::layout_context::{LayoutContext, TinyVgContext};
use crate::elements::ElementStyles;
use crate::geometry::Point;
use crate::reactive::element_state_store::ElementStateStore;
use crate::renderer::renderer::RenderList;
use crate::resource_manager::ResourceIdentifier;
use crate::style::Style;
use crate::generate_component_methods_no_children;
use std::any::Any;
use std::sync::Arc;
use peniko::Color;
use taffy::{NodeId, TaffyTree};
use winit::window::Window;
use crate::text::text_context::TextContext;

#[derive(Clone, Debug)]
pub struct TinyVg {
    pub(crate) resource_identifier: ResourceIdentifier,
    pub element_data: ElementData,
}

impl TinyVg {
    pub fn new(resource_identifier: ResourceIdentifier) -> TinyVg {
        TinyVg {
            resource_identifier,
            element_data: Default::default(),
        }
    }

    pub fn name() -> &'static str {
        "TinyVG"
    }
}

impl Element for TinyVg {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }

    fn name(&self) -> &'static str {
        "TinyVG"
    }

    fn draw(
        &mut self,
        renderer: &mut RenderList,
        text_context: &mut TextContext,
        _taffy_tree: &mut TaffyTree<LayoutContext>,
        _root_node: NodeId,
        _element_state: &mut ElementStateStore,
        _pointer: Option<Point>,
        _window: Option<Arc<dyn Window>>,
    ) {
        if !self.element_data.style.visible() {
            return;
        }
        let computed_box_transformed = self.element_data.computed_box_transformed;
        let content_rectangle = computed_box_transformed.content_rectangle();
        self.draw_borders(renderer);
        
        
        let mut color = None;
        if self.style().color() != Color::TRANSPARENT {
            color = Some(self.style().color());
        }
        renderer.draw_tiny_vg(content_rectangle, self.resource_identifier.clone(), color);
    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        _element_state: &mut ElementStateStore,
        scale_factor: f64,
    ) -> Option<NodeId> {
        self.merge_default_style();
        let style: taffy::Style = self.element_data.style.to_taffy_style_with_scale_factor(scale_factor);

        self.element_data_mut().taffy_node_id = Some(
            taffy_tree
                .new_leaf_with_context(
                    style,
                    LayoutContext::TinyVg(TinyVgContext {
                        resource_identifier: self.resource_identifier.clone(),
                    }),
                )
                .unwrap(),
        );

        self.element_data().taffy_node_id
    }

    fn finalize_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        position: Point,
        z_index: &mut u32,
        transform: glam::Mat4,
        _element_state: &mut ElementStateStore,
        _pointer: Option<Point>,
        text_context: &mut TextContext,
    ) {
        let result = taffy_tree.layout(root_node).unwrap();
        self.resolve_box(position, transform, result, z_index);

        self.finalize_borders();
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn default_style(&self) -> Style {
        let mut style = Style::default();
        *style.color_mut() = Color::TRANSPARENT;
        
        style
    }
}

impl TinyVg {
    generate_component_methods_no_children!();
}

impl ElementStyles for TinyVg {
    fn styles_mut(&mut self) -> &mut Style {
        self.element_data.current_style_mut()
    }
}
