use crate::components::component::{ComponentId, ComponentOrElement, ComponentSpecification};
use crate::components::UpdateResult;
use crate::elements::layout_context::LayoutContext;
use crate::engine::events::OkuMessage;
use crate::engine::renderer::renderer::Rectangle;
use crate::reactive::state_store::StateStore;
use crate::style::Style;
use crate::RendererBox;
use cosmic_text::FontSystem;
use std::any::Any;
use std::fmt::Debug;
use taffy::{NodeId, TaffyTree};

#[derive(Clone, Debug, Default)]
pub struct CommonElementData {
    pub style: Style,
    /// The children of the element.
    pub(crate) children: Vec<ElementBox>,
    // The computed values after transforms are applied.
    pub computed_x_transformed: f32,
    pub computed_y_transformed: f32,

    // The computed values without any transforms applied to them.
    pub computed_x: f32,
    pub computed_y: f32,
    pub computed_width: f32,
    pub computed_height: f32,
    pub computed_scrollbar_width: f32,
    pub computed_scrollbar_height: f32,
    pub computed_content_width: f32,
    pub computed_content_height: f32,
    pub computed_padding: [f32; 4],
    pub computed_border: [f32; 4],
    /// A user-defined id for the element.
    pub id: Option<String>,
    /// The id of the component that this element belongs to.
    pub component_id: ComponentId,
    pub scrollbar_size: [f32; 2],
    
    pub computed_scroll_track: Rectangle,
    pub computed_scroll_thumb: Rectangle,
    pub(crate) max_scroll_y: f32
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
        x >= common_element_data.computed_x_transformed
            && x <= common_element_data.computed_x_transformed + common_element_data.computed_width
            && y >= common_element_data.computed_y_transformed
            && y <= common_element_data.computed_y_transformed + common_element_data.computed_height
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

    fn resolve_position(&mut self, x: f32, y: f32, result: &taffy::Layout) {
        match self.common_element_data().style.position {
            taffy::Position::Relative => {
                self.common_element_data_mut().computed_x = x + result.location.x;
                self.common_element_data_mut().computed_y = y + result.location.y;
            }
            taffy::Position::Absolute => {
                self.common_element_data_mut().computed_x = result.location.x;
                self.common_element_data_mut().computed_y = result.location.y;
            }
        }
    }

    fn finalize_scrollbar(&mut self, scroll_y: f32) {
        let common_element_data = self.common_element_data_mut();

        if common_element_data.style.overflow[0] != taffy::Overflow::Scroll {
            return;
        }

        let computed_x_transformed = common_element_data.computed_x_transformed;
        let computed_y_transformed = common_element_data.computed_y_transformed;

        let computed_width = common_element_data.computed_width;
        let computed_height = common_element_data.computed_height;
        
        let computed_content_height = common_element_data.computed_content_height;

        let border_top = common_element_data.computed_border[0];
        let border_right = common_element_data.computed_border[1];
        let border_bottom = common_element_data.computed_border[2];

        let client_height = computed_height - border_top - border_bottom;
        let scroll_height = computed_content_height - border_top;
        
        let scrolltrack_width = common_element_data.scrollbar_size[0];
        let scrolltrack_height = client_height;

        let max_scroll_y = (scroll_height - client_height).max(0.0);
        common_element_data.max_scroll_y = max_scroll_y;

        let visible_y = client_height / scroll_height;
        let scrollthumb_height = scrolltrack_height * visible_y;
        let remaining_height = scrolltrack_height - scrollthumb_height;
        let scrollthumb_offset = scroll_y / common_element_data.computed_scrollbar_height * remaining_height;

        common_element_data.computed_scroll_track = Rectangle::new(
            computed_x_transformed + computed_width - scrolltrack_width - border_right,
            computed_y_transformed + border_top,
            scrolltrack_width,
            scrolltrack_height,
        );

        let scrollthumb_width = scrolltrack_width;
        
        common_element_data.computed_scroll_thumb = Rectangle::new(
            computed_x_transformed + computed_width - scrolltrack_width - border_right,
            computed_y_transformed + border_top + scrollthumb_offset,
            scrollthumb_width,
            scrollthumb_height,
        )
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
        ComponentSpecification {
            component: ComponentOrElement::Element(element),
            key: None,
            props: None,
            children: vec![],
        }
    }
}

impl From<ComponentOrElement> for ComponentSpecification {
    fn from(element: ComponentOrElement) -> Self {
        ComponentSpecification {
            component: element,
            key: None,
            props: None,
            children: vec![],
        }
    }
}

impl<T: Element> From<T> for ComponentSpecification {
    fn from(element: T) -> Self {
        ComponentSpecification {
            component: ComponentOrElement::Element(element.into()),
            key: None,
            props: None,
            children: vec![],
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
