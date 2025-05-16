use crate::components::{ComponentId, ComponentSpecification};
use crate::components::{Event, Props};
use crate::elements::element::ElementBoxed;
use crate::elements::element_states::ElementState;
use crate::events::{KeyboardInput, MouseWheel, PointerButton, PointerMoved};
use crate::geometry::borders::ComputedBorderSpec;
use crate::geometry::{ElementBox, Rectangle, Size};
use crate::style::Style;
use std::any::Any;
use std::sync::Arc;
use taffy::NodeId;
use winit::event::{Ime, Modifiers};

pub(crate) type EventHandler = Arc<dyn Fn(&mut dyn Any, &mut dyn Any, &mut Event) + Send + Sync + 'static>;

pub(crate) type EventHandlerWithRef<Arg> = Arc<dyn Fn(&mut dyn Any, &mut dyn Any, &mut Event, &Arg) + Send + Sync + 'static>;

pub(crate) type EventHandlerCopy<Arg> = Arc<dyn Fn(&mut dyn Any, &mut dyn Any, &mut Event, Arg) + Send + Sync + 'static>;

#[derive(Clone, Default)]
pub struct ElementData {
    pub current_state: ElementState,

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
    pub children: Vec<ElementBoxed>,

    /// The taffy node id after this element is laid out.
    /// This may be None if this is a non-visual element like Font.
    pub taffy_node_id: Option<NodeId>,

    pub content_size: Size<f32>,
    // The computed values after transforms are applied.
    pub computed_box_transformed: ElementBox,
    // The computed values without any transforms applied to them.
    pub computed_box: ElementBox,

    /// A user-defined id for the element.
    pub id: Option<String>,

    /// The id of the component that this element belongs to.
    pub component_id: ComponentId,
    pub computed_scrollbar_size: Size<f32>,
    pub scrollbar_size: Size<f32>,
    pub computed_scroll_track: Rectangle,
    pub computed_scroll_thumb: Rectangle,
    pub(crate) max_scroll_y: f32,
    pub layout_order: u32,

    // Used for converting the element to a component specification.
    pub child_specs: Vec<ComponentSpecification>,
    pub(crate) key: Option<String>,
    pub(crate) props: Option<Props>,

    pub(crate) on_pointer_button: Option<EventHandlerWithRef<PointerButton>>,
    pub(crate) on_initialized: Option<EventHandler>,
    pub(crate) on_keyboard_input: Option<EventHandlerWithRef<KeyboardInput>>,
    pub(crate) on_pointer_move: Option<EventHandlerWithRef<PointerMoved>>,
    pub(crate) on_mouse_wheel: Option<EventHandlerWithRef<MouseWheel>>,
    pub(crate) on_modifiers_changed: Option<EventHandlerWithRef<Modifiers>>,
    pub(crate) on_ime: Option<EventHandlerWithRef<Ime>>,
    pub(crate) on_text_input_changed: Option<EventHandlerWithRef<str>>,

    pub(crate) on_dropdown_toggled: Option<EventHandlerCopy<bool>>,
    pub(crate) on_dropdown_item_selected: Option<EventHandlerCopy<usize>>,
    pub(crate) on_switch_toggled: Option<EventHandlerCopy<bool>>,
    pub(crate) on_slider_value_changed: Option<EventHandlerCopy<f64>>,
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
                    self.hover_style = Some(Box::new(self.style));
                    self.hover_style.as_mut().unwrap()
                }
            }
            ElementState::Pressed => {
                if let Some(ref mut pressed_style) = self.pressed_style {
                    pressed_style
                } else {
                    self.pressed_style = Some(Box::new(self.style));
                    self.pressed_style.as_mut().unwrap()
                }
            }
            ElementState::Disabled => {
                if let Some(ref mut disabled_style) = self.disabled_style {
                    disabled_style
                } else {
                    self.disabled_style = Some(Box::new(self.style));
                    self.disabled_style.as_mut().unwrap()
                }
            }
            ElementState::Focused => {
                if let Some(ref mut focused_style) = self.focused_style {
                    focused_style
                } else {
                    self.focused_style = Some(Box::new(self.style));
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
