use craft_primitives::geometry::Point;

/// Stores state for elements with a scrollbar.
#[derive(Debug, Clone, Default, Copy)]
pub struct ScrollState {
    /// The total amount of vertical scroll.
    scroll_y: f32,

    /// Where the scrollbar was clicked.
    pub(crate) scroll_click: Option<Point>,

    // True if the scroll changes are new.
    is_new: bool,
}

impl ScrollState {
    /// Returns the total amount of vertical scroll.
    pub fn scroll_y(&self) -> f32 {
        self.scroll_y
    }

    pub fn mark_old(&mut self) {
        self.is_new = false;
    }

    pub fn is_new(&self) -> bool {
        self.is_new
    }

    /// Sets the total amount of vertical scroll.
    ///
    /// # Panics
    ///
    /// This function will panic if `scroll_y` is less than zero.
    pub fn set_scroll_y(&mut self, scroll_y: f32) {
        if self.scroll_y < 0.0 {
            panic!("Scroll cannot be negative.");
        }
        self.is_new = true;
        self.scroll_y = scroll_y;
    }
}
