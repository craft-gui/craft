use crate::components::component::ComponentSpecification;
use crate::components::Props;
use crate::elements::element::Element;
use crate::elements::element_data::ElementData;
use crate::elements::element_styles::ElementStyles;
use crate::layout::layout_context::LayoutContext;
use crate::geometry::{Point, Rectangle};
use crate::reactive::element_state_store::ElementStateStore;
use crate::renderer::renderer::RenderList;
use crate::renderer::RenderCommand;
use crate::style::Style;
use crate::Color;
use crate::generate_component_methods_no_children;
use std::any::Any;
use std::sync::Arc;
use taffy::{NodeId, TaffyTree};
use winit::window::Window;
use crate::text::text_context::TextContext;

#[derive(Clone, Default)]
pub struct Canvas {
    pub element_data: ElementData,
    pub render_list: Vec<RenderCommand>,
}

#[derive(Clone, Copy, Default)]
pub struct CanvasState {}

impl Element for Canvas {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }

    fn name(&self) -> &'static str {
        "Canvas"
    }

    fn draw(
        &mut self,
        renderer: &mut RenderList,
        _text_context: &mut TextContext,
        _taffy_tree: &mut TaffyTree<LayoutContext>,
        _root_node: NodeId,
        element_state: &mut ElementStateStore,
        _pointer: Option<Point>,
        _window: Option<Arc<dyn Window>>,
    ) {
        if !self.element_data.style.visible() {
            return;
        }
        let _border_color: Color = self.style().border_color().top;
        let computed_box_transformed = self.element_data.layout_item.computed_box_transformed;
        let _border_rectangle = computed_box_transformed.border_rectangle();
        let _content_rectangle = computed_box_transformed.content_rectangle();

        // background
        let computed_x_transformed = self.element_data.layout_item.computed_box_transformed.position.x;
        let computed_y_transformed = self.element_data.layout_item.computed_box_transformed.position.y;

        let computed_width = self.element_data.layout_item.computed_box_transformed.size.width;
        let computed_height = self.element_data.layout_item.computed_box_transformed.size.height;

        let border_top = self.element_data.layout_item.computed_box_transformed.border.top;
        let border_right = self.element_data.layout_item.computed_box_transformed.border.right;
        let border_bottom = self.element_data.layout_item.computed_box_transformed.border.bottom;
        let border_left = self.element_data.layout_item.computed_box_transformed.border.left;

        self.draw_borders(renderer, element_state);

        renderer.push_layer(Rectangle::new(
            computed_x_transformed + border_left,
            computed_y_transformed + border_top,
            computed_width - (border_right + border_left),
            computed_height - (border_top + border_bottom),
        ));

        for render_command in self.render_list.iter() {
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
                RenderCommand::DrawText(text_renderer, rectangle, text_scroll, show_cursor,) => {
                    let translated_rectangle = Rectangle::new(
                        rectangle.x + computed_x_transformed,
                        rectangle.y + computed_y_transformed,
                        rectangle.width,
                        rectangle.height,
                    );
                    renderer.draw_text(text_renderer.clone(), translated_rectangle, *text_scroll, *show_cursor);
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
                RenderCommand::FillBezPath(path, brush) => {
                    renderer.fill_bez_path(path.clone(), brush.clone());
                }
                RenderCommand::DrawTinyVg(rectangle, resource_identifier, color) => {
                    renderer.draw_tiny_vg(*rectangle, resource_identifier.clone(), *color);
                }
                RenderCommand::StartOverlay => {
                    renderer.start_overlay();
                }
                RenderCommand::EndOverlay => {
                    renderer.end_overlay();
                }    
            }
        }

        renderer.pop_layer();
    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        element_state: &mut ElementStateStore,
        scale_factor: f64,
    ) -> Option<NodeId> {
        self.merge_default_style();
        
        for child in self.element_data.children.iter_mut() {
            let child_node = child.internal.compute_layout(taffy_tree, element_state, scale_factor);
            self.element_data.layout_item.push_child(&child_node);
        }

        self.element_data.style.scale(scale_factor);
        let style: taffy::Style = self.element_data.style.to_taffy_style();

        self.element_data.layout_item.build_tree(taffy_tree, style)
    }

    fn finalize_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        position: Point,
        z_index: &mut u32,
        transform: glam::Mat4,
        element_state: &mut ElementStateStore,
        pointer: Option<Point>,
        text_context: &mut TextContext,
        clip_bounds: Option<Rectangle>,
    ) {
        let result = taffy_tree.layout(root_node).unwrap();
        self.resolve_box(position, transform, result, z_index);
        self.resolve_clip(clip_bounds);
        self.finalize_borders(element_state);

        for child in self.element_data.children.iter_mut() {
            let taffy_child_node_id = child.internal.taffy_node_id();
            if taffy_child_node_id.is_none() {
                continue;
            }

            child.internal.finalize_layout(
                taffy_tree,
                taffy_child_node_id.unwrap(),
                self.element_data.layout_item.computed_box.position,
                z_index,
                transform,
                element_state,
                pointer,
                text_context,
                None,
            );
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Canvas {
    #[allow(dead_code)]
    fn get_state<'a>(&self, element_state: &'a ElementStateStore) -> &'a CanvasState {
        element_state.storage.get(&self.element_data.component_id).unwrap().data.as_ref().downcast_ref().unwrap()
    }

    #[allow(dead_code)]
    fn get_state_mut<'a>(&self, element_state: &'a mut ElementStateStore) -> &'a mut CanvasState {
        element_state.storage.get_mut(&self.element_data.component_id).unwrap().data.as_mut().downcast_mut().unwrap()
    }

    pub fn new() -> Canvas {
        Canvas {
            element_data: Default::default(),
            render_list: Vec::new(),
        }
    }

    generate_component_methods_no_children!();
}

impl ElementStyles for Canvas {
    fn styles_mut(&mut self) -> &mut Style {
        self.element_data.current_style_mut()
    }
}
