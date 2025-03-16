//! A simple plain text editor and related types.
use core::{
    cmp::PartialEq,
    default::Default,
    fmt::{Debug, Display},
    ops::Range,
};

//#[cfg(feature = "accesskit")]
// use parley::layout::LayoutAccessibility;
//#[cfg(feature = "accesskit")]
// use accesskit::{Node, NodeId, TreeUpdate};
use crate::elements::text_input::driver::PlainEditorDriver;
use parley::{
    Affinity, Alignment, AlignmentOptions, Brush, Cursor, FontContext, Layout, LayoutContext, Rect, Selection,
    StyleProperty, StyleSet,
};

/// A string which is potentially discontiguous in memory.
///
/// This is returned by [`PlainEditor::text`], as the IME preedit
/// area needs to be efficiently excluded from its return value.
#[derive(Debug, Clone, Copy)]
pub struct SplitString<'source>([&'source str; 2]);

impl PartialEq<&'_ str> for SplitString<'_> {
    fn eq(&self, other: &&'_ str) -> bool {
        let [a, b] = self.0;
        let mid = a.len();
        // When our MSRV is 1.80 or above, use split_at_checked instead.
        // is_char_boundary checks bounds
        let (a_1, b_1) = if other.is_char_boundary(mid) {
            other.split_at(mid)
        } else {
            return false;
        };

        a_1 == a && b_1 == b
    }
}
// We intentionally choose not to:
// impl PartialEq<Self> for SplitString<'_> {}
// for simplicity, as the impl wouldn't be useful and is non-trivial

impl Display for SplitString<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let [a, b] = self.0;
        write!(f, "{a}{b}")
    }
}

/// Iterate through the source strings.
impl<'source> IntoIterator for SplitString<'source> {
    type Item = &'source str;
    type IntoIter = <[&'source str; 2] as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// Basic plain text editor with a single style applied to the entire text.
///
/// Internally, this is a wrapper around a string buffer and its corresponding [`Layout`],
/// which is kept up-to-date as needed.
/// This layout is invalidated by a number.
#[derive(Clone)]
pub struct PlainEditor<T>
where
    T: Brush + Clone + Debug + PartialEq + Default,
{
    pub(crate) layout: Layout<T>,
    pub(crate) buffer: String,
    pub(crate) default_style: StyleSet<T>,
    //#[cfg(feature = "accesskit")]
    // pub(crate) layout_access: LayoutAccessibility,
    pub(crate) selection: Selection,
    /// Byte offsets of IME composing preedit text in the text buffer.
    /// `None` if the IME is not currently composing.
    pub(crate) compose: Option<Range<usize>>,
    /// Whether the cursor should be shown. The IME can request to hide the cursor.
    pub(crate) show_cursor: bool,
    pub(crate) width: Option<f32>,
    pub(crate) scale: f32,
    // Simple tracking of when the layout needs to be updated
    // before it can be used for `Selection` calculations or
    // for drawing.
    // Not all operations on `PlainEditor` need to operate on a
    // clean layout, and not all operations trigger a layout.
    pub(crate) layout_dirty: bool,
    // TODO: We could avoid redoing the full text layout if only
    // linebreaking or alignment were changed.
    // linebreak_dirty: bool,
    // alignment_dirty: bool,
    pub(crate) alignment: Alignment,
}

impl<T> PlainEditor<T>
where
    T: Brush,
{
    /// Create a new editor, with default font size `font_size`.
    pub fn new(font_size: f32) -> Self {
        Self {
            default_style: StyleSet::new(font_size),
            buffer: Default::default(),
            layout: Default::default(),
            //#[cfg(feature = "accesskit")]
            //layout_access: Default::default(),
            selection: Default::default(),
            compose: None,
            show_cursor: true,
            width: None,
            scale: 1.0,
            layout_dirty: true,
            alignment: Alignment::Start,
        }
    }
}

impl<T> PlainEditor<T>
where
    T: Brush + Clone + Debug + PartialEq + Default,
{
    /// Run a series of [`PlainEditorDriver`] methods.
    ///
    /// This type is only used to simplify methods which require both
    /// the editor and the provided contexts.
    pub fn driver<'drv>(
        &'drv mut self,
        font_cx: &'drv mut FontContext,
        layout_cx: &'drv mut LayoutContext<T>,
    ) -> PlainEditorDriver<'drv, T> {
        PlainEditorDriver {
            editor: self,
            font_cx,
            layout_cx,
        }
    }

    /// Get rectangles representing the selected portions of text.
    pub fn selection_geometry(&self) -> Vec<Rect> {
        // We do not check `self.show_cursor` here, as the IME handling code collapses the
        // selection to a caret in that case.
        self.selection.geometry(&self.layout)
    }

    /// Get a rectangle representing the current caret cursor position.
    ///
    /// There is not always a caret. For example, the IME may have indicated the caret should be
    /// hidden.
    pub fn cursor_geometry(&self, size: f32) -> Option<Rect> {
        self.show_cursor.then(|| self.selection.focus().geometry(&self.layout, size))
    }

