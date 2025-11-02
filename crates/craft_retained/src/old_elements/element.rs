use crate::events::{CraftMessage, Event};
use crate::layout::layout_context::LayoutContext;
use crate::layout::layout_item::{draw_borders_generic, LayoutItem};
use crate::style::Style;
use crate::text::text_context::TextContext;
#[cfg(feature = "accesskit")]
use accesskit::{Action, Role};
use craft_primitives::geometry::borders::{BorderSpec, ComputedBorderSpec};
use craft_primitives::geometry::{ElementBox, Point, Rectangle, TrblRectangle};
use craft_renderer::renderer::RenderList;
use kurbo::Affine;
use peniko::Color;
use rustc_hash::FxHashMap;
use smallvec::SmallVec;
use std::any::Any;
use std::mem;
use std::sync::Arc;
use std::time::Duration;
use taffy::{NodeId, Overflow, TaffyTree};
use winit::window::Window;

#[derive(Clone)]
pub struct ElementBoxed {
    pub internal: Box<dyn Element>,
}

pub trait Element: Any + StandardElementClone + Send + Sync {

    fn children(&self) -> &SmallVec<[ElementBoxed; 4]>;

    fn children_mut(&mut self) -> &mut SmallVec<[ElementBoxed; 4]>;

    fn in_bounds(&self, point: Point) -> bool {
      true
    }

    #[allow(clippy::too_many_arguments)]
    fn draw(
        &mut self,
        renderer: &mut RenderList,
        text_context: &mut TextContext,
        pointer: Option<Point>,
        window: Option<Arc<Window>>,
        scale_factor: f64,
    );

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        scale_factor: f64,
    ) -> Option<NodeId>;

    /// Finalizes the layout of the element.
    ///
    /// The majority of the layout computation is done in the `compute_layout` method.
    /// Store the computed values in the `element_data` struct.
    #[allow(clippy::too_many_arguments)]
    fn finalize_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        position: Point,
        z_index: &mut u32,
        transform: Affine,
        pointer: Option<Point>,
        text_context: &mut TextContext,
        clip_bounds: Option<Rectangle>,
    );

    fn as_any(&self) -> &dyn Any;

    #[allow(clippy::too_many_arguments)]
    fn on_event(
        &self,
        message: &CraftMessage,
        _text_context: &mut TextContext,
        should_style: bool,
        event: &mut Event,
        target: Option<&dyn Element>,
        _current_target: Option<&dyn Element>,
    ) {
    }

    fn resolve_clip(&mut self, clip_bounds: Option<Rectangle>) {
    }

    fn resolve_box(
        &mut self,
        relative_position: Point,
        scroll_transform: Affine,
        result: &taffy::Layout,
        layout_order: &mut u32,
    ) {
    }
    
    /// A bit of a hack to reset the layout item of an element recursively.
    fn reset_layout_item(&mut self) {
    }

    fn draw_children(
        &mut self,
        renderer: &mut RenderList,
        text_context: &mut TextContext,
        pointer: Option<Point>,
        window: Option<Arc<Window>>,
        scale_factor: f64,
    ) {

    }

    fn draw_borders(&self, renderer: &mut RenderList, scale_factor: f64) {
    }

    fn should_start_new_layer(&self) -> bool {
        false
    }

    fn maybe_start_layer(&self, renderer: &mut RenderList, scale_factor: f64) {
    }

    fn maybe_end_layer(&self, renderer: &mut RenderList) {
    }

    fn draw_scrollbar(&mut self, renderer: &mut RenderList, scale_factor: f64) {

    }

    fn default_style(&self) -> Style {
        Style::default()
    }

    fn merge_default_style(&mut self) {
    }

    // Easy ways to access common items from layout item:
    fn taffy_node_id(&self) -> Option<NodeId> {
        None
    }

    fn computed_box_transformed(&self) -> ElementBox {
        ElementBox::default()
    }
}

impl<T: Element> From<T> for ElementBoxed {
    fn from(element: T) -> Self {
        ElementBoxed {
            internal: Box::new(element),
        }
    }
}

pub trait StandardElementClone {
    fn clone_box(&self) -> Box<dyn Element>;
}

