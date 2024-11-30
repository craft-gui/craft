use crate::components::component::ComponentSpecification;
use crate::components::UpdateResult;
use crate::elements::element::{CommonElementData, Element};
use crate::elements::layout_context::LayoutContext;
use crate::engine::renderer::color::Color;
use crate::engine::renderer::renderer::{Rectangle, RenderCommand};
use crate::reactive::state_store::StateStore;
use crate::style::{Style};
use crate::{generate_component_methods_no_children, RendererBox};
use cosmic_text::FontSystem;
use std::any::Any;
use taffy::{NodeId, TaffyTree};
use winit::event::{ButtonSource, ElementState, MouseButton, MouseScrollDelta};
use crate::elements::element_styles::ElementStyles;
use crate::engine::events::{Message, OkuMessage};
use crate::engine::events::OkuMessage::PointerButtonEvent;
use crate::components::props::Props;

/// A stateless element that stores other elements.
#[derive(Clone, Default, Debug)]
pub struct Canvas {
    pub common_element_data: CommonElementData,
    pub render_commands: Vec<RenderCommand>,
}

#[derive(Clone, Copy, Default)]
pub struct CanvasState {
    pub(crate) scroll_y: f32,
    pub(crate) scroll_click: Option<(f32, f32)>
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
        font_system: &mut FontSystem,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        element_state: &StateStore,
    ) {
        let border_color: Color = self.style().border_color;

        // background
        let computed_x_transformed = self.common_element_data.computed_x_transformed;
        let computed_y_transformed = self.common_element_data.computed_y_transformed;

        let computed_width = self.common_element_data.computed_width;
        let computed_height = self.common_element_data.computed_height;
        
        let border_top = self.common_element_data.computed_border[0];
        let border_right = self.common_element_data.computed_border[1];
        let border_bottom = self.common_element_data.computed_border[2];
        let border_left = self.common_element_data.computed_border[3];
        
        // Background
        renderer.draw_rect(
            Rectangle::new(
                computed_x_transformed,
                computed_y_transformed,
                computed_width,
                computed_height,
            ),
            self.common_element_data.style.background,
        );

        // border top
        renderer.draw_rect(
            Rectangle::new(
                computed_x_transformed,
                computed_y_transformed,
                computed_width,
                border_top,
            ),
            border_color,
        );

        // border right
        renderer.draw_rect(
            Rectangle::new(
                computed_x_transformed + computed_width - border_right,
                computed_y_transformed + border_top,
                border_right,
                computed_height - border_top,
            ),
            border_color,
        );

        // border bottom
        renderer.draw_rect(
            Rectangle::new(
                computed_x_transformed + border_left,
                computed_y_transformed + computed_height - border_bottom,
                computed_width - (border_right + border_left),
                border_bottom,
            ),
            border_color,
        );

        // border left
        renderer.draw_rect(
            Rectangle::new(
                computed_x_transformed,
                computed_y_transformed + border_top,
                border_right,
                computed_height - border_top,
            ),
            border_color,
        );

        renderer.push_layer(Rectangle::new(
            computed_x_transformed + border_left,
            computed_y_transformed + border_top,
            computed_width - (border_right + border_left),
            computed_height - (border_top + border_bottom),
        ));

        for render_command in self.render_commands.iter() {
            match render_command {
                RenderCommand::DrawRect(mut rectangle, color) => {
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
            }
        }

        renderer.pop_layer();

        // scrollbar
        let scroll_track_color = Color::rgba(100, 100, 100, 255);
        
        // track
        renderer.draw_rect(
            self.common_element_data.computed_scroll_track,
            scroll_track_color,
        );

        let scrollthumb_color = Color::rgba(150, 150, 150, 255);
        
        // thumb
        renderer.draw_rect(
            self.common_element_data.computed_scroll_thumb,
            scrollthumb_color,
        );
    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        font_system: &mut FontSystem,
        element_state: &mut StateStore,
        scale_factor: f64,
    ) -> NodeId {
        let mut child_nodes: Vec<NodeId> = Vec::with_capacity(self.children().len());

        for child in self.common_element_data.children.iter_mut() {
            let child_node = child.internal.compute_layout(taffy_tree, font_system, element_state, scale_factor);
            child_nodes.push(child_node);
        }

        let style: taffy::Style = self.common_element_data.style.to_taffy_style_with_scale_factor(scale_factor);

        taffy_tree.new_with_children(style, &child_nodes).unwrap()
    }

    fn finalize_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        x: f32,
        y: f32,
        transform: glam::Mat4,
        font_system: &mut FontSystem,
        element_state: &mut StateStore,
    ) {
        let result = taffy_tree.layout(root_node).unwrap();

        self.common_element_data.computed_content_width = result.content_size.width;
        self.common_element_data.computed_content_height = result.content_size.height;
        self.common_element_data.scrollbar_size = [result.scrollbar_size.width, result.scrollbar_size.height];

        self.resolve_position(x, y, result);
        
        self.common_element_data.computed_width = result.size.width;
        self.common_element_data.computed_height = result.size.height;
        
        self.common_element_data.computed_scrollbar_width = result.scroll_width();
        self.common_element_data.computed_scrollbar_height = result.scroll_height();
        
        self.common_element_data.computed_padding =
            [result.padding.top, result.padding.right, result.padding.bottom, result.padding.left];

        self.common_element_data.computed_border =
            [result.border.top, result.border.right, result.border.bottom, result.border.left];

        let transformed_xy = transform.mul_vec4(glam::vec4(
            self.common_element_data.computed_x,
            self.common_element_data.computed_y,
            0.0,
            1.0,
        ));
        self.common_element_data.computed_x_transformed = transformed_xy.x;
        self.common_element_data.computed_y_transformed = transformed_xy.y;

        let scroll_y = if let Some(canvas_state) =
            element_state.storage.get(&self.common_element_data.component_id).unwrap().downcast_ref::<CanvasState>()
        {
            canvas_state.scroll_y
        } else {
            0.0
        };

        self.finalize_scrollbar(scroll_y);

        let child_transform = glam::Mat4::from_translation(glam::Vec3::new(0.0, -scroll_y, 0.0));

        for (index, child) in self.common_element_data.children.iter_mut().enumerate() {
            let child2 = taffy_tree.child_at_index(root_node, index).unwrap();
            child.internal.finalize_layout(
                taffy_tree,
                child2,
                self.common_element_data.computed_x,
                self.common_element_data.computed_y,
                child_transform * transform,
                font_system,
                element_state,
            );
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Canvas {
    #[allow(dead_code)]
    fn get_state<'a>(&self, element_state: &'a StateStore) -> &'a &CanvasState {
        element_state.storage.get(&self.common_element_data.component_id).unwrap().as_ref().downcast_ref().unwrap()
    }

    fn get_state_mut<'a>(&self, element_state: &'a mut StateStore) -> &'a mut CanvasState {
        element_state.storage.get_mut(&self.common_element_data.component_id).unwrap().as_mut().downcast_mut().unwrap()
    }

    pub fn new() -> Canvas {
        Canvas {
            common_element_data: Default::default(),
            render_commands: Vec::new(),
        }
    }

    pub fn id(mut self, id: &str) -> Self {
        self.common_element_data.id = Some(id.to_string());
        self
    }

    generate_component_methods_no_children!();
}

impl ElementStyles for Canvas {
    fn styles_mut(&mut self) -> &mut Style {
        &mut self.common_element_data.style
    }
}