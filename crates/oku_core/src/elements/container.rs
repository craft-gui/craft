use crate::engine::renderer::color::Color;
use crate::engine::renderer::renderer::Rectangle;
use crate::components::component::{ComponentId, ComponentSpecification};
use crate::elements::element::{CommonElementData, Element, ElementBox};
use crate::elements::layout_context::LayoutContext;
use crate::style::{AlignItems, Display, FlexDirection, JustifyContent, Overflow, Unit, Wrap};
use crate::RendererBox;
use cosmic_text::FontSystem;
use std::any::Any;
use std::collections::HashMap;
use taffy::{NodeId, TaffyTree};
use winit::event::MouseScrollDelta;
use crate::components::UpdateResult;
use crate::engine::events::OkuEvent;
use crate::reactive::state_store::{StateStoreItem, StateStore};

/// A stateless element that stores other elements.
#[derive(Clone, Default, Debug)]
pub struct Container {
    pub common_element_data: CommonElementData,
}

pub struct ContainerState {
    pub(crate) scroll_delta_y: f32,
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
        renderer.draw_rect(
            Rectangle::new(
                self.common_element_data.computed_x_transformed,
                self.common_element_data.computed_y_transformed,
                self.common_element_data.computed_width,
                self.common_element_data.computed_height,
            ),
            self.common_element_data.style.background
        );
        
        for (index, child) in self.common_element_data.children.iter_mut().enumerate() {
            let child2 = taffy_tree.child_at_index(root_node, index).unwrap();
            child.internal.draw(renderer, font_system, taffy_tree, child2, element_state);
        }
    }

    fn compute_layout(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>, font_system: &mut FontSystem, element_state: &mut StateStore) -> NodeId {
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

        self.common_element_data.computed_x = x + result.location.x;
        self.common_element_data.computed_y = y + result.location.y;
        self.common_element_data.computed_width = result.size.width;
        self.common_element_data.computed_height = result.size.height;
        self.common_element_data.computed_scrollbar_width = result.scroll_width();
        self.common_element_data.computed_scrollbar_height = result.scroll_height();
        self.common_element_data.computed_padding = [result.padding.top, result.padding.right, result.padding.bottom, result.padding.left];
        
        let transformed_xy =  transform.mul_vec4(glam::vec4(self.common_element_data.computed_x, self.common_element_data.computed_y, 0.0, 1.0));
        self.common_element_data.computed_x_transformed = transformed_xy.x;
        self.common_element_data.computed_y_transformed = transformed_xy.y;

        let scrollbar_dy = if let Some(container_state) = element_state.storage.get(&self.common_element_data.component_id).unwrap().downcast_ref::<ContainerState>(){
            container_state.scroll_delta_y
        } else {
            0.0
        } * 100.0;


        let child_transform = glam::Mat4::from_translation(glam::Vec3::new(0.0, scrollbar_dy, 0.0));

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

    fn on_event(&self, event: OkuEvent, element_state: &mut StateStore, font_system: &mut FontSystem) -> UpdateResult {
        let container_state = self.get_state_mut(element_state);

        if self.style().overflow[1].is_scroll_container() {
            match event {
                OkuEvent::MouseWheelEvent(mouse_wheel) => {
                    let delta = match mouse_wheel.delta {
                        MouseScrollDelta::LineDelta(x, y) => { y }
                        MouseScrollDelta::PixelDelta(y) => { y.y as f32 }
                    };
                    container_state.scroll_delta_y += 1.0 * delta;
                    UpdateResult::new().prevent_propagate().prevent_defaults()
                }
                _ => {
                    UpdateResult::new()
                }
            }
        } else {
            UpdateResult::new()
        }

    }
}

impl Container {

    fn get_state<'a>(
        &self,
        element_state: &'a StateStore,
    ) -> &'a &ContainerState {
        element_state.storage.get(&self.common_element_data.component_id).unwrap().as_ref().downcast_ref().unwrap()
    }

    fn get_state_mut<'a>(
        &self,
        element_state: &'a mut StateStore,
    ) -> &'a mut ContainerState {
        element_state.storage.get_mut(&self.common_element_data.component_id).unwrap().as_mut().downcast_mut().unwrap()
    }

    pub fn new() -> Container {
        Container {
            common_element_data: Default::default(),
        }
    }
    
    pub const fn margin(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        self.common_element_data.style.margin = [top, right, bottom, left];
        self
    }
    pub const fn padding(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        self.common_element_data.style.padding = [top, right, bottom, left];
        self
    }

    pub const fn background(mut self, background: Color) -> Self {
        self.common_element_data.style.background = background;
        self
    }

    pub const fn display(mut self, display: Display) -> Self {
        self.common_element_data.style.display = display;
        self
    }

    pub const fn wrap(mut self, wrap: Wrap) -> Self {
        self.common_element_data.style.wrap = wrap;
        self
    }

    pub const fn justify_content(mut self, justify_content: JustifyContent) -> Self {
        self.common_element_data.style.justify_content = Some(justify_content);
        self
    }

    pub const fn align_items(mut self, align_items: AlignItems) -> Self {
        self.common_element_data.style.align_items = Some(align_items);
        self
    }

    pub const fn flex_direction(mut self, flex_direction: FlexDirection) -> Self {
        self.common_element_data.style.flex_direction = flex_direction;
        self
    }

    pub const fn flex_grow(mut self, flex_grow: f32) -> Self {
        self.common_element_data.style.flex_grow = flex_grow;
        self
    }

    pub const fn flex_shrink(mut self, flex_shrink: f32) -> Self {
        self.common_element_data.style.flex_shrink = flex_shrink;
        self
    }

    pub const fn flex_basis(mut self, flex_basis: Unit) -> Self {
        self.common_element_data.style.flex_basis = flex_basis;
        self
    }

    pub const fn width(mut self, width: Unit) -> Self {
        self.common_element_data.style.width = width;
        self
    }

    pub const fn height(mut self, height: Unit) -> Self {
        self.common_element_data.style.height = height;
        self
    }

    pub const fn max_width(mut self, max_width: Unit) -> Self {
        self.common_element_data.style.max_width = max_width;
        self
    }

    pub const fn max_height(mut self, max_height: Unit) -> Self {
        self.common_element_data.style.max_height = max_height;
        self
    }
    
    pub const fn overflow_x(mut self, overflow: Overflow) -> Self {
        self.common_element_data.style.overflow[0] = overflow;
        self
    }

    pub const fn overflow_y(mut self, overflow: Overflow) -> Self {
        self.common_element_data.style.overflow[1] = overflow;
        self
    }

    pub fn id(mut self, id: &str) -> Self {
        self.common_element_data.id = Some(id.to_string());
        self
    }
    
    pub fn component(self) -> ComponentSpecification {
        ComponentSpecification::new(self.into())
    }
}
