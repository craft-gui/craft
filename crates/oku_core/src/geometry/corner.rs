use crate::geometry::cornerside::CornerSide;
use crate::geometry::side::Side;
use std::f64::consts::{PI, TAU};

#[repr(usize)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Corner {
    TopLeft = 0,
    TopRight = 1,
    BottomRight = 2,
    BottomLeft = 3,
}

impl Corner {
    pub(crate) const fn get_primary_side(self) -> Side {
        // 000 & 010 = 000
        // 001 & 010 = 000
        // 011 & 010 = 010
        // 010 & 010 = 010
        unsafe { std::mem::transmute(self as usize & 2) }
    }

    pub(crate) const fn get_secondary_side(self) -> Side {
        // 000 -> 011
        // 001 -> 001
        // 010 -> 001
        // 011 -> 011

        let c = self as usize;
        let bit = (c ^ (self as usize >> 1)) & 1;
        let side_val = 3 - (bit << 1);

        unsafe { std::mem::transmute(side_val) }
    }

    pub(crate) fn get_inner_start_side(self) -> CornerSide {
        // 000 -> 010
        // 001 -> 000
        // 010 -> 000
        // 011 -> 010

        match self {
            Corner::TopLeft => CornerSide::Bottom,
            Corner::TopRight => CornerSide::Top,
            Corner::BottomRight => CornerSide::Top,
            Corner::BottomLeft => CornerSide::Bottom,
        }
    }

    pub(crate) fn get_outer_start_side(self) -> CornerSide {
        match self {
            Corner::TopLeft => CornerSide::Top,
            Corner::TopRight => CornerSide::Bottom,
            Corner::BottomRight => CornerSide::Bottom,
            Corner::BottomLeft => CornerSide::Top,
        }
    }

    pub(crate) fn get_relative_angle(self, angle: f64) -> f64 {
        match self {
            Corner::TopRight => angle,
            Corner::TopLeft => PI - angle,
            Corner::BottomLeft => PI + angle,
            Corner::BottomRight => TAU - angle,
        }
    }
}
