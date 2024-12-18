use crate::components::component::{ComponentId, ComponentOrElement, ComponentSpecification};
use crate::components::props::Props;
use crate::components::UpdateResult;
use crate::elements::layout_context::LayoutContext;
use crate::events::OkuMessage;
use crate::renderer::renderer::Rectangle;
use crate::reactive::state_store::{StateStore, StateStoreItem};
use crate::style::Style;
use crate::RendererBox;
use cosmic_text::FontSystem;
use std::any::Any;
use std::fmt::Debug;
use taffy::{NodeId, TaffyTree};
use crate::geometry::{Border, LayeredRectangle, Margin, Padding, Position, Size};

#[derive(Clone, Debug, Default)]
pub struct CommonElementData {
    pub style: Style,
    /// The children of the element.
    pub(crate) children: Vec<ElementBox>,
    
    pub computed_border_rectangle_overflow_size: Size,
    // The computed values after transforms are applied.
    pub computed_layered_rectangle_transformed: LayeredRectangle,
    // The computed values without any transforms applied to them.
    pub computed_layered_rectangle: LayeredRectangle,
    
    /// A user-defined id for the element.
    pub id: Option<String>,
    /// The id of the component that this element belongs to.
    pub component_id: ComponentId,
    pub computed_scrollbar_size: Size,
    pub scrollbar_size: Size,
    pub computed_scroll_track: Rectangle,
    pub computed_scroll_thumb: Rectangle,
    pub(crate) max_scroll_y: f32,
    pub layout_order: u32,

    // Used for converting the element to a component specification.
    pub(crate) child_specs: Vec<ComponentSpecification>,
    pub(crate) key: Option<String>,
    pub(crate) props: Option<Props>,
}

#[derive(Clone, Debug)]
pub struct ElementBox {
    pub(crate) internal: Box<dyn Element>,
}

pub(crate) trait Element: Any + StandardElementClone + Debug + Send + Sync {
    fn common_element_data(&self) -> &CommonElementData;
    fn common_element_data_mut(&mut self) -> &mut CommonElementData;

    fn children(&self) -> Vec<&dyn Element> {
        self.common_element_data().children.iter().map(|x| x.internal.as_ref()).collect()
    }

    fn children_mut(&mut self) -> &mut Vec<ElementBox> {
        &mut self.common_element_data_mut().children
    }

    fn style(&self) -> &Style {
        &self.common_element_data().style
    }

    fn style_mut(&mut self) -> &mut Style {
        &mut self.common_element_data_mut().style
    }
    
    fn in_bounds(&self, x: f32, y: f32) -> bool {
        let common_element_data = self.common_element_data();
        
        let transformed_border_rectangle = common_element_data.computed_layered_rectangle_transformed.border_rectangle();
        
        let left = transformed_border_rectangle.x;
        let right = left + transformed_border_rectangle.width;
        let top = transformed_border_rectangle.y;
        let bottom = top + transformed_border_rectangle.height;

        x >= left && x <= right && y >= top && y <= bottom
    }
    
    fn get_id(&self) -> &Option<String> {
        &self.common_element_data().id
    }

    #[allow(dead_code)]
    fn id(&mut self, id: Option<&str>) -> Box<dyn Element> {
        self.common_element_data_mut().id = id.map(String::from);
        self.clone_box()
    }

    fn component_id(&self) -> u64 {
        self.common_element_data().component_id
    }

    fn set_component_id(&mut self, id: u64) {
        self.common_element_data_mut().component_id = id;
    }

    fn name(&self) -> &'static str;