    /// Replace the whole text buffer.
    pub fn set_text(&mut self, is: &str) {
        assert!(!self.is_composing());

        self.buffer.clear();
        self.buffer.push_str(is);
        self.layout_dirty = true;
    }

    /// Set the width of the layout.
    pub fn set_width(&mut self, width: Option<f32>) {
        self.width = width;
        self.layout_dirty = true;
    }

    /// Set the scale for the layout.
    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
        self.layout_dirty = true;
    }

    /// Modify the styles provided for this editor.
    pub fn edit_styles(&mut self) -> &mut StyleSet<T> {
        self.layout_dirty = true;
        &mut self.default_style
    }

    /// Whether the editor is currently in IME composing mode.
    pub fn is_composing(&self) -> bool {
        self.compose.is_some()
    }

    /// Get the full read-only details from the layout, which will be updated if necessary.
    ///
    /// If the required contexts are not available, then [`refresh_layout`](Self::refresh_layout) can
    /// be called in a scope when they are available, and [`try_layout`](Self::try_layout) can
    /// be used instead.
    pub fn layout(&mut self, font_cx: &mut FontContext, layout_cx: &mut LayoutContext<T>) -> &Layout<T> {
        self.refresh_layout(font_cx, layout_cx);
        &self.layout
    }

    /// Update the layout if it is dirty.
    ///
    /// This should only be used alongside [`try_layout`](Self::try_layout)
    /// or [`try_accessibility`](Self::try_accessibility), if those will be
    /// called in a scope where the contexts are not available.
    pub fn refresh_layout(&mut self, font_cx: &mut FontContext, layout_cx: &mut LayoutContext<T>) {
        if self.layout_dirty {
            self.update_layout(font_cx, layout_cx);
        }
    }

    // --- MARK: Internal Helpers ---
    /// Make a cursor at a given byte index.
    pub(crate) fn cursor_at(&self, index: usize) -> Cursor {
        // TODO: Do we need to be non-dirty?
        // FIXME: `Selection` should make this easier
        if index >= self.buffer.len() {
            Cursor::from_byte_index(&self.layout, self.buffer.len(), Affinity::Upstream)
        } else {
            Cursor::from_byte_index(&self.layout, index, Affinity::Downstream)
        }
    }

    pub(crate) fn replace_selection(&mut self, font_cx: &mut FontContext, layout_cx: &mut LayoutContext<T>, s: &str) {
        let range = self.selection.text_range();
        let start = range.start;
        if self.selection.is_collapsed() {
            self.buffer.insert_str(start, s);
        } else {
            self.buffer.replace_range(range, s);
        }

        self.update_layout(font_cx, layout_cx);
        let new_index = start.saturating_add(s.len());
        let affinity = if s.ends_with("\n") { Affinity::Downstream } else { Affinity::Upstream };
        self.set_selection(Cursor::from_byte_index(&self.layout, new_index, affinity).into());
    }

    /// Update the selection, and nudge the `Generation` if something other than `h_pos` changed.
    pub(crate) fn set_selection(&mut self, new_sel: Selection) {
        // This debug code is quite useful when diagnosing selection problems.
        self.selection = new_sel;
    }
    /// Update the layout.
    pub(crate) fn update_layout(&mut self, font_cx: &mut FontContext, layout_cx: &mut LayoutContext<T>) {
        let mut builder = layout_cx.ranged_builder(font_cx, &self.buffer, self.scale);
        for prop in self.default_style.inner().values() {
            builder.push_default(prop.to_owned());
        }
        if let Some(preedit_range) = &self.compose {
            builder.push(StyleProperty::Underline(true), preedit_range.clone());
        }
        self.layout = builder.build(&self.buffer);
        self.layout.break_all_lines(self.width);
        self.layout.align(self.width, self.alignment, AlignmentOptions::default());
        self.selection = self.selection.refresh(&self.layout);
        self.layout_dirty = false;
    }

    //#[cfg(feature = "accesskit")]
    // /// Perform an accessibility update, assuming that the layout is valid.
    // ///
    // /// The wrapper [`accessibility`](PlainEditorDriver::accessibility) on the driver type should
    // /// be preferred.
    // ///
    // /// You should always call [`refresh_layout`](Self::refresh_layout) before using this method,
    // /// with no other modifying method calls in between.
    //pub(crate) fn accessibility_unchecked(
    //    &mut self,
    //    update: &mut TreeUpdate,
    //    node: &mut Node,
    //    next_node_id: impl FnMut() -> NodeId,
    //    x_offset: f64,
    //    y_offset: f64,
    //) {
    //    self.layout_access.build_nodes(
    //        &self.buffer,
    //        &self.layout,
    //        update,
    //        node,
    //        next_node_id,
    //        x_offset,
    //        y_offset,
    //    );
    //    if self.show_cursor {
    //        if let Some(selection) = self
    //            .selection
    //            .to_access_selection(&self.layout, &self.layout_access)
    //        {
    //            node.set_text_selection(selection);
    //        }
    //    } else {
    //        node.clear_text_selection();
    //    }
    //    node.add_action(accesskit::Action::SetTextSelection);
    //}
}
