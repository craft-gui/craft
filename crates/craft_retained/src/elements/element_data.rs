use std::cell::RefCell;
use std::rc::{Rc, Weak};

use craft_primitives::geometry::{Rectangle, Size};
use rustc_hash::FxHashMap;
use smol_str::SmolStr;
use taffy::{Layout, Overflow};

use crate::animations::animation::ActiveAnimation;
use crate::app::{ELEMENTS, TAFFY_TREE};
use crate::elements::Element;
use crate::elements::element_id::create_unique_element_id;
use crate::elements::scroll_state::ScrollState;
use crate::events::{KeyboardInputHandler, PointerCaptureHandler, PointerEnterHandler, PointerEventHandler, PointerLeaveHandler, PointerUpdateHandler, SliderValueChangedHandler};
use crate::layout::layout_context::LayoutContext;
use crate::layout::layout_item::LayoutItem;
use crate::style::Style;

/// Stores common data to most elements.
#[derive(Clone)]
pub struct ElementData {
    /// A cyclic weak pointer to the element.
    pub(crate) me: Weak<RefCell<dyn Element>>,

    /// The Element's parent.
    pub(crate) parent: Option<Weak<RefCell<dyn Element>>>,

    /// The style of the element.
    pub style: Style,

    /// Stores the layout data for an element.
    pub layout_item: LayoutItem,

    /// The children of the element.
    pub children: Vec<Rc<RefCell<dyn Element>>>,

    /// A user-defined id for the element.
    pub id: Option<SmolStr>,

    /// A unique id for this element. Within a craft app the id will be unique even across windows.
    pub(crate) internal_id: u64,

    pub(crate) animations: Option<FxHashMap<SmolStr, ActiveAnimation>>,

    /// Scrollbar state for elements that may have a scrollbar.
    pub(super) scroll_state: Option<ScrollState>,

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
}

impl ElementData {
    pub fn new(me: Weak<RefCell<dyn Element>>, scrollable: bool) -> Self {
        let mut default = Self {
            me,
            parent: None,
            style: Default::default(),
            layout_item: Default::default(),
            children: Default::default(),
            id: None,
            internal_id: create_unique_element_id(),
            animations: None,
            scroll_state: None,

            on_slider_value_changed: Vec::new(),
            on_pointer_enter: Vec::new(),
            on_pointer_leave: Vec::new(),
            on_got_pointer_capture: Vec::new(),
            on_lost_pointer_capture: Vec::new(),
            on_pointer_button_down: Vec::new(),
            on_pointer_button_up: Vec::new(),
            on_pointer_moved: Vec::new(),
            on_keyboard_input: Vec::new(),
        };

        // Create scroll state if needed.
        if scrollable {
            default.scroll_state = Some(ScrollState::default());
        }

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
            self.layout_item.taffy_node_id = Some(node_id);
        });
    }

    /// Computes the scrollbar's tack and thumb layout.
    pub(crate) fn apply_scroll(&mut self, layout: &Layout) {
        self.layout_item.scrollbar_size = Size::new(layout.scrollbar_size.width, layout.scrollbar_size.height);
        self.layout_item.computed_scrollbar_size = Size::new(layout.scroll_width(), layout.scroll_height());

        if let Some(state) = &mut self.scroll_state {
            if self.style.overflow()[1] != Overflow::Scroll {
                return;
            }

            let box_transformed = self.layout_item.computed_box_transformed;

            // Client Height = padding box height.
            let client_height = box_transformed.padding_rectangle().height;

            let mut content_height = self.layout_item.content_size.height;
            // Taffy is adding the top border and padding height to the content size.
            content_height -= box_transformed.border.top;
            content_height -= box_transformed.padding.top;

            // Content Size = overflowed content size + padding
            // Scroll Height = Content Size
            let scroll_height =
                (content_height + box_transformed.padding.bottom + box_transformed.padding.top).max(1.0);
            let scroll_track_width = self.layout_item.scrollbar_size.width;

            // The scroll track height is the height of the padding box.
            let scroll_track_height = client_height;

            let max_scroll_y = (scroll_height - client_height).max(0.0);
            self.layout_item.max_scroll_y = max_scroll_y;

            self.layout_item.computed_scroll_track = Rectangle::new(
                box_transformed.padding_rectangle().right() - scroll_track_width,
                box_transformed.padding_rectangle().top(),
                scroll_track_width,
                scroll_track_height,
            );

            let visible_y = (client_height / scroll_height).clamp(0.0, 1.0);
            let scroll_thumb_height = scroll_track_height * visible_y;
            let scroll_thumb_height = scroll_thumb_height.max(15.0);
            let remaining_height = scroll_track_height - scroll_thumb_height;
            let scroll_thumb_offset = if max_scroll_y != 0.0 {
                state.scroll_y() / max_scroll_y * remaining_height
            } else {
                0.0
            };

            let thumb_margin = self.style.scrollbar_thumb_margin();
            let scroll_thumb_width = scroll_track_width - (thumb_margin.left + thumb_margin.right);
            let scroll_thumb_height = (scroll_thumb_height - (thumb_margin.top + thumb_margin.bottom)).max(0.0);

            self.layout_item.computed_scroll_thumb = self.layout_item.computed_scroll_track;
            self.layout_item.computed_scroll_thumb.x += thumb_margin.left;
            self.layout_item.computed_scroll_thumb.y += scroll_thumb_offset + thumb_margin.top;
            self.layout_item.computed_scroll_thumb.width = scroll_thumb_width;
            self.layout_item.computed_scroll_thumb.height = scroll_thumb_height;
        }
    }

    pub(crate) fn scroll(&self) -> Option<&ScrollState> {
        self.scroll_state.as_ref()
    }
}

impl ElementData {
    pub fn is_scrollable(&self) -> bool {
        self.style.overflow()[1] == taffy::Overflow::Scroll
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