impl<T> StandardElementClone for T
where
    T: Element + Clone,
{
    fn clone_box(&self) -> Box<dyn Element> {
        Box::new(self.clone())
    }
}

// We can now implement Clone manually by forwarding to clone_box.
impl Clone for Box<dyn Element> {
    fn clone(&self) -> Box<dyn Element> {
        self.clone_box()
    }
}

#[macro_export]
macro_rules! generate_component_methods_no_children {
    () => {
        #[allow(dead_code)]
        pub fn component(self) -> ComponentSpecification {
            ComponentSpecification::new(self.into())
        }

        #[allow(dead_code)]
        pub fn key<T: Into<SmolStr>>(mut self, key: T) -> Self {
            self.element_data.key = Some(key.into());
            self
        }

        #[allow(dead_code)]
        pub fn props(mut self, props: Props) -> Self {
            self.element_data.props = Some(props);
            self
        }

        #[allow(dead_code)]
        pub fn id<T: Into<SmolStr>>(mut self, id: T) -> Self {
            self.element_data.id = Some(id.into());
            self
        }

        #[allow(dead_code)]
        pub fn normal(mut self) -> Self {
            self.element_data.current_state = $crate::old_elements::element_states::ElementState::Normal;
            self
        }

        #[allow(dead_code)]
        pub fn hovered(mut self) -> Self {
            self.element_data.current_state = $crate::old_elements::element_states::ElementState::Hovered;
            self
        }

        #[allow(dead_code)]
        pub fn pressed(mut self) -> Self {
            self.element_data.current_state = $crate::old_elements::element_states::ElementState::Pressed;
            self
        }

        #[allow(dead_code)]
        pub fn disabled(mut self) -> Self {
            self.element_data.current_state = $crate::old_elements::element_states::ElementState::Disabled;
            self
        }

        #[allow(dead_code)]
        pub fn focused(mut self) -> Self {
            self.element_data.current_state = $crate::old_elements::element_states::ElementState::Focused;
            self
        }
    };
}

#[macro_export]
macro_rules! generate_component_methods_private_push {
    () => {
        $crate::generate_component_methods_no_children!();

        #[allow(dead_code)]
        fn push<T>(mut self, component_specification: T) -> Self
        where
            T: Into<ComponentSpecification>,
        {
            self.element_data.child_specs.push(component_specification.into());

            self
        }

        #[allow(dead_code)]
        fn push_children<T>(mut self, children: Vec<T>) -> Self
        where
            T: Into<ComponentSpecification>,
        {
            self.element_data.child_specs = children.into_iter().map(|x| x.into()).collect();

            self
        }

        #[allow(dead_code)]
        fn extend_children<T>(mut self, children: Vec<T>) -> Self
        where
            T: Into<ComponentSpecification>,
        {
            self.element_data.child_specs.extend(children.into_iter().map(|x| x.into()));

            self
        }
    };
}

#[macro_export]
macro_rules! generate_component_methods {

    () => {
        $crate::generate_component_methods_no_children!();

        #[allow(dead_code)]
        pub fn push<T>(mut self, component_specification: T) -> Self
        where
            T: Into<ComponentSpecification>,
        {
            self.element_data.child_specs.push(component_specification.into());

            self
        }

        #[allow(dead_code)]
        pub fn push_children<T>(mut self, children: Vec<T>) -> Self
        where
            T: Into<ComponentSpecification>,
        {
            self.element_data.child_specs = children.into_iter().map(|x| x.into()).collect();

            self
        }

        #[allow(dead_code)]
        pub fn extend_children<T>(mut self, children: Vec<T>) -> Self
        where
            T: Into<ComponentSpecification>,
        {
            self.element_data.child_specs.extend(children.into_iter().map(|x| x.into()));

            self
        }

        #[allow(dead_code)]
        pub fn extend_children_in_place<T>(&mut self, children: Vec<T>)
        where
            T: Into<ComponentSpecification>,
        {
            self.element_data.child_specs.extend(children.into_iter().map(|x| x.into()));
        }

        #[allow(dead_code)]
        pub fn push_in_place(&mut self, component_specification: ComponentSpecification) {
            self.element_data.child_specs.push(component_specification);
        }
    };
}