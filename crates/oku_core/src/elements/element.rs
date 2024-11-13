use crate::components::component::{ComponentId, ComponentOrElement, ComponentSpecification};
use crate::components::UpdateResult;
use crate::elements::layout_context::LayoutContext;
use crate::engine::events::OkuEvent;
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
    pub computed_padding: [f32; 4],
    /// A user-defined id for the element.
    pub id: Option<String>,
    /// The id of the component that this element belongs to.
    pub component_id: ComponentId,
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

    fn on_event(&self, event: OkuEvent, element_state: &mut StateStore, font_system: &mut FontSystem) -> UpdateResult {
        UpdateResult::default()
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
            println!("{}{}, Parent Component Id: {}", prefix, element.name(), element.component_id());
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
