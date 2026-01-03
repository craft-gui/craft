/*use kurbo::Point;
use winit::dpi::{LogicalPosition, LogicalSize, PhysicalSize};
use winit::window::{Cursor, Window};

#[derive(Debug, Clone)]
/// User-level API to get and set common window properties.
/// All values are in logical pixels.
pub struct WindowContext {
    pub(crate) scale_factor: f64,
    pub(crate) zoom_factor: f64,
    pub(crate) window_size: PhysicalSize<u32>,
    pub(crate) mouse_position: Option<Point>,
    pub(crate) cursor: Option<Cursor>,

    requested_window_width: Option<f32>,
    requested_window_height: Option<f32>,
    requested_mouse_position_x: Option<f32>,
    requested_mouse_position_y: Option<f32>,
    requested_cursor: Option<Cursor>,
}

impl WindowContext {
    pub(crate) fn apply_requests(&self, window: &Window) {
        if let Some(requested_cursor) = &self.requested_cursor {
            window.set_cursor(requested_cursor.clone());
        };

        if let Some(requested_window_width) = self.requested_window_width {
            let _ = window.request_inner_size(winit::dpi::Size::Logical(LogicalSize::new(
                requested_window_width as f64,
                self.window_size.height as f64,
            )));
        };

        if let Some(requested_window_height) = self.requested_window_height {
            let _ = window.request_inner_size(winit::dpi::Size::Logical(LogicalSize::new(
                self.window_size.width as f64,
                requested_window_height as f64,
            )));
        };

        if let Some(requested_mouse_position_x) = self.requested_mouse_position_x {
            let mouse_y = self.requested_mouse_position_y.unwrap_or_default() as f64;
            let _ = window.set_cursor_position(winit::dpi::Position::Logical(LogicalPosition::new(
                requested_mouse_position_x as f64,
                mouse_y,
            )));
        };

        if let Some(requested_mouse_position_y) = self.requested_mouse_position_y {
            let mouse_x = self.requested_mouse_position_x.unwrap_or_default() as f64;
            let _ = window.set_cursor_position(winit::dpi::Position::Logical(LogicalPosition::new(
                mouse_x,
                requested_mouse_position_y as f64,
            )));
        };
    }

    pub(crate) fn zoom_in(&mut self) {
        self.zoom_factor += 0.01;
    }

    pub(crate) fn zoom_out(&mut self) {
        self.zoom_factor = (self.zoom_factor - 0.01).max(1.0);
    }
}

impl WindowContext {
    pub(crate) fn new() -> WindowContext {
        Self {
            scale_factor: 1.0,
            zoom_factor: 1.0,
            window_size: Default::default(),
            mouse_position: None,
            cursor: None,
            requested_window_width: None,
            requested_window_height: None,
            requested_mouse_position_x: None,
            requested_mouse_position_y: None,
            requested_cursor: None,
        }
    }

    pub fn cursor(&self) -> Option<&Cursor> {
        self.cursor.as_ref()
    }

    pub fn window_width(&self) -> f32 {
        self.window_size.to_logical(self.effective_scale_factor()).width
    }
    pub fn window_height(&self) -> f32 {
        self.window_size.to_logical(self.effective_scale_factor()).height
    }

    pub fn window_size(&self) -> LogicalSize<f32> {
        self.window_size.to_logical(self.effective_scale_factor())
    }

    pub fn mouse_position_x(&self) -> Option<f32> {
        self.mouse_position.map(|pos| pos.x as f32)
    }

    pub fn mouse_position_y(&self) -> Option<f32> {
        self.mouse_position.map(|pos| pos.y as f32)
    }

    pub fn set_window_width(&mut self, width: f32) {
        self.requested_window_width = Some(width);
    }

    pub fn set_window_height(&mut self, height: f32) {
        self.requested_window_height = Some(height);
    }

    pub fn set_mouse_position_x(&mut self, x: f32) {
        self.requested_mouse_position_x = Some(x);
    }

    pub fn set_mouse_position_y(&mut self, y: f32) {
        self.requested_mouse_position_y = Some(y);
    }

    pub fn set_cursor(&mut self, cursor: Cursor) {
        self.requested_cursor = Some(cursor);
    }

    pub fn effective_scale_factor(&self) -> f64 {
        self.scale_factor * self.zoom_factor
    }

    pub(crate) fn reset(&mut self) {
        self.requested_window_width = None;
        self.requested_window_height = None;
        self.requested_mouse_position_x = None;
        self.requested_mouse_position_y = None;
        self.requested_cursor = None;
    }
}
*/
