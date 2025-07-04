use crate::components::component::ComponentId;
use crate::elements::text::TextState;
use crate::elements::text_input::TextInputState;
use crate::reactive::element_state_store::ElementStateStore;
use crate::resource_manager::resource::Resource;
use crate::resource_manager::{ResourceIdentifier, ResourceManager};
use std::sync::Arc;

use taffy::{AvailableSpace, Size};

use crate::style::Style;
use crate::text::text_context::TextContext;

pub struct TaffyTextContext {
    pub id: ComponentId,
}

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct TextHashKey {
    pub width_constraint: Option<u32>,
    pub height_constraint: Option<u32>,
    pub available_space_width: AvailableSpaceKey,
    pub available_space_height: AvailableSpaceKey,
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum AvailableSpaceKey {
    /// The amount of space available is the specified number of pixels
    Definite(u32),
    /// The amount of space available is indefinite and the node should be laid out under a min-content constraint
    MinContent,
    /// The amount of space available is indefinite and the node should be laid out under a max-content constraint
    MaxContent,
}

impl TaffyTextContext {
    pub fn new(id: ComponentId) -> Self {
        Self { id }
    }
}

pub struct ImageContext {
    pub(crate) resource_identifier: ResourceIdentifier,
}

impl ImageContext {
    pub fn measure(
        &mut self,
        known_dimensions: Size<Option<f32>>,
        _available_space: Size<taffy::AvailableSpace>,
        resource_manager: Arc<ResourceManager>,
        _style: &taffy::Style,
    ) -> Size<f32> {
        let mut original_image_width: f32 = 0.0;
        let mut original_image_height: f32 = 0.0;
        if let Some(resource) = resource_manager.resources.get(&self.resource_identifier) && let Resource::Image(image_data) = resource.as_ref() {
            original_image_width = image_data.width as f32;
            original_image_height = image_data.height as f32;
        }

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

pub type LayoutFn = fn(
    component_id: ComponentId,
    element_state: &mut ElementStateStore,
    known_dimensions: Size<Option<f32>>,
    available_space: Size<AvailableSpace>,
    text_context: &mut TextContext,
) -> Size<f32>;

pub enum LayoutContext {
    Text(TaffyTextContext),
    TextInput(TaffyTextInputContext),
    Image(ImageContext),
    TinyVg(TinyVgContext),
    Other(ComponentId, LayoutFn),
}

pub fn measure_content(
    element_state: &mut ElementStateStore,
    known_dimensions: Size<Option<f32>>,
    available_space: Size<taffy::AvailableSpace>,
    node_context: Option<&mut LayoutContext>,
    text_context: &mut TextContext,
    resource_manager: Arc<ResourceManager>,
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

            text_state.measure(known_dimensions, available_space, text_context)
        }
        Some(LayoutContext::Image(image_context)) => {
            image_context.measure(known_dimensions, available_space, resource_manager, style)
        }
        Some(LayoutContext::TextInput(taffy_text_input_context)) => {
            let text_input_state: &mut TextInputState =
                element_state.storage.get_mut(&taffy_text_input_context.id).unwrap().data.downcast_mut().unwrap();

            text_input_state.measure(known_dimensions, available_space, text_context)
        }
        Some(LayoutContext::TinyVg(tinyvg_context)) => {
            tinyvg_context.measure(known_dimensions, available_space, resource_manager, style)
        }
        Some(LayoutContext::Other(component_id, measure_fn)) => {
            measure_fn(*component_id, element_state, known_dimensions, available_space, text_context)
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

pub struct TaffyTextInputContext {
    pub id: ComponentId,
}

impl TaffyTextInputContext {
    pub fn new(id: ComponentId) -> Self {
        Self { id }
    }
}

impl TextHashKey {
    pub fn new(known_dimensions: Size<Option<f32>>, available_space: Size<taffy::AvailableSpace>) -> Self {
        let available_space_width_u32: AvailableSpaceKey = match available_space.width {
            taffy::AvailableSpace::MinContent => AvailableSpaceKey::MinContent,
            taffy::AvailableSpace::MaxContent => AvailableSpaceKey::MaxContent,
            taffy::AvailableSpace::Definite(width) => AvailableSpaceKey::Definite(width.to_bits()),
        };
        let available_space_height_u32: AvailableSpaceKey = match available_space.height {
            taffy::AvailableSpace::MinContent => AvailableSpaceKey::MinContent,
            taffy::AvailableSpace::MaxContent => AvailableSpaceKey::MaxContent,
            taffy::AvailableSpace::Definite(height) => AvailableSpaceKey::Definite(height.to_bits()),
        };

        Self {
            width_constraint: known_dimensions.width.map(|w| w.to_bits()),
            height_constraint: known_dimensions.height.map(|h| h.to_bits()),
            available_space_width: available_space_width_u32,
            available_space_height: available_space_height_u32,
        }
    }

    pub fn available_space(&self) -> Size<taffy::AvailableSpace> {
        Size {
            width: match self.available_space_width {
                AvailableSpaceKey::Definite(width) => taffy::AvailableSpace::Definite(f32::from_bits(width)),
                AvailableSpaceKey::MinContent => taffy::AvailableSpace::MinContent,
                AvailableSpaceKey::MaxContent => taffy::AvailableSpace::MaxContent,
            },
            height: match self.available_space_height {
                AvailableSpaceKey::Definite(height) => taffy::AvailableSpace::Definite(f32::from_bits(height)),
                AvailableSpaceKey::MinContent => taffy::AvailableSpace::MinContent,
                AvailableSpaceKey::MaxContent => taffy::AvailableSpace::MaxContent,
            },
        }
    }

    pub fn known_dimensions(&self) -> Size<Option<f32>> {
        Size {
            width: self.width_constraint.map(f32::from_bits),
            height: self.height_constraint.map(f32::from_bits),
        }
    }
}

pub struct TinyVgContext {
    pub(crate) resource_identifier: ResourceIdentifier,
}

impl TinyVgContext {
    pub fn measure(
        &mut self,
        known_dimensions: Size<Option<f32>>,
        _available_space: Size<taffy::AvailableSpace>,
        resource_manager: Arc<ResourceManager>,
        _style: &taffy::Style,
    ) -> Size<f32> {
        let mut original_image_width: f32 = 0.0;
        let mut original_image_height: f32 = 0.0;

        if let Some(resource) = resource_manager.resources.get(&self.resource_identifier) && 
            let Resource::TinyVg(resource) = resource.as_ref() && 
            let Some(tinyvg) = &resource.tinyvg {
            original_image_width = tinyvg.header.width as f32;
            original_image_height = tinyvg.header.height as f32;
        }

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
