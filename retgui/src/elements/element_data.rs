use std::cell::RefCell;
use std::rc::{Rc, Weak};

use smol_str::SmolStr;

use smallvec::SmallVec;

use crate::Color;
use crate::accessibility::RetGuiAccessTree;
use crate::app::{ELEMENTS, GUMMY_TREE};
use crate::elements::element_id::create_unique_element_id;
use crate::elements::scrollable::{ScrollState, apply_scroll_layout};
use crate::elements::{ElementInternals, WindowInternal};
use crate::events::EventCallback;
use crate::geometry::TrblRectangle;
use crate::layout::layout::Layout;
use crate::layout::layout_context::LayoutContext;
use crate::style::{Overflow, Style, Unit};

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
    pub(crate) applied_scale_factor: f64,

    pub event_callbacks: SmallVec<[EventCallback; 1]>,

    pub(crate) unfocused_outline_color: Option<TrblRectangle<Color>>,
    pub(crate) unfocused_outline_width: Option<TrblRectangle<Unit>>,
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
            let mut node = issho::AccessNode::new();
            node.set_context(me.clone());
            let key = access_tree.insert_node(node, None);
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
            applied_scale_factor: 1.0,
            event_callbacks: SmallVec::new(),
            unfocused_outline_color: None,
            unfocused_outline_width: None,
        };

        if create_accessibility_node {
            ELEMENTS.with_borrow_mut(|elements| {
                elements.insert_id(default.internal_id, default.me.clone());
            });
        }

        default
    }

    /// Creates a new gummy node for this element with optional layout context.
    pub fn create_layout_node(&mut self, layout_context: Option<LayoutContext>) {
        GUMMY_TREE.with_borrow_mut(|gummy_tree| {
            let style = self.style.to_gummy_style();
            let node_id = if let Some(layout_context) = layout_context {
                gummy_tree.new_leaf_with_context(style, layout_context)
            } else {
                gummy_tree.new_leaf(style)
            };
            self.layout.gummy_node_id = Some(node_id);
            gummy_tree.register_owner(node_id, self.internal_id, self.me.clone());
        });
    }

    pub(crate) fn set_accessibility_role(&mut self, role: issho::Role) {
        if let Some(key) = self.access_key
            && let Some(mut node) = self.access_tree.get_node_mut(key)
        {
            node.set_role(role);
        }
    }

    pub(crate) fn set_selectable(&mut self, selectable: bool) {
        let text_selectable = if selectable {
            issho::SupportedTextSelection::Single
        } else {
            issho::SupportedTextSelection::None
        };
        if let Some(key) = self.access_key
            && let Some(mut node) = self.access_tree.get_node_mut(key)
        {
            node.set_text_supported_text_selection(text_selectable);
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

    pub(crate) fn set_accessibility_selection_data(&mut self, selection_data: Option<issho::SelectionData>) {
        if let Some(key) = self.access_key
            && let Some(mut node) = self.access_tree.get_node_mut(key)
        {
            node.set_selection_data(selection_data);
        }
    }

    /// Applies the element's resolved padding box to its retained accessibility node.
    pub(crate) fn set_accessibility_bounds_from_layout(&mut self, scale_factor: f64) {
        self.access_scale_factor = scale_factor;
        let padding_box = self.layout.world_box().padding_rectangle().scale(scale_factor);

        if let Some(key) = self.access_key
            && let Some(mut node) = self.access_tree.get_node_mut(key)
        {
            let bounds = issho::AccessRect::new(
                padding_box.x as f64,
                padding_box.y as f64,
                padding_box.width as f64,
                padding_box.height as f64,
            );
            if node.bounding_rect() != bounds {
                node.set_bounding_rect(bounds);
            }
        }
    }

    pub fn apply_borders(&mut self, scale_factor: f64) {
        let current_style = self.style();
        let has_border = current_style.has_border();
        let border_radius = current_style.get_border_radius();
        let border_color = current_style.get_border_color();
        let outline_width = current_style.get_outline_width_px();
        let box_shadows = current_style.get_box_shadows();
        self.layout.apply_borders(
            has_border,
            border_radius,
            scale_factor,
            border_color,
            outline_width,
            box_shadows.to_vec(),
        );
    }

    /// Computes the scrollbar's tack and thumb layout.
    pub(crate) fn apply_scroll(&mut self, gummy_layout: &gummy::Layout) {
        apply_scroll_layout(&self.style, &mut self.layout, gummy_layout);
        self.apply_accessibility_scroll_data();
    }

    pub(crate) fn apply_accessibility_scroll_data(&mut self) {
        let scroll_data = if self.is_scrollable() {
            let viewport_height = self.layout.local_box().padding_rectangle().height.max(0.0);
            let content_height = viewport_height + self.layout.max_scroll_y;
            let vertical_size = if content_height > 0.0 {
                f64::from(viewport_height / content_height * 100.0)
            } else {
                100.0
            };
            let vertical_percentage = (self.layout.max_scroll_y > 0.0)
                .then(|| f64::from(self.layout.scroll_state.scroll_y() / self.layout.max_scroll_y * 100.0));

            issho::ScrollData::ScrollContainer(issho::ScrollContainerData {
                vertical_size,
                horizontal_size: 100.0,
                horizontal_percentage: None,
                vertical_percentage,
            })
        } else {
            issho::ScrollData::None
        };

        if let Some(key) = self.access_key
            && let Some(mut node) = self.access_tree.get_node_mut(key)
            && node.scroll_data() != &scroll_data
        {
            node.set_scroll_data(scroll_data);
        }
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
