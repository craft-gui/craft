use crate::geometry::Rectangle;

pub(crate) type ClipRectangle = Rectangle;


pub(crate) struct RenderGroup {
    pub(crate) clip_rectangle: ClipRectangle,
}

impl ClipRectangle {
    pub(crate) fn constrain_to_clip_rectangle(&self, parent_clip_rectangle: &ClipRectangle) -> Rectangle {
        // Compute the constrained x and y first.
        let constrained_x = self.x.clamp(parent_clip_rectangle.x, parent_clip_rectangle.x + parent_clip_rectangle.width);
        let constrained_y = self.y.clamp(parent_clip_rectangle.y, parent_clip_rectangle.y + parent_clip_rectangle.height);

        // Constrain the width and height to fit within the parent's clip rectangle.
        let constrained_width = (self.width + self.x).clamp(parent_clip_rectangle.x, parent_clip_rectangle.x + parent_clip_rectangle.width) - constrained_x;
        let constrained_height = (self.height + self.y).clamp(parent_clip_rectangle.y, parent_clip_rectangle.y + parent_clip_rectangle.height) - constrained_y;

        Rectangle {
            x: constrained_x,
            y: constrained_y,
            width: constrained_width,
            height: constrained_height,
        }
    }

    
}