use std::cmp::Ordering;
use std::collections::HashMap;
use std::time::Duration;
use kurbo::{CubicBez, ParamCurve, ParamCurveCurvature, Point};
use kurbo::offset::CubicOffset;
use peniko::color::HueDirection;
use rustc_hash::FxHashMap;
use smallvec::SmallVec;
use crate::components::ComponentId;
use crate::elements::ElementState;
use crate::style::{Style, StyleProperty};

#[derive(Clone, Debug)]
pub struct KeyFrame {
    pub offset_percentage: f32,
    //pub properties: SmallVec<[StyleProperty; 5]>,
    pub properties: Vec<StyleProperty>,
}

impl KeyFrame {
    
}

#[derive(Clone, Debug)]
#[derive(PartialEq)]
pub enum AnimationStatus {
    Paused,
    Playing,
    Scheduled,
}

#[derive(Clone, Debug)]
pub struct CubicBezier {
    cubic_bez: CubicBez,
}

impl CubicBezier {
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


#[derive(Default, Clone, Debug)]
pub enum TimingFunction {
    #[default]
    Linear,
    EaseIn,
    EaseOut,
    BezierCurve(CubicBezier),
    EaseInOut,
    Ease,
}

#[derive(Clone, Debug)]
pub struct Animation {
    pub key_frames: Vec<KeyFrame>,
    pub duration: Duration,
    pub timing_function: TimingFunction,
}

pub struct ActiveAnimation {
    current: Duration,
    status: AnimationStatus,
    element_state: ElementState
}

pub struct AnimationController {
    pub(crate) animations: FxHashMap<ComponentId, ActiveAnimation>,
}

impl AnimationController {
    pub fn remove(&mut self, component: ComponentId) {
        self.animations.remove(&component);
    }

    pub fn tick(&mut self, animation: &Animation, state: ElementState, component: ComponentId, delta: Duration) {
        let active_animation = if let Some(active_animation) = self.animations.get_mut(&component) {
            active_animation
        } else {
            self.animations.insert(component, ActiveAnimation {
                current: Duration::ZERO,
                status: AnimationStatus::Playing,
                element_state: state,
            });
            self.animations.get_mut(&component).unwrap()
        };

        if active_animation.element_state != state {
            active_animation.current = Duration::ZERO;
            active_animation.status = AnimationStatus::Playing;
            active_animation.element_state = state;
        }

        if active_animation.status == AnimationStatus::Playing && active_animation.element_state == state {
            active_animation.current += delta;

            if active_animation.current >= animation.duration {
                active_animation.current = Duration::ZERO;
                active_animation.status = AnimationStatus::Paused;
            }
        }
    }

    pub fn compute_style(&mut self, element_style: &Style, animation: &Animation, state: ElementState, component: ComponentId) -> Style {
        let active_animation = if let Some(active_animation) = self.animations.get_mut(&component) {
            active_animation
        } else {
            return element_style.clone();
        };

        if active_animation.status != AnimationStatus::Playing || active_animation.element_state != state {
            return element_style.clone();
        }

        let pos = Duration::div_duration_f32(active_animation.current, animation.duration);
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
        println!("{:?}", (keyframe_start, keyframe_end));
        
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
                    let linear = CubicBezier::new(0.0, 0.0, 1.0, 1.0);
                    linear.cubic_bez.eval(local_t as f64).y
                }
                TimingFunction::Ease => {
                    let ease = CubicBezier::new(0.25, 0.1, 0.25, 1.0);
                    ease.cubic_bez.eval(local_t as f64).y
                }
                TimingFunction::EaseIn => {
                    let ease_in = CubicBezier::new(0.42, 0.0, 1.0, 1.0);
                    ease_in.cubic_bez.eval(local_t as f64).y
                }
                TimingFunction::EaseOut => {
                    let ease_out = CubicBezier::new(0.0, 0.0, 0.58, 1.0);
                    ease_out.cubic_bez.eval(local_t as f64).y
                }
                TimingFunction::EaseInOut => {
                    let ease_in_out = CubicBezier::new(0.42, 0.0, 0.58, 1.0);
                    ease_in_out.cubic_bez.eval(local_t as f64).y
                }
                TimingFunction::BezierCurve(cubic_bezier) => {
                    cubic_bezier.cubic_bez.eval(local_t as f64).y
                }
            };

            match (start_prop, end_prop) {
                (Some(StyleProperty::Background(start)), Some(StyleProperty::Background(end))) => {
                    let new_color = start.lerp_rect(*end, t as f32);
                    style.set_background(new_color);
                }
                _ => {}
            }
            
        }



        style
    }
}