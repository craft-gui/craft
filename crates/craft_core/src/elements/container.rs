use crate::components::component::ComponentSpecification;
use crate::components::Event;
use crate::components::Props;
use crate::elements::element::{resolve_clip_for_scrollable, Element};
use crate::elements::element_data::ElementData;
use crate::elements::element_styles::ElementStyles;
use crate::elements::scroll_state::ScrollState;
use crate::events::CraftMessage;
use crate::generate_component_methods;
use crate::geometry::{Point, Rectangle, Size};
use crate::layout::layout_context::LayoutContext;
use crate::reactive::element_state_store::{ElementStateStore, ElementStateStoreItem};
use crate::renderer::renderer::RenderList;
use crate::style::Style;
use crate::text::text_context::TextContext;
use std::any::Any;
use std::sync::Arc;
use taffy::{NodeId, TaffyTree};
use winit::window::Window;

/// An element for storing related elements.
#[derive(Clone, Default)]
pub struct Container {
    pub element_data: ElementData,
}

#[derive(Clone, Copy, Default)]
pub struct ContainerState {
    pub(crate) scroll_state: ScrollState,
}

impl Element for Container {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }

    fn name(&self) -> &'static str {
        "Container"
    }

    fn draw(
        &mut self,
        renderer: &mut RenderList,
        text_context: &mut TextContext,
        element_state: &mut ElementStateStore,
        pointer: Option<Point>,
        window: Option<Arc<Window>>,
        scale_factor: f64,
    ) {
        let base_state = self.get_base_state_mut(element_state);
        let current_style = base_state.base.current_style(self.element_data());

        if !current_style.visible() {
            return;
        }

        // We draw the borders before we start any layers, so that we don't clip the borders.
        self.draw_borders(renderer, element_state, scale_factor);
        self.maybe_start_layer(renderer, scale_factor);
        {
            self.draw_children(renderer, text_context, element_state, pointer, window, scale_factor);
        }
        self.maybe_end_layer(renderer);

        self.draw_scrollbar(renderer, scale_factor);
    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        element_state: &mut ElementStateStore,
        scale_factor: f64,
    ) -> Option<NodeId> {
        self.merge_default_style();

        for child in &mut self.element_data.children {
            let child_node = child.internal.compute_layout(taffy_tree, element_state, scale_factor);
            self.element_data.layout_item.push_child(&child_node);
        }

        let base_state = self.get_base_state_mut(element_state);
        base_state.base.current_style_mut(&mut self.element_data);

        let current_style = {
            let base_state = self.get_base_state(element_state);
            base_state.base.current_style(&self.element_data).to_taffy_style()
        };

        self.element_data.layout_item.build_tree(taffy_tree, current_style)
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
        self.finalize_borders(element_state);

        self.element_data.layout_item.scrollbar_size =
            Size::new(result.scrollbar_size.width, result.scrollbar_size.height);
        self.element_data.layout_item.computed_scrollbar_size =
            Size::new(result.scroll_width(), result.scroll_height());

        let scroll_y = if let Some(container_state) = element_state
            .storage
            .get_mut(&self.element_data.component_id)
            .unwrap()
            .data
            .downcast_mut::<ContainerState>()
        {
            self.finalize_scrollbar(&mut container_state.scroll_state);
            container_state.scroll_state.scroll_y
        } else {
            0.0
        };
        self.resolve_clip(clip_bounds);

        let child_transform = glam::Mat4::from_translation(glam::Vec3::new(0.0, -scroll_y, 0.0));

        for child in self.element_data.children.iter_mut() {
            let taffy_child_node_id = child.internal.element_data().layout_item.taffy_node_id;
            if taffy_child_node_id.is_none() {
                continue;
            }

            child.internal.finalize_layout(
                taffy_tree,
                taffy_child_node_id.unwrap(),
                self.element_data.layout_item.computed_box.position,
                z_index,
                transform * child_transform,
                element_state,
                pointer,
                text_context,
                self.element_data.layout_item.clip_bounds,
            );
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn on_event(
        &self,
        message: &CraftMessage,
        element_state: &mut ElementStateStore,
        _text_context: &mut TextContext,
        should_style: bool,
        event: &mut Event,
    ) {
        self.on_style_event(message, element_state, should_style, event);
        let base_state = self.get_base_state_mut(element_state);
        let container_state = base_state.data.as_mut().downcast_mut::<ContainerState>().unwrap();

        container_state.scroll_state.on_event(message, &self.element_data, &mut base_state.base, event);
    }

    fn resolve_clip(&mut self, clip_bounds: Option<Rectangle>) {
        resolve_clip_for_scrollable(self, clip_bounds);
    }

    fn initialize_state(&mut self, _scaling_factor: f64) -> ElementStateStoreItem {
        ElementStateStoreItem {
            base: Default::default(),
            data: Box::new(ContainerState::default()),
        }
    }
}

impl Container {
    #[allow(dead_code)]
    fn get_state<'a>(&self, element_state: &'a ElementStateStore) -> &'a ContainerState {
        element_state.storage.get(&self.element_data.component_id).unwrap().data.as_ref().downcast_ref().unwrap()
    }

    pub fn new() -> Container {
        Container {
            element_data: Default::default(),
        }
    }

    generate_component_methods!();
}

impl ElementStyles for Container {
    fn styles_mut(&mut self) -> &mut Style {
        self.element_data.current_style_mut()
    }
}
