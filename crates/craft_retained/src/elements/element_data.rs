use std::cell::RefCell;
use std::rc::{Rc, Weak};
use smol_str::SmolStr;

use crate::app::{ELEMENTS, TAFFY_TREE};
use crate::elements::ElementInternals;
use crate::elements::element_id::create_unique_element_id;
use crate::elements::scrollable::{apply_scroll_layout, ScrollState};
use crate::events::{KeyboardInputHandler, PointerCaptureHandler, PointerEnterHandler, PointerEventHandler, PointerLeaveHandler, PointerUpdateHandler, ScrollHandler, SliderValueChangedHandler};
use crate::layout::layout_context::LayoutContext;
use crate::layout::layout::Layout;
use crate::style::{Overflow, Style};

/// Stores common data to most elements.
#[derive(Clone)]
pub struct ElementData {
    /// A cyclic weak pointer to the element.
    pub(crate) me: Weak<RefCell<dyn ElementInternals>>,

    /// The Element's parent.
    pub(crate) parent: Option<Weak<RefCell<dyn ElementInternals>>>,

    /// The style of the element.
    pub style: Box<Style>,

    /// Stores the layout data for an element.
    pub layout: Layout,

    /// The children of the element.
    pub children: Vec<Rc<RefCell<dyn ElementInternals>>>,

    /// A user-defined id for the element.
    pub id: Option<SmolStr>,

    /// A unique id for this element. Within a craft app the id will be unique even across windows.
    pub(crate) internal_id: u64,

    // Events:
    pub on_slider_value_changed: Vec<SliderValueChangedHandler>,
    pub on_pointer_enter: Vec<PointerEnterHandler>,
    pub on_pointer_leave: Vec<PointerLeaveHandler>,
    pub on_got_pointer_capture: Vec<PointerCaptureHandler>,
    pub on_lost_pointer_capture: Vec<PointerCaptureHandler>,
    pub on_pointer_button_down: Vec<PointerEventHandler>,
    pub on_pointer_button_up: Vec<PointerEventHandler>,
    pub on_pointer_moved: Vec<PointerUpdateHandler>,
    pub on_keyboard_input: Vec<KeyboardInputHandler>,
    pub on_scroll: Vec<ScrollHandler>,
}

impl ElementData {
    pub fn new(me: Weak<RefCell<dyn ElementInternals>>, is_scrollable: bool) -> Self {
        let default = Self {
            me,
            parent: None,
            style: Style::new(),
            layout: Layout::new(is_scrollable),
            children: Default::default(),
            id: None,
            internal_id: create_unique_element_id(),
            on_slider_value_changed: Vec::new(),
            on_pointer_enter: Vec::new(),
            on_pointer_leave: Vec::new(),
            on_got_pointer_capture: Vec::new(),
            on_lost_pointer_capture: Vec::new(),
            on_pointer_button_down: Vec::new(),
            on_pointer_button_up: Vec::new(),
            on_pointer_moved: Vec::new(),
            on_keyboard_input: Vec::new(),
            on_scroll: Vec::new(),
        };

        ELEMENTS.with_borrow_mut(|elements| {
            elements.insert_id(default.internal_id, default.me.clone());
        });

        default
    }

    /// Creates a new taffy node for this element with optional layout context.
    pub fn create_layout_node(&mut self, layout_context: Option<LayoutContext>) {
        TAFFY_TREE.with_borrow_mut(|taffy_tree| {
            let style = self.style.to_taffy_style();
            let node_id = if let Some(layout_context) = layout_context {
                taffy_tree.new_leaf_with_context(style, layout_context)
            } else {
                taffy_tree.new_leaf(style)
            };
            self.layout.taffy_node_id = Some(node_id);
        });
    }

    /// Computes the scrollbar's tack and thumb layout.
    pub(crate) fn apply_scroll(&mut self, layout: &taffy::Layout) {
       apply_scroll_layout(self, layout);
    }

    pub(crate) fn scroll(&self) -> ScrollState {
        self.layout.scroll_state
    }
}

impl ElementData {
    pub fn is_scrollable(&self) -> bool {
        self.style.get_overflow()[1] == Overflow::Scroll && self.layout.is_scrollable()
    }

    pub fn current_style(&self) -> &Style {
        &self.style
    }

    pub fn current_style_mut(&mut self) -> &mut Style {
        &mut self.style
    }

    pub fn current_style_mut_no_fallback<'a>(&self) -> Option<&'a mut Style> {
        None
    }
}
