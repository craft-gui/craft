use craft_primitives::geometry::Rectangle;

#[derive(Debug)]
pub struct TargetItem {
    pub custom_id: u64,
    pub rectangle: Rectangle,
    pub overlay_depth: u64,
}

impl TargetItem {
    pub fn new(custom_id: u64, rectangle: Rectangle, overlay_depth: u64) -> Self {
        Self {
            custom_id,
            rectangle,
            overlay_depth,
        }
    }

    // Sorts the items by the overlay depth and in ascending order.
    pub fn sort_items_by_overlay_depth(targets: &mut [TargetItem]) {
        targets.sort_by_key(|t1| t1.overlay_depth);
    }
}
