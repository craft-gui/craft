use crate::geometry::Point;

pub trait HitTestable {
    fn hit_test(&self, point: &Point) -> bool;

    fn has_event_handlers(&self) -> bool;

    fn id(&self) -> usize;
}
