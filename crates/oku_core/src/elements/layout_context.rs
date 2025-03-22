use crate::components::component::ComponentId;
use crate::elements::text::TextState;
use crate::elements::text_input::TextInputState;
use crate::reactive::element_state_store::ElementStateStore;
use crate::resource_manager::resource::Resource;
use crate::resource_manager::{ResourceIdentifier, ResourceManager};

use cosmic_text::{FontSystem, Metrics};

use taffy::Size;

use crate::style::Style;
use tokio::sync::RwLockReadGuard;

pub struct TaffyTextContext {
    pub id: ComponentId
}

impl TaffyTextContext {
    pub fn new(id: ComponentId) -> Self {
        Self {
            id
        }
    }
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct MetricsRaw {
    /// Font size in pixels
    pub font_size: u32,
    /// Line height in pixels
    pub line_height: u32,
    pub scaling_factor: u64,
}

impl MetricsRaw {
    pub(crate) fn from(style: &Style, scaling_factor: f64) -> Self {
        Self {
            font_size: (style.font_size() * scaling_factor as f32).to_bits(),
            line_height: (style.font_size() * scaling_factor as f32).to_bits(),
            scaling_factor: scaling_factor.to_bits(),
        }
    }

    pub(crate) fn to_metrics(self) -> Metrics {
        Metrics {
            font_size: f32::from_bits(self.font_size),
            line_height: f32::from_bits(self.line_height),
        }
    }
}

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct TextHashKey {
    pub width_constraint: Option<u32>,
    pub height_constraint: Option<u32>,
    pub available_space_width: AvailableSpace,
    pub available_space_height: AvailableSpace,
}

impl TextHashKey {
    pub(crate) fn new(known_dimensions: Size<Option<f32>>, available_space:  Size<taffy::AvailableSpace>) -> Self {
        // Set width constraint
        let width_constraint = known_dimensions.width.or(match available_space.width {
            taffy::AvailableSpace::MinContent => Some(0.0),
            taffy::AvailableSpace::MaxContent => None,
            taffy::AvailableSpace::Definite(width) => Some(width),
        });

        let height_constraint = known_dimensions.height;

        let available_space_width_u32: AvailableSpace = match available_space.width {
            taffy::AvailableSpace::MinContent => AvailableSpace::MinContent,
            taffy::AvailableSpace::MaxContent => AvailableSpace::MaxContent,
            taffy::AvailableSpace::Definite(width) => AvailableSpace::Definite(width.to_bits()),
        };
        let available_space_height_u32: AvailableSpace = match available_space.height {
            taffy::AvailableSpace::MinContent => AvailableSpace::MinContent,
            taffy::AvailableSpace::MaxContent => AvailableSpace::MaxContent,
            taffy::AvailableSpace::Definite(height) => AvailableSpace::Definite(height.to_bits()),
        };

        Self {
            width_constraint: width_constraint.map(|w| w.to_bits()),
            height_constraint: height_constraint.map(|h| h.to_bits()),
            available_space_width: available_space_width_u32,
            available_space_height: available_space_height_u32,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum AvailableSpace {
    /// The amount of space available is the specified number of pixels
    Definite(u32),
    /// The amount of space available is indefinite and the node should be laid out under a min-content constraint
    MinContent,
    /// The amount of space available is indefinite and the node should be laid out under a max-content constraint
    MaxContent,
}

pub(crate) struct ImageContext {
    pub(crate) resource_identifier: ResourceIdentifier,
}

impl ImageContext {
    pub fn measure(
        &mut self,
        known_dimensions: Size<Option<f32>>,
        _available_space: Size<taffy::AvailableSpace>,
        resource_manager: &RwLockReadGuard<ResourceManager>,
        _style: &taffy::Style,
    ) -> Size<f32> {
        let mut original_image_width: f32 = 0.0;
        let mut original_image_height: f32 = 0.0;
        if let Some(Resource::Image(image_data)) = resource_manager.resources.get(&self.resource_identifier) {
            original_image_width = image_data.width as f32;
            original_image_height = image_data.height as f32;
        }
        // println!("image size: {} {}", original_image_width, original_image_height);
        // println!("known dims: {:?}", known_dimensions);
        match (known_dimensions.width, known_dimensions.height) {
            (Some(width), Some(height)) => Size { width, height },
            (Some(width), None) => Size {
                width,
                height: (width / original_image_width) * original_image_height,
            },
            (None, Some(height)) => Size {
                width: (height / original_image_height) * original_image_width,
                height,
            },
            (None, None) => Size {
                width: original_image_width,
                height: original_image_height,
            },
        }
    }
}

pub(crate) enum LayoutContext {
    Text(TaffyTextContext),
    TextInput(TaffyTextInputContext),
    Image(ImageContext),
}

pub fn measure_content(
    element_state: &mut ElementStateStore,
    known_dimensions: Size<Option<f32>>,
    available_space: Size<taffy::AvailableSpace>,
    node_context: Option<&mut LayoutContext>,
    font_system: &mut FontSystem,
    resource_manager: &RwLockReadGuard<ResourceManager>,
    style: &taffy::Style,
) -> Size<f32> {
    if let Size { width: Some(width), height: Some(height) } = known_dimensions {
        return Size { width, height };
    }

    match node_context {
        None => Size::ZERO,
        Some(LayoutContext::Text(taffy_text_context)) => {
            let text_state: &mut TextState = element_state.storage.get_mut(&taffy_text_context.id).unwrap().data.downcast_mut().unwrap();

            text_state.cached_editor.measure(
                known_dimensions,
                available_space,
                font_system,
            )
        }
        Some(LayoutContext::Image(image_context)) => {
            image_context.measure(known_dimensions, available_space, resource_manager, style)
        }
        Some(LayoutContext::TextInput(taffy_text_input_context)) => {
            let text_input_state: &mut TextInputState = element_state.storage.get_mut(&taffy_text_input_context.id).unwrap().data.downcast_mut().unwrap();
            
            text_input_state.cached_editor.measure(
                known_dimensions,
                available_space,
                font_system,
            )
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

pub struct TaffyTextInputContext {
    pub id: ComponentId,
}

impl TaffyTextInputContext {
    pub fn new(id: ComponentId) -> Self {
        Self {
            id,
        }
    }
}
