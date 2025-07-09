use crate::elements::ElementState;
use crate::style::{Style, StyleProperty, Unit};
use craft_primitives::geometry::TrblRectangle;
use kurbo::{CubicBez, ParamCurve, Point};
use smallvec::SmallVec;
use std::collections::HashMap;
use std::iter::zip;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct KeyFrame {
    /// The action / styles interpolated at `offset_percentage`.
    /// Range [0.0, 100.0]
    offset_percentage: f32,
    
    /// The list of styles interpolated to an element at this keyframe.
    properties: SmallVec<[StyleProperty; 3]>,
}

impl KeyFrame {
    pub fn new(offset_percentage: f32) -> Self {
        KeyFrame {
            offset_percentage,
            properties: SmallVec::new(),
        }
    }
    
    pub fn push(mut self, property: StyleProperty) -> Self {
        self.properties.push(property);
        self
    }
}

#[derive(Clone, Debug)]
#[derive(PartialEq)]
pub enum AnimationStatus {
    Paused,
    Playing,
    Scheduled,
}

/// A cubic bézier curve where P0 and P3 are stuck at (0,0) and (1,1).
#[derive(Clone, Debug)]
pub struct FixedCubicBezier {
    cubic_bez: CubicBez,
}

impl FixedCubicBezier {
    /// Sets P1 and P2 of a fixed cubic bézier curve.
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self {
            cubic_bez: CubicBez::new(
                Point::new(0.0, 0.0),
                Point::new(x1 as f64, y1 as f64),
                Point::new(x2 as f64, y2 as f64),
                Point::new(1.0, 1.0),
            )
        }
    }
}


/// The motion of an animation modeled with a mathematical function.
#[derive(Default, Clone, Debug)]
pub enum TimingFunction {
    /// https://developer.mozilla.org/en-US/docs/Web/CSS/animation-timing-function#linear
    #[default]
    Linear,
    /// https://developer.mozilla.org/en-US/docs/Web/CSS/animation-timing-function#ease
    Ease,
    /// https://developer.mozilla.org/en-US/docs/Web/CSS/animation-timing-function#ease-in
    EaseIn,
    /// https://developer.mozilla.org/en-US/docs/Web/CSS/animation-timing-function#ease-out
    EaseOut,
    /// https://developer.mozilla.org/en-US/docs/Web/CSS/animation-timing-function#ease-in-out
    EaseInOut,
    /// https://developer.mozilla.org/en-US/docs/Web/CSS/animation-timing-function#cubic-beziernumber_01_number_number_01_number
    BezierCurve(FixedCubicBezier),
}

#[derive(Clone, Debug)]
pub struct Animation {
    pub name: String,
    pub key_frames: SmallVec<[KeyFrame; 2]>,
    pub duration: Duration,
    pub timing_function: TimingFunction,
    pub loop_amount: LoopAmount,
}

#[derive(Clone, Debug)]
pub enum LoopAmount {
    Infinite,
    Fixed(u32)
}

impl Animation {
    pub fn new(name: String, duration: Duration, timing_function: TimingFunction) -> Self {
        Self {
            name,
            key_frames: SmallVec::new(),
            duration,
            timing_function,
            loop_amount: LoopAmount::Fixed(1),
        }
    }
    
    pub fn push(mut self, key_frame: KeyFrame) -> Self {
        self.key_frames.push(key_frame);
        self
    }
    
    pub fn loop_amount(mut self, loop_amount: LoopAmount) -> Self {
        self.loop_amount = loop_amount;
        self
    }
}

#[derive(Clone, Debug)]
pub struct ActiveAnimation {
    /// How far into an animation we are.
    pub(crate) current: Duration,
    /// Tracks the status of an animation, if it is playing, scheduled, or paused.
    pub(crate) status: AnimationStatus,
    pub(crate) loop_amount: LoopAmount,
}

/// For damage tracking across recursive calls to `on_animation_frame`.
#[derive(Clone, Debug, Default)]
pub struct AnimationFlags {
    needs_relayout: bool,
    has_active_animation: bool,
}

impl AnimationFlags {
    /// OR'd with the provided boolean and the previously stored boolean, to track if an animatable property effects layout.
    /// This is used after `on_animation_frame` to optionally recompute the layout.
    pub fn set_needs_relayout(&mut self, needs_relayout: bool) {
        self.needs_relayout = self.needs_relayout | needs_relayout;
    }
    
    /// Returns whether we need to perform a relayout or not.
    pub fn needs_relayout(&self) -> bool {
        self.needs_relayout
    }

    /// OR'd with the provided boolean and the previously stored boolean, to track if any animation is active.
    pub fn set_has_active_animation(&mut self, has_active_animation: bool) {
        self.has_active_animation = self.has_active_animation | has_active_animation;
    }

    /// Returns true if any animation is in the Playing state.
    pub fn has_active_animation(&self) -> bool {
        self.has_active_animation
    }
}

impl ActiveAnimation {
    
