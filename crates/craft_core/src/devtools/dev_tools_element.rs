use crate::components::component::ComponentSpecification;
use crate::components::Props;
use crate::components::{ComponentId, Event};
use crate::elements::element::Element;
use crate::elements::element_data::ElementData;
use crate::elements::element_styles::ElementStyles;
use crate::events::CraftMessage;
use crate::generate_component_methods;
use craft_primitives::geometry::{Point, Rectangle};
use crate::layout::layout_context::LayoutContext;
use crate::reactive::element_state_store::{ElementStateStore, ElementStateStoreItem};
use craft_primitives::Color;
use craft_renderer::renderer::RenderList;
use crate::style::Style;
use crate::text::text_context::TextContext;
use std::any::Any;
use std::sync::Arc;
use kurbo::Affine;
use taffy::{NodeId, TaffyTree};
use winit::window::Window;
use smol_str::SmolStr;

#[derive(Clone, Default)]
pub struct DevTools {
    pub element_data: ElementData,
    /// The tree to inspect.
    pub(crate) debug_inspector_tree: Option<Box<dyn Element>>,
    /// The selected element in the inspector tree.
    pub(crate) selected_inspector_element: Option<ComponentId>,
    /// The hovered element in the inspector tree.
    pub(crate) hovered_inspector_element: Option<ComponentId>,
}

#[derive(Clone, Copy, Default)]
pub struct DevToolsState {}

impl DevTools {
    pub fn push_debug_inspector_tree(mut self, root: Box<dyn Element>) -> Self {
        self.debug_inspector_tree = Some(root.clone());
        self
    }
    pub fn push_selected_inspector_element(mut self, element: Option<ComponentId>) -> Self {
        self.selected_inspector_element = element;
        self
    }
    pub fn push_hovered_inspector_element(mut self, element: Option<ComponentId>) -> Self {
        self.hovered_inspector_element = element;
        self
    }
}

impl Element for DevTools {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }

    fn name(&self) -> &'static str {
        "Dev Tools"
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
        self.draw_borders(renderer, element_state, scale_factor);
        self.draw_children(renderer, text_context, element_state, pointer, window, scale_factor);

        // Find the element we are hovering over and draw an overlay.
        if let Some(hovered_inspector_element_component_id) = self.hovered_inspector_element {
            let mut hovered_inspector_element: Option<&dyn Element> = None;
            let root = self.debug_inspector_tree.as_ref().unwrap();

            // Find the hovered inspector element.
            for element in root.pre_order_iter().collect::<Vec<&dyn Element>>().iter().rev() {
                if element.component_id() != hovered_inspector_element_component_id {
                    continue;
                }

                hovered_inspector_element = Some(*Box::new(<&dyn Element>::clone(element)));
                break;
            }

            // Highlight the hovered element and draw their margin, padding, and content box.
            if let Some(selected_element) = hovered_inspector_element {
                // FIXME: Make use of layers, so that the boxes only mix with the element's colors.
                let margin_box_highlight_color = Color::from_rgba8(255, 0, 0, 200);
                let border_box_highlight_color = Color::from_rgba8(0, 255, 0, 200);
                let padding_box_highlight_color = Color::from_rgba8(0, 0, 255, 200);
                let content_box_highlight_color = Color::from_rgba8(0, 255, 255, 200);

                let margin_rectangle =
                    selected_element.element_data().layout_item.computed_box_transformed.margin_rectangle().scale(scale_factor);
                renderer.push_layer(margin_rectangle);
                renderer.draw_rect(margin_rectangle, margin_box_highlight_color);
                renderer.pop_layer();

                let border_rectangle =
                    selected_element.element_data().layout_item.computed_box_transformed.border_rectangle().scale(scale_factor);
                renderer.push_layer(border_rectangle);
                renderer.draw_rect(border_rectangle, border_box_highlight_color);
                renderer.pop_layer();

                let padding_rectangle =
                    selected_element.element_data().layout_item.computed_box_transformed.padding_rectangle().scale(scale_factor);
                renderer.push_layer(padding_rectangle);
                renderer.draw_rect(padding_rectangle, padding_box_highlight_color);
                renderer.pop_layer();

                let content_rectangle =
                    selected_element.element_data().layout_item.computed_box_transformed.content_rectangle().scale(scale_factor);
                renderer.push_layer(content_rectangle);
                renderer.draw_rect(content_rectangle, content_box_highlight_color);
                renderer.pop_layer();

                if let Some(clip_bounds) = selected_element.element_data().layout_item.clip_bounds {
                    let clip_bounds = clip_bounds.scale(scale_factor);
                    renderer.push_layer(clip_bounds);
                    renderer.draw_rect_outline(clip_bounds, Color::from_rgba8(255, 0, 0, 255));
                    renderer.pop_layer();
                }
            }
        }
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

        let style: taffy::Style = self.element_data.style.to_taffy_style();

        self.element_data.layout_item.build_tree(taffy_tree, style)
    }

    fn finalize_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        position: Point,
        z_index: &mut u32,
        transform: Affine,
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
            let taffy_child_node_id = child.internal.element_data().layout_item.taffy_node_id;
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

    fn on_event(
        &self,
        message: &CraftMessage,
        element_state: &mut ElementStateStore,
        _text_context: &mut TextContext,
        should_style: bool,
        event: &mut Event,
        target: Option<&dyn Element>,
        _current_target: Option<&dyn Element>,
    ) {
        self.on_style_event(message, element_state, should_style, event);
        self.maybe_unset_focus(message, event, target);
    }

    fn initialize_state(&mut self, _scaling_factor: f64) -> ElementStateStoreItem {
        ElementStateStoreItem {
            base: Default::default(),
            data: Box::new(DevToolsState::default()),
        }
    }
}

impl DevTools {
    #[allow(dead_code)]
    fn get_state<'a>(&self, element_state: &'a ElementStateStore) -> &'a DevToolsState {
        element_state.storage.get(&self.element_data.component_id).unwrap().data.as_ref().downcast_ref().unwrap()
    }

    pub fn new() -> DevTools {
        DevTools {
            debug_inspector_tree: None,
            element_data: Default::default(),
            selected_inspector_element: None,
            hovered_inspector_element: None,
        }
    }

    generate_component_methods!();
}

impl ElementStyles for DevTools {
    fn styles_mut(&mut self) -> &mut Style {
        &mut self.element_data.style
    }
}
