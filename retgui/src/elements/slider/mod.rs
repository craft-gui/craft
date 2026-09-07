#[cfg(feature = "audio")]
pub(crate) use slider_element::SliderElement;
pub use slider_element::{Slider, SliderDirection};

mod slider_element;
