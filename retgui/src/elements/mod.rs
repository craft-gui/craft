use std::any::Any;

use rustc_hash::FxHashMap;

use slotmap::{DefaultKey, SlotMap};

pub use crate::accessibility::RetGuiAccessTree;
#[cfg(feature = "audio")]
pub use crate::elements::audio::Audio;
pub use crate::elements::button::Button;
#[cfg(feature = "audio")]
pub(crate) use crate::elements::button::ButtonElement;
pub use crate::elements::calendar::Calendar;
pub use crate::elements::checkbox::Checkbox;
pub use crate::elements::checkboxgroup::CheckboxGroup;
#[cfg(feature = "code_highlighting")]
pub use crate::elements::codeeditor::CodeEditor;
pub use crate::elements::container::Container;
pub(crate) use crate::elements::container::ContainerElement;
pub use crate::elements::dropdown::Dropdown;
pub(crate) use crate::elements::dropdown::DropdownElement;
pub use crate::elements::dyn_element::DynElement;
pub use crate::elements::editor::ElementEditor;
pub use crate::elements::element_data::ElementData;
pub use crate::elements::image::Image;
pub(crate) use crate::elements::image::ImageElement;
#[cfg(feature = "markdown")]
pub use crate::elements::markdown::render_markdown;
pub use crate::elements::radio::Radio;
pub(crate) use crate::elements::radio::RadioElement;
pub use crate::elements::radiogroup::RadioGroup;
pub use crate::elements::scrollable::{ScrollOptions, ScrollState, ScrollToBox};
#[cfg(feature = "audio")]
pub(crate) use crate::elements::slider::SliderElement;
pub use crate::elements::slider::{Slider, SliderDirection};
pub use crate::elements::store::{RetainedElements, State};
pub use crate::elements::text::Text;
pub(crate) use crate::elements::text::TextElement;
pub use crate::elements::text_input::TextInput;
pub(crate) use crate::elements::text_input::TextInputElement;
pub use crate::elements::tinyvg::TinyVg;
pub(crate) use crate::elements::tinyvg::TinyVgElement;
pub(crate) use crate::elements::traits::set_focus_outline_visible;
pub use crate::elements::traits::{AnimationInstant, AnimationSchedule, Element, ElementInternals, HasElementData, clone_element};
pub use crate::elements::window::Window;
pub(crate) use crate::elements::window::WindowElement;

pub(crate) mod internal_helpers;
pub(crate) mod scrollable;

#[cfg(feature = "audio")]
pub(crate) mod audio;
pub mod button;
mod calendar;
mod checkbox;
mod checkboxgroup;
#[cfg(feature = "code_highlighting")]
mod codeeditor;
mod container;
mod dropdown;
mod dyn_element;
mod editor;
mod element_data;
mod element_id;
pub(crate) mod gui_actions;
mod image;
#[cfg(feature = "markdown")]
mod markdown;
mod radio;
mod radiogroup;
mod slider;
mod store;
mod text;
mod text_input;
mod tinyvg;
mod traits;
mod window;

pub type ElementStates = SlotMap<DefaultKey, Box<dyn Any>>;
pub type ElementIds = FxHashMap<u64, DynElement>;
