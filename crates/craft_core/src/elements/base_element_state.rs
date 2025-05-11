use crate::elements::element_states::ElementState;
use std::collections::HashMap;
use crate::elements::element_data::ElementData;
use crate::style::Style;

#[derive(Debug, Default, Clone)]
pub struct BaseElementState {
    pub(crate) hovered: bool,
    pub(crate) active: bool,
    #[allow(dead_code)]
    pub(crate) current_state: ElementState,
    /// Whether this element should receive pointer events regardless of hit testing.
    /// Useful for scroll thumbs.
    pub(crate) pointer_capture: HashMap<i64, bool>,
}

impl BaseElementState {
    
    pub fn current_style<'a>(&self, element_data: &'a ElementData) -> &'a Style {
        if self.active {
            if let Some(pressed_style) = &element_data.pressed_style {
                return pressed_style;
            }
        }
        if self.hovered {
            if let Some(hover_style) = &element_data.hover_style {
                return hover_style;
            }
        }
        &element_data.style
    }
    
}

// HACK: Remove this and all usages when pointer capture per device works.
pub(crate) const DUMMY_DEVICE_ID: i64 = -1;
