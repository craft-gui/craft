use smallvec::SmallVec;
use smol_str::SmolStr;
use crate::components::{ComponentId, ComponentSpecification};
use crate::components::Props;
use crate::elements::element::ElementBoxed;
use crate::elements::element_states::ElementState;
use crate::events::event_handlers::EventHandlers;
use crate::layout::layout_item::LayoutItem;
use crate::style::Style;

#[derive(Clone, Default)]
pub struct ElementData {
    pub current_state: ElementState,

    /// The style of the element.
    pub style: Style,

    pub layout_item: LayoutItem,

    /// The style of the element when it is hovered.
    pub hover_style: Option<Style>,

    /// The style of the element when it is pressed.
    pub pressed_style: Option<Style>,

    /// The style of the element when it is disabled.
    pub disabled_style: Option<Style>,

    /// The style of the element when it is focused.
    pub focused_style: Option<Style>,

    /// The children of the element.
    pub children: SmallVec<[ElementBoxed; 4]>,

    /// A user-defined id for the element.
    pub id: Option<SmolStr>,

    /// The id of the component that this element belongs to.
    pub component_id: ComponentId,

    // Used for converting the element to a component specification.
    pub child_specs: Vec<ComponentSpecification>,
    pub(crate) key: Option<SmolStr>,
    pub(crate) props: Option<Props>,
    pub(crate) event_handlers: EventHandlers,
}

impl ElementData {
    pub fn is_scrollable(&self) -> bool {
        self.style.overflow()[1] == taffy::Overflow::Scroll
    }

    pub fn current_style_mut(&mut self) -> &mut Style {
        match self.current_state {
            ElementState::Normal => &mut self.style,
            ElementState::Hovered => {
                if let Some(ref mut hover_style) = self.hover_style {
                    hover_style
                } else {
                    self.hover_style = Some(self.style.clone());
                    self.hover_style.as_mut().unwrap()
                }
            }
            ElementState::Pressed => {
                if let Some(ref mut pressed_style) = self.pressed_style {
                    pressed_style
                } else {
                    self.pressed_style = Some(self.style.clone());
                    self.pressed_style.as_mut().unwrap()
                }
            }
            ElementState::Disabled => {
                if let Some(ref mut disabled_style) = self.disabled_style {
                    disabled_style
                } else {
                    self.disabled_style = Some(self.style.clone());
                    self.disabled_style.as_mut().unwrap()
                }
            }
            ElementState::Focused => {
                if let Some(ref mut focused_style) = self.focused_style {
                    focused_style
                } else {
                    self.focused_style = Some(self.style.clone());
                    self.focused_style.as_mut().unwrap()
                }
            }
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
            }
            ElementState::Pressed => {
                if let Some(ref pressed_style) = self.pressed_style {
                    pressed_style
                } else {
                    &self.style
                }
            }
            ElementState::Disabled => {
                if let Some(ref disabled_style) = self.disabled_style {
                    disabled_style
                } else {
                    &self.style
                }
            }
            ElementState::Focused => {
                if let Some(ref focused_style) = self.focused_style {
                    focused_style
                } else {
                    &self.style
                }
            }
        }
    }
}
