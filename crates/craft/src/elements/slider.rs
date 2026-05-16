use std::cell::RefCell;
use std::rc::Rc;

use craft_retained::Color;
use craft_retained::elements::{AsElement, Element, ElementInternals, SliderDirection};

use crate::signals::Bindable;

#[derive(Clone)]
pub struct Slider {
    pub inner: craft_retained::elements::Slider,
}

impl AsElement for Slider {
    fn as_element_rc(&self) -> Rc<RefCell<dyn ElementInternals>> {
        self.inner.inner.clone()
    }
}

impl Element for Slider {}

impl Slider {
    pub fn new(thumb_size: impl Bindable<f64>) -> Self {
        let inner = craft_retained::elements::Slider::new(1.0);
        let inner_clone = inner.clone();
        thumb_size.bind(move |value| {
            inner_clone.clone().thumb_size(value);
        });
        Self { inner }
    }

    pub fn value(self, value: impl Bindable<f64>) -> Self {
        let element = self.clone();
        value.bind(move |value| {
            element.clone().inner.value(value);
        });
        self
    }

    pub fn get_value(&self) -> f64 {
        self.inner.get_value()
    }

    pub fn step(self, value: impl Bindable<f64>) -> Self {
        let element = self.clone();
        value.bind(move |value| {
            element.clone().inner.step(value);
        });
        self
    }

    pub fn get_step(&self) -> f64 {
        self.inner.get_step()
    }

    pub fn min(self, min: impl Bindable<f64>) -> Self {
        let element = self.clone();
        min.bind(move |value| {
            element.clone().inner.min(value);
        });
        self
    }

    pub fn get_min(&self) -> f64 {
        self.inner.get_min()
    }

    pub fn max(self, max: impl Bindable<f64>) -> Self {
        let element = self.clone();
        max.bind(move |value| {
            element.clone().inner.max(value);
        });
        self
    }

    pub fn get_max(&self) -> f64 {
        self.inner.get_max()
    }

    pub fn direction(self, direction: impl Bindable<SliderDirection>) -> Self {
        let element = self.clone();
        direction.bind(move |direction| {
            element.clone().inner.direction(direction);
        });
        self
    }

    pub fn get_direction(&self) -> SliderDirection {
        self.inner.get_direction()
    }

    pub fn thumb_size(self, thumb_size: impl Bindable<f64>) -> Self {
        let element = self.clone();
        thumb_size.bind(move |value| {
            element.clone().inner.thumb_size(value);
        });
        self
    }

    pub fn get_thumb_size(&self) -> f64 {
        self.inner.get_thumb_size()
    }

    pub fn thumb_color(self, thumb_background_color: impl Bindable<Color>) -> Self {
        let element = self.clone();
        thumb_background_color.bind(move |value| {
            element.clone().inner.thumb_color(value);
        });
        self
    }

    pub fn get_thumb_color(&self) -> Color {
        self.inner.get_thumb_color()
    }

    pub fn thumb_border_radius(self, thumb_border_radius: impl Bindable<[(f32, f32); 4]>) -> Self {
        let element = self.clone();
        thumb_border_radius.bind(move |value| {
            element
                .clone()
                .inner
                .thumb_border_radius(value[0], value[1], value[2], value[3]);
        });
        self
    }

    pub fn get_thumb_border_radius(&self) -> Option<[(f32, f32); 4]> {
        self.inner.get_thumb_border_radius()
    }

    pub fn track_color(self, track_background_color: impl Bindable<Color>) -> Self {
        let element = self.clone();
        track_background_color.bind(move |value| {
            element.clone().track_color(value);
        });
        self
    }

    pub fn get_track_color(&self) -> Option<Color> {
        self.inner.get_track_color()
    }

    pub fn track_border_radius(self, track_border_radius: impl Bindable<[(f32, f32); 4]>) -> Self {
        let element = self.clone();
        track_border_radius.bind(move |value| {
            element
                .clone()
                .inner
                .track_border_radius(value[0], value[1], value[2], value[3]);
        });
        self
    }

    pub fn get_track_border_radius(&self) -> Option<[(f32, f32); 4]> {
        self.inner.get_track_border_radius()
    }
}
