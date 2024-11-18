use crate::components::component::ComponentSpecification;
use crate::components::UpdateResult;
use crate::elements::element::{CommonElementData, Element};
use crate::elements::layout_context::LayoutContext;
use crate::engine::renderer::color::Color;
use crate::engine::renderer::renderer::Rectangle;
use crate::reactive::state_store::StateStore;
use crate::style::{Style};
use crate::RendererBox;
use cosmic_text::FontSystem;
use std::any::Any;
use taffy::{NodeId, TaffyTree};
use winit::event::MouseScrollDelta;
use crate::elements::element_styles::ElementStyles;
use crate::engine::events::{OkuMessage};

/// A stateless element that stores other elements.
#[derive(Clone, Default, Debug)]
pub struct Container {
    pub common_element_data: CommonElementData,
}

pub struct ContainerState {
    pub(crate) scroll_y: f32,
}

impl Element for Container {
    fn common_element_data(&self) -> &CommonElementData {
        &self.common_element_data
    }

    fn common_element_data_mut(&mut self) -> &mut CommonElementData {
        &mut self.common_element_data
    }

    fn name(&self) -> &'static str {
        "Container"
    }

    fn draw(
        &mut self,
        renderer: &mut RendererBox,
        font_system: &mut FontSystem,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        element_state: &StateStore,
    ) {
        let scroll_y = if let Some(container_state) =
            element_state.storage.get(&self.common_element_data.component_id).unwrap().downcast_ref::<ContainerState>()
        {
            container_state.scroll_y
        } else {
            0.0
        };

        let border_color: Color = self.style().border_color;

        // background
        let computed_x_transformed = self.common_element_data.computed_x_transformed;
        let computed_y_transformed = self.common_element_data.computed_y_transformed;

        let computed_width = self.common_element_data.computed_width;
        let computed_height = self.common_element_data.computed_height;

        let computed_content_width = self.common_element_data.computed_content_width;
        let computed_content_height = self.common_element_data.computed_content_height;

        let border_top = self.common_element_data.computed_border[0];
        let border_right = self.common_element_data.computed_border[1];
        let border_bottom = self.common_element_data.computed_border[2];
        let border_left = self.common_element_data.computed_border[3];

        let scrolltrack_width = self.common_element_data.scrollbar_size[0];
        let scrolltrack_height = computed_height - border_top - border_bottom;

        let client_height = computed_height - border_top - border_bottom;
        let scroll_height = computed_content_height - border_top;

        let max_scroll_y = scroll_height - client_height;

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
        for (index, child) in self.common_element_data.children.iter_mut().enumerate() {
            let child2 = taffy_tree.child_at_index(root_node, index).unwrap();
            child.internal.draw(renderer, font_system, taffy_tree, child2, element_state);
        }
        renderer.pop_layer();

        // scrollbar
        let scroll_track_color = Color::rgba(100, 100, 100, 255);
        let visible_y = (computed_height - border_top - border_bottom) / max_scroll_y;
        let scrollthumb_height = scrolltrack_height * visible_y;
        let remaining_height = scrolltrack_height - scrollthumb_height;
        let scrollthumb_offset = scroll_y / self.common_element_data.computed_scrollbar_height * remaining_height;

        // track
        renderer.draw_rect(
            Rectangle::new(
                computed_x_transformed + computed_width - scrolltrack_width - border_right,
                computed_y_transformed + border_top,
                scrolltrack_width,
                scrolltrack_height,
            ),
            scroll_track_color,
        );

        let scrollthumb_color = Color::rgba(150, 150, 150, 255);
        let scrollthumb_width = scrolltrack_width;
        // thumb
        renderer.draw_rect(
            Rectangle::new(
                computed_x_transformed + computed_width - scrolltrack_width - border_right,
                computed_y_transformed + border_top + scrollthumb_offset,
                scrollthumb_width,
                scrollthumb_height,
            ),
            scrollthumb_color,
        );
    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        font_system: &mut FontSystem,
        element_state: &mut StateStore,
    ) -> NodeId {
        let mut child_nodes: Vec<NodeId> = Vec::with_capacity(self.children().len());

        for child in self.common_element_data.children.iter_mut() {
            let child_node = child.internal.compute_layout(taffy_tree, font_system, element_state);
            child_nodes.push(child_node);
        }

        let style: taffy::Style = self.common_element_data.style.into();

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

        let scroll_y = if let Some(container_state) =
            element_state.storage.get(&self.common_element_data.component_id).unwrap().downcast_ref::<ContainerState>()
        {
            container_state.scroll_y
        } else {
            0.0
        };

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

    fn on_event(&self, message: OkuMessage, element_state: &mut StateStore, _font_system: &mut FontSystem) -> UpdateResult {
        let container_state = self.get_state_mut(element_state);

        if self.style().overflow[1].is_scroll_container() {
            match message {
                OkuMessage::MouseWheelEvent(mouse_wheel) => {
                    let delta = match mouse_wheel.delta {
                        MouseScrollDelta::LineDelta(_x, y) => y,
                        MouseScrollDelta::PixelDelta(y) => y.y as f32,
                    };
                    let delta = -delta * self.common_element_data.style.font_size.max(12.0) * 1.2;

                    //let max_scroll_y = self.common_element_data.computed_scrollbar_height;

                    let client_height = self.common_element_data.computed_height - self.common_element_data.computed_border[0] - self.common_element_data.computed_border[2];
                    let scroll_height = self.common_element_data.computed_content_height - self.common_element_data.computed_border[0];

                    let max_scroll_y = scroll_height - client_height;

                    container_state.scroll_y = (container_state.scroll_y + delta).clamp(0.0, max_scroll_y);

                    UpdateResult::new().prevent_propagate().prevent_defaults()
                }
                _ => UpdateResult::new(),
            }
        } else {
            UpdateResult::new()
        }
    }
}

impl Container {
    #[allow(dead_code)]
    fn get_state<'a>(&self, element_state: &'a StateStore) -> &'a &ContainerState {
        element_state.storage.get(&self.common_element_data.component_id).unwrap().as_ref().downcast_ref().unwrap()
    }

    fn get_state_mut<'a>(&self, element_state: &'a mut StateStore) -> &'a mut ContainerState {
        element_state.storage.get_mut(&self.common_element_data.component_id).unwrap().as_mut().downcast_mut().unwrap()
    }

    pub fn new() -> Container {
        Container {
            common_element_data: Default::default(),
        }
    }

    pub fn id(mut self, id: &str) -> Self {
        self.common_element_data.id = Some(id.to_string());
        self
    }

    pub fn component(self) -> ComponentSpecification {
        ComponentSpecification::new(self.into())
    }
}

impl ElementStyles for Container {
    fn styles_mut(&mut self) -> &mut Style {
        &mut self.common_element_data.style
    }
}