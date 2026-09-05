use std::sync::Arc;

use gummy::{AvailableSpace, Size};

use retgui_resource_manager::image::ImageResource;
use retgui_resource_manager::{ResourceId, ResourceManager};

use tinyvg_rs::TinyVg;

use crate::elements::{DynElement, RetainedElements, TextElement, TextInputElement};
use crate::text::text_context::TextContext;

#[derive(Clone)]
pub(crate) struct GummyTextContext {
    pub(crate) element: DynElement,
}

#[derive(Clone)]
pub(crate) struct GummyTextInputContext {
    pub(crate) element: DynElement,
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
    /// The amount of space available is indefinite and the element should be laid out under a min-content constraint
    MinContent,
    /// The amount of space available is indefinite and the element should be laid out under a max-content constraint
    MaxContent,
}

impl GummyTextContext {}

#[derive(Clone)]
pub(crate) struct ImageContext {
    pub(crate) resource_id: ResourceId,
}

impl ImageContext {
    pub(crate) fn new(resource_id: ResourceId) -> Self {
        Self { resource_id }
    }

    pub(crate) fn measure(
        &mut self,
        known_dimensions: Size<Option<f32>>,
        _available_space: Size<AvailableSpace>,
        resource_manager: Arc<ResourceManager>,
        _style: &gummy::Style,
    ) -> Size<f32> {
        let mut original_image_width: f32 = 0.0;
        let mut original_image_height: f32 = 0.0;
        if let Some(resource) = resource_manager.get(&self.resource_id)
            && let Some(image_data) = resource.data.downcast_ref::<ImageResource>().as_ref()
        {
            original_image_width = image_data.image.width() as f32;
            original_image_height = image_data.image.height() as f32;
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

#[derive(Clone)]
pub(crate) enum LayoutContext {
    Text(GummyTextContext),
    TextInput(GummyTextInputContext),
    Image(ImageContext),
    TinyVg(TinyVgContext),
}
//////////////////////////////////////////////////////////////////////////////
impl TextHashKey {
    pub fn new(known_dimensions: Size<Option<f32>>, available_space: Size<gummy::AvailableSpace>) -> Self {
        let available_space_width_u32: AvailableSpaceKey = match available_space.width {
            gummy::AvailableSpace::MinContent => AvailableSpaceKey::MinContent,
            gummy::AvailableSpace::MaxContent => AvailableSpaceKey::MaxContent,
            gummy::AvailableSpace::Definite(width) => AvailableSpaceKey::Definite(width.to_bits()),
        };
        let available_space_height_u32: AvailableSpaceKey = match available_space.height {
            gummy::AvailableSpace::MinContent => AvailableSpaceKey::MinContent,
            gummy::AvailableSpace::MaxContent => AvailableSpaceKey::MaxContent,
            gummy::AvailableSpace::Definite(height) => AvailableSpaceKey::Definite(height.to_bits()),
        };

        Self {
            width_constraint: known_dimensions.width.map(|w| w.to_bits()),
            height_constraint: known_dimensions.height.map(|h| h.to_bits()),
            available_space_width: available_space_width_u32,
            available_space_height: available_space_height_u32,
        }
    }

    pub fn available_space(&self) -> Size<gummy::AvailableSpace> {
        Size {
            width: match self.available_space_width {
                AvailableSpaceKey::Definite(width) => gummy::AvailableSpace::Definite(f32::from_bits(width)),
                AvailableSpaceKey::MinContent => gummy::AvailableSpace::MinContent,
                AvailableSpaceKey::MaxContent => gummy::AvailableSpace::MaxContent,
            },
            height: match self.available_space_height {
                AvailableSpaceKey::Definite(height) => gummy::AvailableSpace::Definite(f32::from_bits(height)),
                AvailableSpaceKey::MinContent => gummy::AvailableSpace::MinContent,
                AvailableSpaceKey::MaxContent => gummy::AvailableSpace::MaxContent,
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

#[derive(Clone)]
pub(crate) struct TinyVgContext {
    pub(crate) resource_id: ResourceId,
}

pub(crate) fn measure_content(
    known_dimensions: Size<Option<f32>>,
    available_space: Size<gummy::AvailableSpace>,
    node_context: Option<&mut LayoutContext>,
    elements: &mut RetainedElements,
    text_context: &mut TextContext,
    resource_manager: Arc<ResourceManager>,
    style: &gummy::Style,
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
        Some(LayoutContext::Text(gummy_text_context)) => elements
            .get_as_mut::<TextElement>(gummy_text_context.element)
            .measure(known_dimensions, available_space, text_context),
        Some(LayoutContext::TextInput(gummy_text_input_context)) => elements
            .get_as_mut::<TextInputElement>(gummy_text_input_context.element)
            .state
            .measure(known_dimensions, available_space, text_context),
        Some(LayoutContext::Image(image_context)) => {
            image_context.measure(known_dimensions, available_space, resource_manager, style)
        }
        Some(LayoutContext::TinyVg(tinyvg_context)) => {
            tinyvg_context.measure(known_dimensions, available_space, resource_manager, style)
        }
    }
}

impl TinyVgContext {
    pub(crate) fn new(resource_id: ResourceId) -> Self {
        Self { resource_id }
    }

    pub(crate) fn measure(
        &mut self,
        known_dimensions: Size<Option<f32>>,
        _available_space: Size<gummy::AvailableSpace>,
        resource_manager: Arc<ResourceManager>,
        _style: &gummy::Style,
    ) -> Size<f32> {
        let mut original_image_width: f32 = 0.0;
        let mut original_image_height: f32 = 0.0;

        if let Some(resource) = resource_manager.get(&self.resource_id)
            && let Some(tinyvg) = resource.data.downcast_ref::<TinyVg>()
        {
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
