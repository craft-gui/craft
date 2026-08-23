use crate::style::StyleVariant;
use smallvec::SmallVec;

/// A fixed point in an Animation that will be used to interpolate all other values.
#[derive(Clone, Debug)]
pub struct KeyFrame {
    /// Percent through the animation. [0, 1].
    percentage: f32,

    /// Style values at the KeyFrame.
    ///
    /// Not all styles can be interpolated e.g. the font face.
    styles: SmallVec<[StyleVariant; 3]>,
}

impl KeyFrame {
    /// Create a blank keyframe with a percentage value from 0.0 to 100.0.
    pub fn new(percentage: f32) -> Self {
        if percentage < 0.0 || percentage > 100.0 {
            panic!("percentage must be between 0 and 100");
        }
        KeyFrame {
            percentage: percentage / 100.0,
            styles: SmallVec::new(),
        }
    }

    pub fn push(mut self, property: StyleVariant) -> Self {
        self.styles.push(property);
        self
    }

    /// Sets the percent at which the keyframe will be applied. This must be 0.0 to 100.0.
    pub fn set_percentage(mut self, percentage: f32) {
        if percentage < 0.0 || percentage > 100.0 {
            panic!("percentage must be between 0 and 100");
        }
        self.percentage = percentage;
    }

    /// Returns the percent at which the keyframe will be applied.
    pub fn percentage(&self) -> f32 {
        self.percentage
    }

    pub(crate) fn styles(&self) -> &[StyleVariant] {
        &self.styles
    }
}
