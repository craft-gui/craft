use crate::components::component::{ComponentId, ComponentOrElement};
use crate::elements::text::{TextFragment, TextState};
use crate::reactive::element_state_store::ElementStateStore;
use crate::resource_manager::resource::Resource;
use crate::resource_manager::{ResourceIdentifier, ResourceManager};
use parley::{Alignment, AlignmentOptions, FontContext, FontStack, Layout, TextStyle, TreeBuilder};
use peniko::color::palette;
use peniko::Brush;

use taffy::Size;

use crate::elements::Span;
use tokio::sync::RwLockReadGuard;
use crate::elements;
use crate::elements::element::Element;
use crate::style::Style;

pub struct TaffyTextContext {
    pub id: ComponentId,
}

impl<'a> TaffyTextContext {
    pub fn new(id: ComponentId) -> Self {
        Self {
            id,
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
    
    pub font_family_length: u8,
    pub font_family: [u8; 64]
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
    Image(ImageContext),
}

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
    if let Size { width: Some(width), height: Some(height) } = known_dimensions {
        return Size { width, height };
    }

    match node_context {
        None => Size::ZERO,
        Some(LayoutContext::Text(taffy_text_context)) => {
            let state: &mut TextState = element_state.storage.get_mut(&taffy_text_context.id).unwrap().data.downcast_mut().unwrap();

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

            fn style_to_parley_style<'a>(style: &Style) -> TextStyle<'a, Brush> {
                let text_brush = Brush::Solid(style.color());
                let font_stack = FontStack::from("system-ui");
                TextStyle {
                    brush: text_brush,
                    font_stack,
                    line_height: 1.3,
                    font_size: style.font_size(),
                    ..Default::default()
                }
            }

            let root_style = style_to_parley_style(&state.style);
            let mut builder: TreeBuilder<Brush> = font_layout_context.tree_builder(font_context, 1.0, &root_style);

            for fragment in state.fragments.iter() {
                match fragment {
                    TextFragment::String(str) => {
                        builder.push_text(str);
                    }
                    TextFragment::Span(span_index) => {
                        let span = state.children.get(*span_index as usize).unwrap();

                        match &span.component {
                            ComponentOrElement::Element(ele) => {
                                let ele = &*ele.internal;

                                if let Some(span) = ele.as_any().downcast_ref::<Span>() {
                                    builder.push_style_span(style_to_parley_style(span.style()));
                                    builder.push_text(&span.text);
                                    builder.pop_style_span();
                                }
                            }
                            _ => {}
                        }
                    }
                    TextFragment::InlineComponentSpecification(inline) => {}
                }
            }
            
            
            // Build the builder into a Layout
            let (mut layout, _text): (Layout<Brush>, String) = builder.build();
            layout.break_all_lines(width_constraint);
            layout.align(width_constraint, Alignment::Start, AlignmentOptions::default());

            let width = layout.width().ceil() as u32;
            let height = layout.height().ceil() as u32;

            state.layout = layout;
            
            Size {
                width: width as f32,
                height: height as f32,
            }
            
        }
        Some(LayoutContext::Image(image_context)) => {
            image_context.measure(known_dimensions, available_space, resource_manager, style)
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

pub struct TaffyTextInputContext {
    pub id: ComponentId,
}

impl<'a> TaffyTextInputContext {
    pub fn new(id: ComponentId) -> Self {
        Self {
            id,
        }
    }
}