    /// Advances an active animation, and it is also responsible for tracking the status and element_state. 
    pub fn tick(&mut self, animation_flags: &mut AnimationFlags, animation: &Animation, state: ElementState, delta: Duration) {
        if self.status == AnimationStatus::Playing {
            self.current += delta;

            let is_completed = self.current >= animation.duration;
            
            match &mut self.loop_amount {
                LoopAmount::Infinite => {
                    if is_completed {
                        self.current = Duration::ZERO;
                    }
                }
                LoopAmount::Fixed(amount) => {
                    if is_completed {
                        *amount -= 1;

                        if *amount == 0 {
                            self.current = Duration::ZERO;
                            self.status = AnimationStatus::Paused;
                            animation_flags.set_needs_relayout(true);
                        } else {
                            self.current = Duration::ZERO;
                        }
                    }
                }
            }
            
        }
    }

    /// Called after `tick`, and is responsible for using the current animation time and
    /// computing an interpolated style from a provided `Animation`.
    pub fn compute_style(&mut self, element_style: &Style, animation: &Animation, state: ElementState, animation_flags: &mut AnimationFlags) -> Style {
        if self.status != AnimationStatus::Playing {
            return element_style.clone();
        }

        let pos = Duration::div_duration_f32(self.current, animation.duration);
        fn find_keyframe_pair(pos: f32, animation: &Animation) -> (&KeyFrame, &KeyFrame) {
            let mut sorted = animation.key_frames.iter().collect::<Vec<_>>();
            sorted.sort_by(|a, b| a.offset_percentage.total_cmp(&b.offset_percentage));
            for window in sorted.windows(2) {
                let [start, end] = window else { continue };
                if pos >= (start.offset_percentage / 100.0) && pos <= (end.offset_percentage / 100.0) {
                    return (start, end);
                }
            }

            panic!("No keyframes available.");
        }

        let (keyframe_start, keyframe_end) = find_keyframe_pair(pos, animation);
        
        let mut style = Style::default();
        let mut start_map = HashMap::new();
        let mut end_map = HashMap::new();

        for prop in &keyframe_start.properties {
            start_map.insert(std::mem::discriminant(prop), prop);
        }
        
        for prop in &keyframe_end.properties {
            end_map.insert(std::mem::discriminant(prop), prop);
        }

        for key in start_map.keys().chain(end_map.keys()).collect::<std::collections::HashSet<_>>() {
            let start_prop = start_map.get(key);
            let end_prop = end_map.get(key);

            let start_percentage = keyframe_start.offset_percentage / 100.0;
            let end_percentage = keyframe_end.offset_percentage / 100.0;
            let local_t = (pos - start_percentage) / (end_percentage - start_percentage);

            let t = match &animation.timing_function {
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
                TimingFunction::BezierCurve(cubic_bezier) => {
                    cubic_bezier.cubic_bez.eval(local_t as f64).y
                }
            };

            fn lerp(a: f32, b: f32, t: f32) -> f32 {
                a + (b - a) * t
            }

            #[inline(always)]
            fn resolve_unit(start: &Unit, end: &Unit, t: f64, set_prop: &mut dyn FnMut(Unit)) {
                let resolved_start = match start {
                    Unit::Px(px) => *px,
                    Unit::Percentage(percent) => *percent,
                    Unit::Auto => panic!("Unit must not be auto.")
                };
                
                let resolved_end = match end {
                    Unit::Px(px) => *px,
                    Unit::Percentage(percent) => *percent,
                    Unit::Auto => panic!("Unit must not be auto.")
                };
                let new = lerp(resolved_start, resolved_end, t as f32);
                
                // Naively asserts that start and end must be the same Unit type.
                let new = match start {
                    Unit::Px(_) => Unit::Px(new),
                    Unit::Percentage(_) => Unit::Percentage(new),
                    _ => unreachable!()
                };
                
                set_prop(new);
            }
            
            match (start_prop, end_prop) {
                (Some(StyleProperty::Background(start)), Some(StyleProperty::Background(end))) => {
                    let new_color = start.lerp_rect(*end, t as f32);
                    style.set_background(new_color);
                }
                (Some(StyleProperty::Color(start)), Some(StyleProperty::Color(end))) => {
                    let new_color = start.lerp_rect(*end, t as f32);
                    style.set_color(new_color);
                    animation_flags.set_needs_relayout(true);
                }
                (Some(StyleProperty::FontSize(start)), Some(StyleProperty::FontSize(end))) => {
                    let new = lerp(*start, *end, t as f32);
                    style.set_font_size(new);
                    animation_flags.set_needs_relayout(true);
                }
                (Some(StyleProperty::Width(start)), Some(StyleProperty::Width(end))) 
                => {
                    resolve_unit(start, end, t, &mut |new| {
                        style.set_width(new);
                    });
                    animation_flags.set_needs_relayout(true);
                }
                (Some(StyleProperty::Height(start)), Some(StyleProperty::Height(end))) => {
                    resolve_unit(start, end, t, &mut |new| {
                        style.set_height(new);
                    });
                    animation_flags.set_needs_relayout(true);
                }

                (Some(StyleProperty::Inset(start)), Some(StyleProperty::Inset(end))) => {
                    let trlb = zip(start.to_array(), end.to_array()).map(|(start, end)| {
                        let mut inset_unit = Unit::Auto;
                        resolve_unit(&start, &end, t, &mut |new| {
                            inset_unit = new;
                        });
                        
                        inset_unit
                    }).collect::<Vec<Unit>>();

                    let inset = TrblRectangle::new(trlb[0], trlb[1], trlb[2], trlb[3]);
                    
                    style.set_inset(inset);
                    animation_flags.set_needs_relayout(true);
                }
                
                _ => {}
            }

        }



        style
    }
}