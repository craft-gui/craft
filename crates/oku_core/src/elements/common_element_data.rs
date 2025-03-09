use taffy::NodeId;
use crate::components::{ComponentId, ComponentSpecification};
use crate::components::props::Props;
use crate::elements::element::ElementBox;
use crate::elements::element_states::ElementState;
use crate::geometry::borders::ComputedBorderSpec;
use crate::geometry::{ElementRectangle, Rectangle, Size};
use crate::style::Style;

#[derive(Clone, Debug, Default)]
pub struct CommonElementData {
    pub(crate) current_state: ElementState,

    pub computed_border: ComputedBorderSpec,

    /// The style of the element.
    pub style: Style,

    /// The style of the element when it is hovered.
    pub hover_style: Option<Box<Style>>,

    /// The style of the element when it is pressed.
    pub pressed_style: Option<Box<Style>>,

    /// The style of the element when it is disabled.
    pub disabled_style: Option<Box<Style>>,

    /// The style of the element when it is focused.
    pub focused_style: Option<Box<Style>>,

    /// The children of the element.
    pub(crate) children: Vec<ElementBox>,

    /// The taffy node id after this element is laid out.
    /// This may be None if this is a non-visual element like Font.
    pub(crate) taffy_node_id: Option<NodeId>,

    pub computed_border_rectangle_overflow_size: Size,
    // The computed values after transforms are applied.
    pub computed_layered_rectangle_transformed: ElementRectangle,
    // The computed values without any transforms applied to them.
    pub computed_layered_rectangle: ElementRectangle,

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

impl CommonElementData {

    pub(crate) fn current_style_mut(&mut self) -> &mut Style {
        match self.current_state {
            ElementState::Normal => &mut self.style,
            ElementState::Hovered => {
                if let Some(ref mut hover_style) = self.hover_style {
                    hover_style
                } else {
                    self.hover_style = Some(Box::new(self.style));
                    self.hover_style.as_mut().unwrap()
                }
            },
            ElementState::Pressed => {
                if let Some(ref mut pressed_style) = self.pressed_style {
                    pressed_style
                } else {
                    self.pressed_style = Some(Box::new(self.style));
                    self.pressed_style.as_mut().unwrap()
                }
            },
            ElementState::Disabled => {
                if let Some(ref mut disabled_style) = self.disabled_style {
                    disabled_style
                } else {
                    self.disabled_style = Some(Box::new(self.style));
                    self.disabled_style.as_mut().unwrap()
                }
            },
            ElementState::Focused => {
                if let Some(ref mut focused_style) = self.focused_style {
                    focused_style
                } else {
                    self.focused_style = Some(Box::new(self.style));
                    self.focused_style.as_mut().unwrap()
                }
            },
        }
    }
    
    pub fn current_style(&self) -> &Style {
        match self.current_state {
            ElementState::Normal => &self.style,
            ElementState::Hovered => {
                if let Some(ref hover_style) = self.hover_style {
                    hover_style
                } else {
                    &self.style
                }
            },
            ElementState::Pressed => {
                if let Some(ref pressed_style) = self.pressed_style {
                    pressed_style
                } else {
                    &self.style
                }
            },
            ElementState::Disabled => {
                if let Some(ref disabled_style) = self.disabled_style {
                    disabled_style
                } else {
                    &self.style
                }
            },
            ElementState::Focused => {
                if let Some(ref focused_style) = self.focused_style {
                    focused_style
                } else {
                    &self.style
                }
            },
        }
    }
    
}