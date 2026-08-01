use std::cell::RefCell;
use std::rc::{Rc, Weak};

use smol_str::SmolStr;

use crate::accessibility::RetGuiAccessTree;
use crate::app::{ELEMENTS, TAFFY_TREE};
use crate::elements::element_id::create_unique_element_id;
use crate::elements::scrollable::{ScrollState, apply_scroll_layout};
use crate::elements::{ElementInternals, WindowInternal};
use crate::events::{CheckboxToggledHandler, DropdownItemSelectedHandler, KeyboardInputHandler, PointerCaptureHandler, PointerEnterHandler, PointerEventHandler, PointerLeaveHandler, PointerUpdateHandler, RadioValueChangedHandler, ScrollHandler, SliderValueChangedHandler, TextInputChangedHandler};
use crate::layout::layout::Layout;
use crate::layout::layout_context::LayoutContext;
use crate::style::{Overflow, Style};

/// Stores common data to most elements.
#[derive(Clone)]
pub struct ElementData {
    /// A cyclic weak pointer to the element.
    pub(crate) me: Weak<RefCell<dyn ElementInternals>>,

    /// The Element's parent.
    pub(crate) parent: Option<Weak<RefCell<dyn ElementInternals>>>,

    /// A pointer to the owning window.
    pub(crate) window: Option<Weak<RefCell<WindowInternal>>>,

    /// The style of the element.
    pub style: Style,

    /// Stores the layout data for an element.
    pub layout: Layout,

    /// The children of the element.
    pub children: Vec<Rc<RefCell<dyn ElementInternals>>>,

    /// A user-defined id for the element.
    pub id: Option<SmolStr>,

    /// A unique id for this element. Within a retgui app the id will be unique even across windows.
    pub(crate) internal_id: u64,

    pub(crate) access_tree: RetGuiAccessTree,
    pub(crate) access_key: Option<issho::AccessKey>,
    pub(crate) access_root: Option<issho::AccessKey>,
    pub(crate) access_scale_factor: f64,

    // Events:
    pub on_dropdown_item_selected: Vec<DropdownItemSelectedHandler>,
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
    pub on_radio_value_changed: Vec<RadioValueChangedHandler>,
    pub on_checkbox_toggled: Vec<CheckboxToggledHandler>,
    pub on_text_input_changed: Vec<TextInputChangedHandler>,
}

impl ElementData {
    pub fn new(me: Weak<RefCell<dyn ElementInternals>>, is_scrollable: bool) -> Self {
        Self::new_internal(me, is_scrollable, true)
    }

    pub(crate) fn new_pseudo(me: Weak<RefCell<dyn ElementInternals>>, is_scrollable: bool) -> Self {
        Self::new_internal(me, is_scrollable, false)
    }

    fn new_internal(
        me: Weak<RefCell<dyn ElementInternals>>,
        is_scrollable: bool,
        create_accessibility_node: bool,
    ) -> Self {
        let access_tree = crate::accessibility::access_tree();
        let (access_key, access_root) = if create_accessibility_node {
            let key = access_tree.insert_node(issho::AccessNode::new(), None);
            (Some(key), Some(key))
        } else {
            (None, None)
        };

        let default = Self {
            me,
            parent: None,
            window: None,
            style: Style::new(),
            layout: Layout::new(is_scrollable),
            children: Default::default(),
            id: None,
            internal_id: create_unique_element_id(),
            access_tree,
            access_key,
            access_root,
            access_scale_factor: 1.0,
            on_dropdown_item_selected: Vec::new(),
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
            on_radio_value_changed: Vec::new(),
            on_checkbox_toggled: Vec::new(),
            on_text_input_changed: Vec::new(),
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

    pub(crate) fn set_accessibility_role(&mut self, role: issho::Role) {
        if let Some(key) = self.access_key
            && let Some(mut node) = self.access_tree.get_node_mut(key)
        {
            node.set_role(role);
        }
    }

    pub(crate) fn set_accessibility_name(&mut self, name: impl Into<SmolStr>) {
        if let Some(key) = self.access_key {
            self.access_tree.set_name(key, name);
        }
    }

    pub(crate) fn set_accessibility_value(&mut self, value: impl Into<String>) {
        if let Some(key) = self.access_key {
            self.access_tree.set_value(key, value);
        }
    }

    pub(crate) fn set_accessibility_enabled(&mut self, enabled: bool) {
        if let Some(key) = self.access_key
            && let Some(mut node) = self.access_tree.get_node_mut(key)
        {
            node.set_enabled(enabled);
        }
    }

    pub(crate) fn set_accessibility_checked(&mut self, checked: bool) {
        if let Some(key) = self.access_key {
            self.access_tree.set_checked(key, checked);
        }
    }

    pub(crate) fn set_accessibility_toggle_action(&mut self, action: impl Fn() + 'static) {
        if let Some(key) = self.access_key
            && let Some(mut node) = self.access_tree.get_node_mut(key)
        {
            node.set_toggle_action(action);
        }
    }

    /// Applies the element's resolved padding box to its retained accessibility node.
    pub(crate) fn set_accessibility_bounds_from_layout(&mut self, scale_factor: f64) {
        self.access_scale_factor = scale_factor;
        let padding_box = self
            .layout
            .computed_box_transformed
            .padding_rectangle()
            .scale(scale_factor);

        if let Some(key) = self.access_key
            && let Some(mut node) = self.access_tree.get_node_mut(key)
        {
            node.set_bounding_rect(issho::AccessRect::new(
                padding_box.x as f64,
                padding_box.y as f64,
                padding_box.width as f64,
                padding_box.height as f64,
            ));
        }
    }

    pub fn apply_borders(&mut self, scale_factor: f64) {
        let current_style = self.style();
        let has_border = current_style.has_border();
        let border_radius = current_style.get_border_radius();
        let border_color = current_style.get_border_color();
        let box_shadows = current_style.get_box_shadows();
        self.layout.apply_borders(
            has_border,
            border_radius,
            scale_factor,
            border_color,
            box_shadows.to_vec(),
        );
    }

    /// Computes the scrollbar's tack and thumb layout.
    pub(crate) fn apply_scroll(&mut self, taffy_layout: &taffy::Layout) {
        apply_scroll_layout(&self.style, &mut self.layout, taffy_layout);
    }

    pub(crate) fn scroll(&self) -> ScrollState {
        self.layout.scroll_state
    }

    pub fn is_scrollable(&self) -> bool {
        self.style.get_overflow()[1] == Overflow::Scroll && self.layout.is_scrollable_layout()
    }

    pub fn style(&self) -> &Style {
        &self.style
    }

    pub fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }
}
