/// A structure representing the size of a 2D object.
#[derive(Copy, Clone, Debug, Default)]
pub struct Size {
    /// The width of the object.
    pub(crate) width: f32,
    /// The height of the object.
    pub(crate) height: f32,
}

impl Size {
    /// Creates a new `Size` with the given width and height.
    ///
    /// # Arguments
    ///
    /// * `width` - A float representing the width of the object.
    /// * `height` - A float representing the height of the object.
    ///
    /// # Returns
    ///
    /// A `Size` instance with the specified width and height.
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

impl From<taffy::Size<f32>> for Size {
    /// Converts a `taffy::Size<f32>` to a `Size`.
    ///
    /// # Arguments
    ///
    /// * `size` - A `taffy::Size<f32>` instance to convert.
    ///
    /// # Returns
    ///
    /// A `Size` instance with the same width and height as the input.
    fn from(size: taffy::Size<f32>) -> Self {
        Self::new(size.width, size.height)
    }
}