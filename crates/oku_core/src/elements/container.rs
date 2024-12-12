use crate::components::component::ComponentSpecification;
use crate::components::UpdateResult;
use crate::elements::element::{CommonElementData, Element};
use crate::elements::layout_context::LayoutContext;
use crate::renderer::color::Color;
use crate::renderer::renderer::Rectangle;
use crate::reactive::state_store::{StateStore, StateStoreItem};
use crate::style::{Style};
use crate::{generate_component_methods, RendererBox};
use crate::components::props::Props;
use cosmic_text::FontSystem;
use std::any::Any;
use taffy::{NodeId, Overflow, TaffyTree};
use winit::event::{ButtonSource, ElementState, MouseButton, MouseScrollDelta, PointerSource};
use crate::elements::element_styles::ElementStyles;
use crate::events::{Message, OkuMessage};
use crate::events::OkuMessage::PointerButtonEvent;

/// A stateless element that stores other elements.
#[derive(Clone, Default, Debug)]
pub struct Container {
    pub common_element_data: CommonElementData,
}

#[derive(Clone, Copy, Default)]
pub struct ContainerState {
    pub(crate) scroll_y: f32,
    pub(crate) scroll_click: Option<(f32, f32)>
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


        if self.common_element_data.style.overflow[1] == Overflow::Scroll {
            renderer.push_layer(Rectangle::new(
                computed_x_transformed + border_left,
                computed_y_transformed + border_top,
                computed_width - (border_right + border_left),
                computed_height - (border_top + border_bottom),
            ));
        }
        for (index, child) in self.common_element_data.children.iter_mut().enumerate() {
            let child2 = taffy_tree.child_at_index(root_node, index).unwrap();
            child.internal.draw(renderer, font_system, taffy_tree, child2, element_state);
        }
        
        if self.common_element_data.style.overflow[1] == Overflow::Scroll {
            renderer.pop_layer();
        }

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

        let scroll_y = if let Some(container_state) =
            element_state.storage.get(&self.common_element_data.component_id).unwrap().downcast_ref::<ContainerState>()
        {
            container_state.scroll_y
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

    fn on_event(&self, message: OkuMessage, element_state: &mut StateStore, _font_system: &mut FontSystem) -> UpdateResult {
        let container_state = self.get_state_mut(element_state);

        if self.style().overflow[1] == taffy::Overflow::Scroll {
            match message {
                OkuMessage::MouseWheelEvent(mouse_wheel) => {
                    let delta = match mouse_wheel.delta {
                        MouseScrollDelta::LineDelta(_x, y) => y,
                        MouseScrollDelta::PixelDelta(y) => y.y as f32,
                    };
                    let delta = -delta * self.common_element_data.style.font_size.max(12.0) * 1.2;
                    let max_scroll_y = self.common_element_data.max_scroll_y;

                    container_state.scroll_y = (container_state.scroll_y + delta).clamp(0.0, max_scroll_y);

                    UpdateResult::new().prevent_propagate().prevent_defaults()
                }
                OkuMessage::PointerButtonEvent(pointer_button) => {
                    if pointer_button.button.mouse_button() == MouseButton::Left {
                        
                        // DEVICE(TOUCH): Handle scrolling within the content area on touch based input devices.
                        if let ButtonSource::Touch { .. } = pointer_button.button {
                            let container_rectangle = Rectangle::new(
                                self.common_element_data.computed_x + self.common_element_data.computed_border[3],
                                self.common_element_data.computed_y + self.common_element_data.computed_border[0],
                                self.common_element_data.computed_width - self.common_element_data.computed_border[1],
                                self.common_element_data.computed_height - self.common_element_data.computed_border[2]
                            );
                            
                            let in_scroll_bar = self.common_element_data.computed_scroll_thumb.contains(pointer_button.position.x as f32, pointer_button.position.y as f32);

                            if container_rectangle.contains(pointer_button.position.x as f32, pointer_button.position.y as f32) && !in_scroll_bar {
                                container_state.scroll_click = Some((pointer_button.position.x as f32, pointer_button.position.y as f32));
                                return UpdateResult::new().prevent_propagate().prevent_defaults();
                            }
                        }

                        match pointer_button.state {
                            ElementState::Pressed => {
                                if self.common_element_data.computed_scroll_thumb.contains(pointer_button.position.x as f32, pointer_button.position.y as f32) {
                                    container_state.scroll_click = Some((pointer_button.position.x as f32, pointer_button.position.y as f32));
                                    UpdateResult::new().prevent_propagate().prevent_defaults()
                                } else if self.common_element_data.computed_scroll_track.contains(pointer_button.position.x as f32, pointer_button.position.y as f32) {
                                    let offset_y = pointer_button.position.y as f32 - self.common_element_data.computed_scroll_track.y;

                                    let percent = offset_y / self.common_element_data.computed_scroll_track.height;
                                    let scroll_y = percent * self.common_element_data.max_scroll_y;

                                    container_state.scroll_y = scroll_y.clamp(0.0, self.common_element_data.max_scroll_y);

                                    UpdateResult::new().prevent_propagate().prevent_defaults()
                                } else {
                                    UpdateResult::new()
                                }
                            }
                            ElementState::Released => {
                                container_state.scroll_click = None;
                                UpdateResult::new().prevent_propagate().prevent_defaults()
                            }
                        }
                    } else {
                        UpdateResult::new()
                    }
                },
                OkuMessage::PointerMovedEvent(pointer_motion) => {
                    if let Some((click_x, click_y)) = container_state.scroll_click {
                        // Todo: Translate scroll wheel pixel to scroll position for diff.
                        let delta = pointer_motion.position.y as f32 - click_y;

                        let max_scroll_y = self.common_element_data.max_scroll_y;

                        let mut delta = max_scroll_y * (delta / (self.common_element_data.computed_scroll_track.height - self.common_element_data.computed_scroll_thumb.height));

                        // DEVICE(TOUCH): Reverse the direction on touch based input devices.
                        if let PointerSource::Touch {..} = pointer_motion.source {
                            delta = -delta;
                        }
                        
                        container_state.scroll_y = (container_state.scroll_y + delta).clamp(0.0, max_scroll_y);
                        container_state.scroll_click = Some((click_x, pointer_motion.position.y as f32));
                        UpdateResult::new().prevent_propagate().prevent_defaults()
                    } else {
                        UpdateResult::new()
                    }
                },
            _ => UpdateResult::new(),
            }
        } else {
            UpdateResult::new()
        }
    }

    fn initialize_state(&self, _font_system: &mut FontSystem) -> Box<StateStoreItem> {
        Box::new(ContainerState::default())
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

    generate_component_methods!();
}

impl ElementStyles for Container {
    fn styles_mut(&mut self) -> &mut Style {
        &mut self.common_element_data.style
    }
}