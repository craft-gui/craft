use crate::components::component::ComponentSpecification;
use crate::components::props::Props;
use crate::elements::common_element_data::CommonElementData;
use crate::elements::element::Element;
use crate::elements::element_styles::ElementStyles;
use crate::elements::layout_context::LayoutContext;
use crate::geometry::{Point, Rectangle};
use crate::reactive::element_state_store::ElementStateStore;
use crate::renderer::color::Color;
use crate::renderer::renderer::RenderCommand;
use crate::style::Style;
use crate::{generate_component_methods_no_children, RendererBox};
use cosmic_text::FontSystem;
use std::any::Any;
use taffy::{NodeId, TaffyTree};

#[derive(Clone, Default, Debug)]
pub struct Canvas {
    pub common_element_data: CommonElementData,
    pub render_commands: Vec<RenderCommand>,
}

#[derive(Clone, Copy, Default)]
pub struct CanvasState {
}

impl Element for Canvas {
    fn common_element_data(&self) -> &CommonElementData {
        &self.common_element_data
    }

    fn common_element_data_mut(&mut self) -> &mut CommonElementData {
        &mut self.common_element_data
    }

    fn name(&self) -> &'static str {
        "Canvas"
    }

    fn draw(
        &mut self,
        renderer: &mut RendererBox,
        _font_system: &mut FontSystem,
        _taffy_tree: &mut TaffyTree<LayoutContext>,
        _root_node: NodeId,
        _element_state: &ElementStateStore,
        _pointer: Option<Point>,
    ) {
        let _border_color: Color = self.style().border_color()[0];
        let computed_layer_rectangle_transformed = self.common_element_data.computed_layered_rectangle_transformed.clone();
        let _border_rectangle = computed_layer_rectangle_transformed.border_rectangle();
        let _content_rectangle = computed_layer_rectangle_transformed.content_rectangle();
        
        // background
        let computed_x_transformed = self.common_element_data.computed_layered_rectangle_transformed.position.x;
        let computed_y_transformed = self.common_element_data.computed_layered_rectangle_transformed.position.y;

        let computed_width = self.common_element_data.computed_layered_rectangle_transformed.size.width;
        let computed_height = self.common_element_data.computed_layered_rectangle_transformed.size.height;
        
        let border_top = self.common_element_data.computed_layered_rectangle_transformed.border.top;
        let border_right = self.common_element_data.computed_layered_rectangle_transformed.border.right;
        let border_bottom = self.common_element_data.computed_layered_rectangle_transformed.border.bottom;
        let border_left = self.common_element_data.computed_layered_rectangle_transformed.border.left;

        self.draw_borders(renderer);

        renderer.push_layer(Rectangle::new(
            computed_x_transformed + border_left,
            computed_y_transformed + border_top,
            computed_width - (border_right + border_left),
            computed_height - (border_top + border_bottom),
        ));

        for render_command in self.render_commands.iter() {
            match render_command {
                RenderCommand::DrawRect(rectangle, color) => {
                    let translated_rectangle = Rectangle::new(
                        rectangle.x + computed_x_transformed,
                        rectangle.y + computed_y_transformed,
                        rectangle.width,
                        rectangle.height,
                    );
                    renderer.draw_rect(translated_rectangle, *color);
                }
                RenderCommand::DrawRectOutline(rectangle, color) => {
                    let translated_rectangle = Rectangle::new(
                        rectangle.x + computed_x_transformed,
                        rectangle.y + computed_y_transformed,
                        rectangle.width,
                        rectangle.height,
                    );
                    renderer.draw_rect_outline(translated_rectangle, *color);
                }
                RenderCommand::DrawImage(rectangle, resource_identifier) => {
                    let translated_rectangle = Rectangle::new(
                        rectangle.x + computed_x_transformed,
                        rectangle.y + computed_y_transformed,
                        rectangle.width,
                        rectangle.height,
                    );
                    renderer.draw_image(translated_rectangle, resource_identifier.clone());
                }
                RenderCommand::DrawText(rectangle, component_id, color) => {
                    let translated_rectangle = Rectangle::new(
                        rectangle.x + computed_x_transformed,
                        rectangle.y + computed_y_transformed,
                        rectangle.width,
                        rectangle.height,
                    );
                    renderer.draw_text(*component_id, translated_rectangle, *color);
                }
                RenderCommand::PushLayer(rectangle) => {
                    let translated_rectangle = Rectangle::new(
                        rectangle.x + computed_x_transformed,
                        rectangle.y + computed_y_transformed,
                        rectangle.width,
                        rectangle.height,
                    );
                    renderer.push_layer(translated_rectangle);
                }
                RenderCommand::PopLayer => {
                    renderer.pop_layer();
                }
                RenderCommand::FillBezPath(path, color) => {
                    renderer.fill_bez_path(path.clone(), *color);
                }
            }
        }

        renderer.pop_layer();
    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        font_system: &mut FontSystem,
        element_state: &mut ElementStateStore,
        scale_factor: f64,
    ) -> Option<NodeId> {
        let mut child_nodes: Vec<NodeId> = Vec::with_capacity(self.children().len());

        for child in self.common_element_data.children.iter_mut() {
            let child_node = child.internal.compute_layout(taffy_tree, font_system, element_state, scale_factor);
            if let Some(child_node) = child_node {
                child_nodes.push(child_node);
            }
        }

        let style: taffy::Style = self.common_element_data.style.to_taffy_style_with_scale_factor(scale_factor);

        self.common_element_data_mut().taffy_node_id = Some(taffy_tree.new_with_children(style, &child_nodes).unwrap());
        self.common_element_data().taffy_node_id
    }

    fn finalize_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        position: Point,
        z_index: &mut u32,
        transform: glam::Mat4,
        font_system: &mut FontSystem,
        element_state: &mut ElementStateStore,
        pointer: Option<Point>,
    ) {
        let result = taffy_tree.layout(root_node).unwrap();
        self.resolve_layer_rectangle(position, transform, result, z_index);
        self.finalize_borders();
        
        for child in self.common_element_data.children.iter_mut() {
            let taffy_child_node_id = child.internal.taffy_node_id();
            if taffy_child_node_id.is_none() {
                continue;
            }

            child.internal.finalize_layout(
                taffy_tree,
                taffy_child_node_id.unwrap(),
                self.common_element_data.computed_layered_rectangle.position,
                z_index,
                transform,
                font_system,
                element_state,
                pointer,
            );
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Canvas {
    #[allow(dead_code)]
    fn get_state<'a>(&self, element_state: &'a ElementStateStore) -> &'a &CanvasState {
        element_state.storage.get(&self.common_element_data.component_id).unwrap().data.as_ref().downcast_ref().unwrap()
    }

    #[allow(dead_code)]
    fn get_state_mut<'a>(&self, element_state: &'a mut ElementStateStore) -> &'a mut CanvasState {
        element_state.storage.get_mut(&self.common_element_data.component_id).unwrap().data.as_mut().downcast_mut().unwrap()
    }

    pub fn new() -> Canvas {
        Canvas {
            common_element_data: Default::default(),
            render_commands: Vec::new(),
        }
    }

    generate_component_methods_no_children!();
}

impl ElementStyles for Canvas {
    fn styles_mut(&mut self) -> &mut Style {
        self.common_element_data.current_style_mut()
    }
}
