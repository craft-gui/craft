use crate::components::component::ComponentId;
use crate::elements::text::TextState;
use crate::reactive::element_state_store::ElementStateStore;
use crate::resource_manager::resource::Resource;
use crate::resource_manager::{ResourceIdentifier, ResourceManager};
use parley::FontContext;
use peniko::Brush;

use taffy::Size;

use crate::elements::text_input::text_input::TextInputState;
use tokio::sync::RwLockReadGuard;

pub struct TaffyTextContext {
    pub id: ComponentId,
    text_hash: u64,
    font_settings_hash: u64
}
pub struct TaffyTextInputContext {
    pub id: ComponentId,
    text_hash: u64,
    font_settings_hash: u64
}

impl TaffyTextContext {
    pub fn new(id: ComponentId, text_hash: u64, font_settings_hash: u64) -> Self {
        Self { id, text_hash, font_settings_hash }
    }
}

impl TaffyTextInputContext {
    pub fn new(id: ComponentId, text_hash: u64, font_settings_hash: u64) -> Self {
        Self { id, text_hash, font_settings_hash }
    }
}

/*#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct MetricsDummy {
    /// Font size in pixels
    pub font_size: u32,
    /// Line height in pixels
    pub line_height: u32,
}*/

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

#[allow(clippy::too_many_arguments)]
pub fn measure_content(
    element_state: &mut ElementStateStore,
    known_dimensions: Size<Option<f32>>,
    available_space: Size<taffy::AvailableSpace>,
    node_context: Option<&mut LayoutContext>,
    font_context: &mut FontContext,
    font_layout_context: &mut parley::LayoutContext<Brush>,
    resource_manager: &RwLockReadGuard<ResourceManager>,
    style: &taffy::Style,
) -> Size<f32> {
    if let Size {
        width: Some(width),
        height: Some(height),
    } = known_dimensions
    {
        return Size { width, height };
    }

    match node_context {
        None => Size::ZERO,
        Some(LayoutContext::Text(taffy_text_context)) => {
            let text_state: &mut TextState =
                element_state.storage.get_mut(&taffy_text_context.id).unwrap().data.downcast_mut().unwrap();

            text_state.measure(known_dimensions, available_space, font_context, font_layout_context, taffy_text_context.text_hash, taffy_text_context.font_settings_hash)
        }
        Some(LayoutContext::TextInput(taffy_text_input_context)) => {
            let text_input_state: &mut TextInputState =
                element_state.storage.get_mut(&taffy_text_input_context.id).unwrap().data.downcast_mut().unwrap();

            text_input_state.measure(known_dimensions, available_space, font_context, font_layout_context, taffy_text_input_context.text_hash, taffy_text_input_context.font_settings_hash)
        }
        Some(LayoutContext::Image(image_context)) => {
            image_context.measure(known_dimensions, available_space, resource_manager, style)
        }
    }
}
