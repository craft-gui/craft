use crate::elements::element::ElementState;
use crate::platform::resource_manager::resource::Resource;
use crate::platform::resource_manager::{ResourceIdentifier, ResourceManager};
use crate::components::component::{ComponentId, GenericUserState};
use crate::elements::text::TextState;
use crate::elements::text_input::TextInputState;
use cosmic_text::{Attrs, Buffer, Edit, FontSystem, Metrics, Shaping, Editor, Cursor, Action, Motion};
use std::collections::HashMap;
use taffy::Size;
use tokio::sync::RwLockReadGuard;

pub struct TaffyTextContext<'a> {
    pub id: ComponentId,
    pub metrics: Metrics,

    pub text: String,
    pub text_hash: u64,
    pub attributes: Attrs<'a>,
}

impl<'a> TaffyTextContext<'a> {
    pub fn new(id: ComponentId, metrics: Metrics, text: String, text_hash: u64, attributes: Attrs<'a>) -> Self {
        Self {
            id,
            metrics,
            text,
            text_hash,
            attributes,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct MetricsDummy {
    /// Font size in pixels
    pub font_size: u32,
    /// Line height in pixels
    pub line_height: u32,
}

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct TextHashKey {
    pub text_hash: u64,
    pub width_constraint: Option<u32>,
    pub height_constraint: Option<u32>,
    pub available_space_width: AvailableSpace,
    pub available_space_height: AvailableSpace,
    pub metrics: MetricsDummy,
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

pub struct ImageContext {
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
        if let Some(image_resource) = resource_manager.resources.get(&self.resource_identifier) {
            match image_resource {
                Resource::Image(image_data) => {
                    original_image_width = image_data.width as f32;
                    original_image_height = image_data.height as f32;
                }
            }
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

pub enum LayoutContext<'a> {
    Text(TaffyTextContext<'a>),
    TextInput(TaffyTextInputContext<'a>),
    Image(ImageContext),
}

pub fn measure_content(
    element_state: &mut HashMap<ComponentId, Box<ElementState>>,
    known_dimensions: Size<Option<f32>>,
    available_space: Size<taffy::AvailableSpace>,
    node_context: Option<&mut LayoutContext>,
    font_system: &mut FontSystem,
    resource_manager: &RwLockReadGuard<ResourceManager>,
    style: &taffy::Style,
) -> Size<f32> {
    match node_context {
        None => Size::ZERO,
        Some(LayoutContext::Text(taffy_text_context)) => {
            let cosmic_text_content: &mut TextState = if let Some(cosmic_text_content) =
                element_state.get_mut(&taffy_text_context.id).unwrap().downcast_mut()
            {
                let cosmic_text_content: &mut TextState = cosmic_text_content;

                if cosmic_text_content.text_hash != taffy_text_context.text_hash
                    || cosmic_text_content.metrics != taffy_text_context.metrics
                {
                    cosmic_text_content.text_hash = taffy_text_context.text_hash;
                    cosmic_text_content.metrics = taffy_text_context.metrics;
                    cosmic_text_content.buffer.set_metrics(font_system, cosmic_text_content.metrics);
                    cosmic_text_content.buffer.set_text(
                        font_system,
                        &taffy_text_context.text,
                        taffy_text_context.attributes,
                        Shaping::Advanced,
                    );
                }
                cosmic_text_content
            } else {
                let mut buffer = Buffer::new(font_system, taffy_text_context.metrics);
                buffer.set_text(
                    font_system,
                    &taffy_text_context.text,
                    taffy_text_context.attributes,
                    Shaping::Advanced,
                );

                let cosmic_text_content = TextState::new(
                    taffy_text_context.id,
                    taffy_text_context.metrics,
                    taffy_text_context.text_hash,
                    buffer,
                    taffy_text_context.attributes.color_opt
                );

                element_state.insert(taffy_text_context.id, Box::new(cosmic_text_content));
                element_state.get_mut(&taffy_text_context.id).unwrap().downcast_mut().unwrap()
            };
            cosmic_text_content.measure(
                known_dimensions,
                available_space,
                font_system,
                taffy_text_context.text_hash,
                taffy_text_context.metrics,
            )
        }
        Some(LayoutContext::Image(image_context)) => {
            image_context.measure(known_dimensions, available_space, resource_manager, style)
        },
        Some(LayoutContext::TextInput(taffy_text_input_context)) => {
            let cosmic_text_content: &mut TextInputState = if let Some(cosmic_text_content) =
                element_state.get_mut(&taffy_text_input_context.id).unwrap().downcast_mut()
            {
                let cosmic_text_content: &mut TextInputState = cosmic_text_content;

                if cosmic_text_content.text_hash != taffy_text_input_context.text_hash
                    || cosmic_text_content.metrics != taffy_text_input_context.metrics
                {
                    cosmic_text_content.text_hash = taffy_text_input_context.text_hash;
                    cosmic_text_content.metrics = taffy_text_input_context.metrics;
                    
                    cosmic_text_content.editor.with_buffer_mut(|buffer| {
                        buffer.set_metrics(font_system, cosmic_text_content.metrics);
                        buffer.set_text(
                            font_system,
                            &taffy_text_input_context.text,
                            taffy_text_input_context.attributes,
                            Shaping::Advanced,
                        );   
                    });
                }
                cosmic_text_content
            } else {

                let mut font_system = FontSystem::new();
                
                let buffer = Buffer::new(
                    &mut font_system,
                    taffy_text_input_context.metrics,
                );
                let mut editor = Editor::new(buffer);
                editor.borrow_with(&mut font_system);
                
                editor.with_buffer_mut(|buffer| {
                    buffer.set_text(&mut font_system, &taffy_text_input_context.text, taffy_text_input_context.attributes, Shaping::Advanced)
                });
                editor.action(&mut font_system, Action::Motion(Motion::End));

                let cosmic_text_content = TextInputState::new(
                    taffy_text_input_context.id,
                    taffy_text_input_context.metrics,
                    taffy_text_input_context.text_hash,
                    font_system,
                    editor,
                    taffy_text_input_context.attributes.color_opt
                );

                element_state.insert(taffy_text_input_context.id, Box::new(cosmic_text_content));
                element_state.get_mut(&taffy_text_input_context.id).unwrap().downcast_mut().unwrap()
            };

            cosmic_text_content.measure(
                known_dimensions,
                available_space,
                font_system,
                taffy_text_input_context.text_hash,
                taffy_text_input_context.metrics,
            )
        }
    }
}


//////////////////////////////////////////////////////////////////////////////

pub struct TaffyTextInputContext<'a> {
    pub id: ComponentId,
    pub metrics: Metrics,

    pub text: String,
    pub text_hash: u64,
    pub attributes: Attrs<'a>,
}

impl<'a> TaffyTextInputContext<'a> {
    pub fn new(id: ComponentId, metrics: Metrics, text: String, text_hash: u64, attributes: Attrs<'a>) -> Self {
        Self {
            id,
            metrics,
            text,
            text_hash,
            attributes,
        }
    }
}