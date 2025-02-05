use crate::components::component::ComponentSpecification;
use crate::components::props::Props;
use crate::components::UpdateResult;
use crate::elements::common_element_data::CommonElementData;
use crate::elements::element::Element;
use crate::elements::element_styles::ElementStyles;
use crate::elements::layout_context::LayoutContext;
use crate::events::OkuMessage;
use crate::geometry::{Point, Size};
use crate::reactive::element_state_store::{ElementStateStore, ElementStateStoreItem};
use crate::style::Style;
use crate::{generate_component_methods, RendererBox};
use cosmic_text::FontSystem;
use std::any::Any;
use taffy::{NodeId, Overflow, TaffyTree};
use winit::event::{ButtonSource, ElementState as WinitElementState, MouseButton, MouseScrollDelta, PointerSource};
use crate::elements::base_element_state::DUMMY_DEVICE_ID;

/// A stateless element that stores other elements.
#[derive(Clone, Default, Debug)]
pub struct Container {
    pub common_element_data: CommonElementData,
}

#[derive(Clone, Copy, Default)]
pub struct ContainerState {
    pub(crate) scroll_y: f32,
    pub(crate) scroll_click: Option<(f32, f32)>,
}

impl Container {
    pub fn draw_scrollbar(&mut self, renderer: &mut RendererBox) {
        let scrollbar_color = self.common_element_data.current_style().scrollbar_color();

        // track
        renderer.draw_rect(
            self.common_element_data.computed_scroll_track,
            scrollbar_color.track_color,
        );

        // thumb
        renderer.draw_rect(
            self.common_element_data.computed_scroll_thumb,
            scrollbar_color.thumb_color,
        );
    }
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
        _root_node: NodeId,
        element_state: &ElementStateStore,
        pointer: Option<Point>,
    ) {
        let computed_layer_rectangle_transformed = self.common_element_data.computed_layered_rectangle_transformed;
        let padding_rectangle = computed_layer_rectangle_transformed.padding_rectangle();
        
        self.draw_borders(renderer);
        if self.common_element_data.current_style().overflow()[1] == Overflow::Scroll {
            renderer.push_layer(padding_rectangle);
        }
       self.draw_children(renderer, font_system, taffy_tree, element_state, pointer);
        if self.common_element_data.style.overflow()[1] == Overflow::Scroll {
            renderer.pop_layer();
        }
        
        self.draw_scrollbar(renderer);
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
        
        self.common_element_data.scrollbar_size = Size::new(result.scrollbar_size.width, result.scrollbar_size.height);
        self.common_element_data.computed_scrollbar_size = Size::new(result.scroll_width(), result.scroll_height());

        let scroll_y = if let Some(container_state) =
            element_state.storage.get(&self.common_element_data.component_id).unwrap().data.downcast_ref::<ContainerState>()
        {
            container_state.scroll_y
        } else {
            0.0
        };
        
        self.finalize_scrollbar(scroll_y);
        let child_transform = glam::Mat4::from_translation(glam::Vec3::new(0.0, -scroll_y, 0.0));
        
        for child in self.common_element_data.children.iter_mut() {
            let taffy_child_node_id = child.internal.common_element_data().taffy_node_id;
            if taffy_child_node_id.is_none() {
                continue;
            }
            
            child.internal.finalize_layout(
                taffy_tree,
                taffy_child_node_id.unwrap(),
                self.common_element_data.computed_layered_rectangle.position,
                z_index,
                transform * child_transform,
                font_system,
                element_state,
                pointer,
            );
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn on_event(&self, message: OkuMessage, element_state: &mut ElementStateStore, _font_system: &mut FontSystem) -> UpdateResult {
        let base_state = self.get_base_state_mut(element_state);
        let container_state = base_state.data.as_mut().downcast_mut::<ContainerState>().unwrap();
        
        if self.style().overflow()[1] == taffy::Overflow::Scroll {
            match message {
                OkuMessage::MouseWheelEvent(mouse_wheel) => {
                    let delta = match mouse_wheel.delta {
                        MouseScrollDelta::LineDelta(_x, y) => y,
                        MouseScrollDelta::PixelDelta(y) => y.y as f32,
                    };
                    let delta = -delta * self.common_element_data.style.font_size().max(12.0) * 1.2;
                    let max_scroll_y = self.common_element_data.max_scroll_y;

                    container_state.scroll_y = (container_state.scroll_y + delta).clamp(0.0, max_scroll_y);

                    UpdateResult::new().prevent_propagate().prevent_defaults()
                }
                OkuMessage::PointerButtonEvent(pointer_button) => {
                    if pointer_button.button.mouse_button() == MouseButton::Left {
                        
                        // DEVICE(TOUCH): Handle scrolling within the content area on touch based input devices.
                        if let ButtonSource::Touch { .. } = pointer_button.button {
                            let container_rectangle = self.common_element_data.computed_layered_rectangle_transformed.padding_rectangle();
                            
                            let in_scroll_bar = self.common_element_data.computed_scroll_thumb.contains(&pointer_button.position);

                            if container_rectangle.contains(&pointer_button.position) && !in_scroll_bar {
                                container_state.scroll_click = Some((pointer_button.position.x, pointer_button.position.y));
                                return UpdateResult::new().prevent_propagate().prevent_defaults();
                            }
                        }

                        match pointer_button.state {
                            WinitElementState::Pressed => {
                                if self.common_element_data.computed_scroll_thumb.contains(&pointer_button.position) {
                                    container_state.scroll_click = Some((pointer_button.position.x, pointer_button.position.y));
                                    // FIXME: Turn pointer capture on with the correct device id. 
                                    base_state.base.pointer_capture.insert(DUMMY_DEVICE_ID, true);
                                    
                                    UpdateResult::new().prevent_propagate().prevent_defaults()
                                } else if self.common_element_data.computed_scroll_track.contains(&pointer_button.position) {
                                    let offset_y = pointer_button.position.y - self.common_element_data.computed_scroll_track.y;

                                    let percent = offset_y / self.common_element_data.computed_scroll_track.height;
                                    let scroll_y = percent * self.common_element_data.max_scroll_y;

                                    container_state.scroll_y = scroll_y.clamp(0.0, self.common_element_data.max_scroll_y);

                                    UpdateResult::new().prevent_propagate().prevent_defaults()
                                } else {
                                    UpdateResult::new()
                                }
                            }
                            WinitElementState::Released => {
                                container_state.scroll_click = None;
                                // FIXME: Turn pointer capture off with the correct device id. 
                                base_state.base.pointer_capture.insert(DUMMY_DEVICE_ID, false);
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
                        let delta = pointer_motion.position.y - click_y;

                        let max_scroll_y = self.common_element_data.max_scroll_y;

                        let mut delta = max_scroll_y * (delta / (self.common_element_data.computed_scroll_track.height - self.common_element_data.computed_scroll_thumb.height));

                        // DEVICE(TOUCH): Reverse the direction on touch based input devices.
                        if let PointerSource::Touch {..} = pointer_motion.source {
                            delta = -delta;
                        }
                        
                        container_state.scroll_y = (container_state.scroll_y + delta).clamp(0.0, max_scroll_y);
                        container_state.scroll_click = Some((click_x, pointer_motion.position.y));
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

    fn initialize_state(&self, _font_system: &mut FontSystem) -> ElementStateStoreItem {
        ElementStateStoreItem {
            base: Default::default(),
            data: Box::new(ContainerState::default())
        }
    }
}

impl Container {
    #[allow(dead_code)]
    fn get_state<'a>(&self, element_state: &'a ElementStateStore) -> &'a &ContainerState {
        element_state.storage.get(&self.common_element_data.component_id).unwrap().data.as_ref().downcast_ref().unwrap()
    }

    fn get_state_mut<'a>(&self, element_state: &'a mut ElementStateStore) -> &'a mut ContainerState {
        element_state.storage.get_mut(&self.common_element_data.component_id).unwrap().data.as_mut().downcast_mut().unwrap()
    }

    pub fn new() -> Container {
        Container {
            common_element_data: Default::default(),
        }
    }
    
    generate_component_methods!();
}

impl ElementStyles for Container {
    fn styles_mut(&mut self) -> &mut Style {
        self.common_element_data.current_style_mut()
    }
}
