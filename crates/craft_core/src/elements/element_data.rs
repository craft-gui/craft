use crate::components::{ComponentId, ComponentSpecification};
use crate::components::{Event, Props};
use crate::elements::element::ElementBoxed;
use crate::elements::element_states::ElementState;
use crate::layout::layout_item::LayoutItem;
use crate::style::Style;
use std::any::Any;
use std::sync::Arc;
use ui_events::keyboard::KeyboardEvent;
use ui_events::pointer::{PointerButtonUpdate, PointerScrollUpdate, PointerUpdate};
use winit::event::{Ime, Modifiers};

pub(crate) type EventHandler = Arc<dyn Fn(&mut dyn Any, &mut dyn Any, &mut Event) + Send + Sync + 'static>;

pub(crate) type EventHandlerWithRef<Arg> =
    Arc<dyn Fn(&mut dyn Any, &mut dyn Any, &mut Event, &Arg) + Send + Sync + 'static>;

pub(crate) type EventHandlerCopy<Arg> =
    Arc<dyn Fn(&mut dyn Any, &mut dyn Any, &mut Event, Arg) + Send + Sync + 'static>;

#[derive(Clone, Default)]
pub struct ElementData {
    pub current_state: ElementState,

    /// The style of the element.
    pub style: Style,

    pub layout_item: LayoutItem,

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

    /// A user-defined id for the element.
    pub id: Option<String>,

    /// The id of the component that this element belongs to.
    pub component_id: ComponentId,

    // Used for converting the element to a component specification.
    pub child_specs: Vec<ComponentSpecification>,
    pub(crate) key: Option<String>,
    pub(crate) props: Option<Props>,

    pub(crate) on_pointer_button_up: Option<EventHandlerWithRef<PointerButtonUpdate>>,
    pub(crate) on_pointer_button_down: Option<EventHandlerWithRef<PointerButtonUpdate>>,
    pub(crate) on_initialized: Option<EventHandler>,
    pub(crate) on_keyboard_input: Option<EventHandlerWithRef<KeyboardEvent>>,
    pub(crate) on_pointer_move: Option<EventHandlerWithRef<PointerUpdate>>,
    pub(crate) on_pointer_scroll: Option<EventHandlerWithRef<PointerScrollUpdate>>,
    pub(crate) on_modifiers_changed: Option<EventHandlerWithRef<Modifiers>>,
    pub(crate) on_ime: Option<EventHandlerWithRef<Ime>>,
    pub(crate) on_text_input_changed: Option<EventHandlerWithRef<str>>,
    pub(crate) on_link_clicked: Option<EventHandlerWithRef<str>>,

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
