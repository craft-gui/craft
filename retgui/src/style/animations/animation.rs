use std::time::Duration;

use smallvec::SmallVec;

use crate::style::StyleVariant;
use crate::style::animations::interpolation::interpolate_style_variant;
use crate::style::animations::keyframe::KeyFrame;
use crate::style::animations::timing_function::{FixedCubicBezier, ParamCurve, TimingFunction};

/// A key-frame based Animation.
#[derive(Clone, Debug)]
pub struct Animation {
    /// The list of keyframes for the Animation. It should have at least 2 elements.
    ///
    /// It is assumed key frames are sorted.
    pub key_frames: SmallVec<[KeyFrame; 2]>,
    /// The duration within the current loop.
    pub current_duration: Duration,
    /// The duration of the animation.
    pub duration: Duration,
    /// The function used to interpolate the style values between keyframes.
    pub timing_function: TimingFunction,
    /// How many times the animation repeats.
    pub repeat: Repeat,
    /// Amount of times the animation has finished.
    completed_loops: u32,
}

/// How many times the animation repeats.
#[derive(Clone, Copy, Debug)]
pub enum Repeat {
    Forever,
    Fixed(u32),
}

impl Animation {
    /// Creates a blank animation.
    pub fn new(duration: Duration, repeat: Repeat, timing_function: TimingFunction) -> Self {
        Self {
            key_frames: SmallVec::new(),
            current_duration: Duration::ZERO,
            duration,
            timing_function,
            completed_loops: 0,
            repeat,
        }
    }

    /// Adds a keyframe to the animation.
    pub fn push(mut self, key_frame: KeyFrame) -> Self {
        self.key_frames.push(key_frame);
        self
    }

    /// Sets the repeat count of the animation.
    pub fn repeat(mut self, repeat: Repeat) -> Self {
        self.repeat = repeat;
        self
    }

    /// Sets the duration of the animation.
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Advances an active animation.
    pub(crate) fn tick(&mut self, delta: Duration) {
        if self.is_finished() {
            return;
        }

        self.current_duration += delta;
        if self.current_duration < self.duration {
            return;
        }

        let elapsed_loops = (self.current_duration.as_nanos() / self.duration.as_nanos()).min(u32::MAX as u128) as u32;
        if let Repeat::Fixed(amount) = self.repeat
            && elapsed_loops >= amount.saturating_sub(self.completed_loops)
        {
            self.completed_loops = amount;
            self.current_duration = self.duration;
            return;
        }

        self.completed_loops = self.completed_loops.saturating_add(elapsed_loops);
        self.current_duration -= self.duration * elapsed_loops;
    }

    /// Returns if the animation is finished.
    pub fn is_finished(&self) -> bool {
        self.duration.is_zero() || matches!(self.repeat, Repeat::Fixed(amount) if self.completed_loops >= amount)
    }

    /// Returns whether this animation changes the font size.
    pub(crate) fn animates_font_size(&self) -> bool {
        self.key_frames
            .iter()
            .flat_map(KeyFrame::styles)
            .any(|style| matches!(style, StyleVariant::FontSize(_)))
    }

    /// Called after `tick`, and is responsible for using the current animation time and
    /// computing an interpolated style from a provided `Animation`.
    pub fn apply_styles(&self, set_style_variant: &mut dyn FnMut(StyleVariant)) {
        let pos = if self.duration.is_zero() {
            1.0
        } else {
            Duration::div_duration_f32(self.current_duration, self.duration)
        };
        let (keyframe_start, keyframe_end) = find_keyframe_pair(pos, &self.key_frames);
        let start_percentage = keyframe_start.percentage();
        let end_percentage = keyframe_end.percentage();
        let local_t = (pos - start_percentage) / (end_percentage - start_percentage);

        let t = match &self.timing_function {
            TimingFunction::Linear => {
                // https://developer.mozilla.org/en-US/docs/Web/CSS/animation-timing-function#linear
                let linear = FixedCubicBezier::new(0.0, 0.0, 1.0, 1.0);
                linear.cubic_bez.eval(local_t as f64).y
            }
            TimingFunction::Ease => {
                // https://developer.mozilla.org/en-US/docs/Web/CSS/animation-timing-function#ease
                let ease = FixedCubicBezier::new(0.25, 0.1, 0.25, 1.0);
                ease.cubic_bez.eval(local_t as f64).y
            }
            TimingFunction::EaseIn => {
                // https://developer.mozilla.org/en-US/docs/Web/CSS/animation-timing-function#ease-in
                let ease_in = FixedCubicBezier::new(0.42, 0.0, 1.0, 1.0);
                ease_in.cubic_bez.eval(local_t as f64).y
            }
            TimingFunction::EaseOut => {
                // https://developer.mozilla.org/en-US/docs/Web/CSS/animation-timing-function#ease-out
                let ease_out = FixedCubicBezier::new(0.0, 0.0, 0.58, 1.0);
                ease_out.cubic_bez.eval(local_t as f64).y
            }
            TimingFunction::EaseInOut => {
                // https://developer.mozilla.org/en-US/docs/Web/CSS/animation-timing-function#ease-in-out
                let ease_in_out = FixedCubicBezier::new(0.42, 0.0, 0.58, 1.0);
                ease_in_out.cubic_bez.eval(local_t as f64).y
            }
            // https://developer.mozilla.org/en-US/docs/Web/CSS/animation-timing-function#cubic-beziernumber_01_number_number_01_number
            TimingFunction::BezierCurve(cubic_bezier) => cubic_bezier.cubic_bez.eval(local_t as f64).y,
        };

        for start_prop in keyframe_start.styles() {
            let property = std::mem::discriminant(start_prop);
            let Some(end_prop) = keyframe_end
                .styles()
                .iter()
                .rev()
                .find(|end_prop| std::mem::discriminant(*end_prop) == property)
            else {
                continue;
            };

            set_style_variant(interpolate_style_variant(start_prop, end_prop, t));
        }
    }
}

fn find_keyframe_pair(pos: f32, keyframes: &[KeyFrame]) -> (&KeyFrame, &KeyFrame) {
    let mut keyframes = keyframes.iter();
    let Some(mut start) = keyframes.next() else {
        panic!("No keyframes available for the current animation position.");
    };
    if pos < start.percentage() {
        panic!("No keyframes available for the current animation position.");
    }

    for end in keyframes {
        if pos <= end.percentage() {
            return (start, end);
        }
        start = end;
    }

    panic!("No keyframes available for the current animation position.");
}