    fn draw(
        &mut self,
        renderer: &mut RendererBox,
        font_system: &mut FontSystem,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        element_state: &StateStore,
    );

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        font_system: &mut FontSystem,
        element_state: &mut StateStore,
        scale_factor: f64,
    ) -> NodeId;

    /// Finalizes the layout of the element.
    ///
    /// The majority of the layout computation is done in the `compute_layout` method.
    /// Store the computed values in the `common_element_data` struct.
    fn finalize_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        x: f32,
        y: f32,
        z_index: &mut u32,
        transform: glam::Mat4,
        font_system: &mut FontSystem,
        element_state: &mut StateStore,
    );

    fn as_any(&self) -> &dyn Any;

    fn on_event(
        &self,
        _message: OkuMessage,
        _element_state: &mut StateStore,
        _font_system: &mut FontSystem,
    ) -> UpdateResult {
        UpdateResult::default()
    }

    fn resolve_layer_rectangle(&mut self, relative_x: f32, relative_y: f32, scroll_transform: glam::Mat4, result: &taffy::Layout, layout_order: &mut u32) {
        let common_element_data_mut = self.common_element_data_mut();
        common_element_data_mut.layout_order = *layout_order;
        *layout_order += 1;
        
        let position = match common_element_data_mut.style.position {
            taffy::Position::Relative => {
                Position::new(relative_x + result.location.x, relative_y + result.location.y, 1.0)
            }
            taffy::Position::Absolute => {
                Position::new(result.location.x, result.location.y, 1.0)
            }
        };

        common_element_data_mut.computed_border_rectangle_overflow_size = Size::new(result.content_size.width, result.content_size.height);
        common_element_data_mut.computed_layered_rectangle = LayeredRectangle {
            margin: Margin::new(result.margin.top, result.margin.right, result.margin.bottom, result.margin.left),
            border: Border::new(result.border.top, result.border.right, result.border.bottom, result.border.left),
            padding: Padding::new(result.padding.top, result.padding.right, result.padding.bottom, result.padding.left),
            position,
            size: Size::new(result.size.width, result.size.height),
        };
        common_element_data_mut.computed_layered_rectangle_transformed = common_element_data_mut.computed_layered_rectangle.transform(scroll_transform);
    }

    fn finalize_scrollbar(&mut self, scroll_y: f32) {
        let common_element_data = self.common_element_data_mut();

        if common_element_data.style.overflow[0] != taffy::Overflow::Scroll {
            return;
        }

        let computed_layered_rectangle_transformed = common_element_data.computed_layered_rectangle_transformed.clone();
        let content_rectangle = computed_layered_rectangle_transformed.content_rectangle();

        let computed_content_height = common_element_data.computed_border_rectangle_overflow_size.height;

        let client_height = content_rectangle.height - computed_layered_rectangle_transformed.border.top - computed_layered_rectangle_transformed.border.bottom;
        let scroll_height = computed_content_height - computed_layered_rectangle_transformed.border.top;

        let scrolltrack_width = common_element_data.scrollbar_size.width;
        let scrolltrack_height = client_height;

        let max_scroll_y = (scroll_height - client_height).max(0.0);
        common_element_data.max_scroll_y = max_scroll_y;

        let visible_y = client_height / scroll_height;
        let scrollthumb_height = scrolltrack_height * visible_y;
        let remaining_height = scrolltrack_height - scrollthumb_height;
        let scrollthumb_offset = if max_scroll_y != 0.0 {
            scroll_y / max_scroll_y * remaining_height
        } else {
            0.0
        };

        common_element_data.computed_scroll_track = Rectangle::new(
            computed_layered_rectangle_transformed.position.x + computed_layered_rectangle_transformed.size.width - scrolltrack_width - computed_layered_rectangle_transformed.border.right,
            computed_layered_rectangle_transformed.position.y + computed_layered_rectangle_transformed.border.top,
            scrolltrack_width,
            scrolltrack_height,
        );

        let scrollthumb_width = scrolltrack_width;

        common_element_data.computed_scroll_thumb = Rectangle::new(
            computed_layered_rectangle_transformed.position.x + computed_layered_rectangle_transformed.size.width - scrolltrack_width - computed_layered_rectangle_transformed.border.right,
            computed_layered_rectangle_transformed.position.y + computed_layered_rectangle_transformed.border.top + scrollthumb_offset,
            scrollthumb_width,
            scrollthumb_height,
        );
    }
    
    /// Called when the element is assigned a unique component id.
    fn initialize_state(&self, _font_system: &mut FontSystem) -> Box<StateStoreItem> {
        Box::new(())
    }

    /// Called on sequential renders to update any state that the element may have.
    fn update_state(&self, _font_system: &mut FontSystem, _element_state: &mut StateStore) {
    }
}

impl<T: Element> From<T> for ElementBox {
    fn from(element: T) -> Self {
        ElementBox {
            internal: Box::new(element),
        }
    }
}

impl<T: Element> From<T> for ComponentOrElement {
    fn from(element: T) -> Self {
        ComponentOrElement::Element(element.into())
    }
}

impl From<ElementBox> for ComponentOrElement {
    fn from(element: ElementBox) -> Self {
        ComponentOrElement::Element(element)
    }
}

impl From<ElementBox> for ComponentSpecification {
    fn from(element: ElementBox) -> Self {
        let key = element.internal.common_element_data().key.clone();
        let children = element.internal.common_element_data().child_specs.clone();
        let props = element.internal.common_element_data().props.clone();
        ComponentSpecification {
            component: ComponentOrElement::Element(element),
            key,
            props,
            children,
        }
    }
}

impl<T> From<T> for ComponentSpecification
where
    T: Element,
{
    fn from(element: T) -> Self {
        let key = element.common_element_data().key.clone();
        let children_specs = element.common_element_data().child_specs.clone();
        let props = element.common_element_data().props.clone();
        ComponentSpecification {
            component: ComponentOrElement::Element(element.into()),
            key,
            props,
            children: children_specs,
        }
    }
}

impl dyn Element {
    pub fn print_tree(&self) {
        let mut elements: Vec<(&dyn Element, usize, bool)> = vec![(self, 0, true)];
        while let Some((element, indent, is_last)) = elements.pop() {
            let mut prefix = String::new();
            for _ in 0..indent {
                prefix.push_str("  ");
            }
            if is_last {
                prefix.push_str("└─");
            } else {
                prefix.push_str("├─");
            }
            println!(
                "{}{}, Component Id: {} Id: {:?}",
                prefix,
                element.name(),
                element.component_id(),
                element.get_id()
            );
            let children = element.children();
            for (i, child) in children.iter().enumerate().rev() {
                let is_last = i == children.len() - 1;
                elements.push((*child, indent + 1, is_last));
            }
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
        pub fn component(self) -> ComponentSpecification {
            ComponentSpecification::new(self.into())
        }

        pub fn key(mut self, key: &str) -> Self {
            self.common_element_data.key = Some(key.to_string());

            self
        }

        pub fn props(mut self, props: Props) -> Self {
            self.common_element_data.props = Some(props);

            self
        }
    };
}

#[macro_export]
macro_rules! generate_component_methods {
    () => {
        pub fn component(self) -> ComponentSpecification {
            ComponentSpecification::new(self.into())
        }

        pub fn key(mut self, key: &str) -> Self {
            self.common_element_data.key = Some(key.to_string());

            self
        }

        pub fn props(mut self, props: Props) -> Self {
            self.common_element_data.props = Some(props);

            self
        }

        pub fn push<T>(mut self, component_specification: T) -> Self
        where
            T: Into<ComponentSpecification>,
        {
            self.common_element_data.child_specs.push(component_specification.into());

            self
        }

        pub fn push_children<T>(mut self, children: Vec<T>) -> Self
        where
            T: Into<ComponentSpecification>,
        {
            self.common_element_data.child_specs = children.into_iter().map(|x| x.into()).collect();

            self
        }
    };
}
