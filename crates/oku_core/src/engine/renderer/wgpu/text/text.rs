use std::collections::HashMap;
use cosmic_text::{Buffer, BufferRef, CacheKey, Edit, FontSystem, SwashCache};
use cosmic_text::fontdb::ID;
use image::{ColorType, DynamicImage, GenericImage, GrayImage, Luma};
use tokio::sync::RwLockReadGuard;
use wgpu::{MultisampleState, RenderPass};
use wgpu::util::DeviceExt;
use crate::components::ComponentId;
use crate::elements::text::TextState;
use crate::elements::text_input::TextInputState;
use crate::engine::renderer::color::Color;
use crate::engine::renderer::renderer::{Rectangle, RenderCommand};
use crate::engine::renderer::wgpu::context::Context;
use crate::engine::renderer::wgpu::pipeline_2d::TextRenderInfo;
use crate::engine::renderer::wgpu::rectangle::PerFrameData;
use crate::engine::renderer::wgpu::render_group::{ClipRectangle, RenderGroup};
use crate::engine::renderer::wgpu::text::caching::{GlyphInfo, TextAtlas};
use crate::engine::renderer::wgpu::text::pipeline::{TextPipeline, TextPipelineConfig, DEFAULT_PIPELINE_CONFIG};
use crate::engine::renderer::wgpu::text::vertex::Vertex;
use crate::platform::resource_manager::ResourceManager;
use crate::reactive::state_store::StateStore;

pub(crate) struct TextRenderer {
    pub(crate) cached_pipelines: HashMap<TextPipelineConfig, TextPipeline>,
    pub(crate) text_areas: Vec<TextRenderInfo>,
    pub(crate) swash_cache: SwashCache,
    pub(crate) text_atlas: TextAtlas,
    pub(crate) vertices: Vec<Vertex>,
    pub(crate) indices: Vec<u32>,
}

impl TextRenderer {
    pub(crate) fn new(context: &Context) -> Self {

        let mut renderer = TextRenderer {
            cached_pipelines: Default::default(),
            text_areas: vec![],
            swash_cache: SwashCache::new(),
            text_atlas: TextAtlas::new(&context.device, 280, 400),
            vertices: vec![],
            indices: vec![],
        };

        renderer.cached_pipelines.insert(
            DEFAULT_PIPELINE_CONFIG,
            TextPipeline::new_pipeline_with_configuration(context, DEFAULT_PIPELINE_CONFIG)
        );

        renderer
    }

    pub(crate) fn build(&mut self, rectangle: Rectangle, component_id: ComponentId, color: Color) {
        self.text_areas.push(TextRenderInfo {
            element_id: component_id,
            rectangle,
            fill_color: color,
        });
    }

    pub(crate) fn prepare(&mut self, context: &Context, font_system: &mut FontSystem, element_state: &StateStore, clip_rectangle: ClipRectangle) -> PerFrameData {

        for text_area in self.text_areas.iter() { 
            if let Some(text_context) = element_state.storage.get(&text_area.element_id).unwrap().downcast_ref::<TextInputState>() {
                let text_buffer = match text_context.editor.buffer_ref() {
                    BufferRef::Owned(buffer) => buffer,
                    BufferRef::Borrowed(_) => panic!("Editor must own buffer."),
                    BufferRef::Arc(_) => panic!("Editor must own buffer."),
                };
            } else if let Some(text_context) = element_state.storage.get(&text_area.element_id).unwrap().downcast_ref::<TextState>() {
                // let buffer_glyphs = create_glyphs(&text_context.buffer, Color::BLACK);
                
                for run in text_context.buffer.layout_runs() {
                    for glyph in run.glyphs.iter() {
                        let physical_glyph = glyph.physical((0., 0.), 1.0);

                        let glyph_color = match glyph.color_opt {
                            Some(some) => some,
                            None => cosmic_text::Color::rgba(0, 0, 0, 255),
                        };

                        // Check if the image is available in the cache
                        let glyph_info: Option<GlyphInfo> = if let Some(glyph_info) = self.text_atlas.get_cached_glyph_info(physical_glyph.cache_key) {
                            Some(glyph_info)
                        } else if let Some(image) = self.swash_cache.get_image(font_system, physical_glyph.cache_key) {
                            self.text_atlas.add_glyph(image, physical_glyph.cache_key, &context.queue);

                            self.text_atlas.get_cached_glyph_info(physical_glyph.cache_key)
                        } else {
                            None
                        };
                        
                        if let Some(glyph_info) = glyph_info {
                            let rel_gylh_x = physical_glyph.x + glyph_info.swash_image_placement.left;
                            let rel_gylh_y = run.line_y as i32 + physical_glyph.y + (-glyph_info.swash_image_placement.top);
                            build_glyph_rectangle(Rectangle {
                                x: text_area.rectangle.x + rel_gylh_x as f32,
                                y: text_area.rectangle.y + rel_gylh_y as f32,
                                width: glyph_info.width as f32,
                                height: glyph_info.height as f32,
                            }, Color::BLACK, &mut self.vertices, &mut self.indices);   
                        }

                    }
                }

                
            } else {
                panic!("Unknown state provided to the renderer!");
            }
        }


        let vertex_buffer = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&self.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        PerFrameData {
            vertex_buffer,
            index_buffer
        }
    }
    
    pub(crate) fn draw(&mut self, render_pass: &mut RenderPass, per_frame_data: &PerFrameData) {
        if self.vertices.is_empty() {
            return;
        }
        let text_pipeline = self.cached_pipelines.get(&DEFAULT_PIPELINE_CONFIG).unwrap();
        render_pass.set_pipeline(&text_pipeline.pipeline);
        render_pass.set_bind_group(0, Some(&text_pipeline.global_bind_group), &[]);
        render_pass.set_vertex_buffer(0, per_frame_data.vertex_buffer.slice(..));
        render_pass.set_index_buffer(per_frame_data.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..(self.indices.len() as u32), 0, 0..1);
        self.vertices.clear();
        self.indices.clear();
    }
}

pub(crate) fn build_glyph_rectangle(rectangle: Rectangle, fill_color: Color, vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>) {
    let x = rectangle.x;
    let y = rectangle.y;
    let width = rectangle.width;
    let height = rectangle.height;

    let top_left = glam::vec4(x, y, 0.0, 1.0);
    let bottom_left = glam::vec4(x, y + height, 0.0, 1.0);
    let top_right = glam::vec4(x + width, y, 0.0, 1.0);
    let bottom_right = glam::vec4(x + width, y + height, 0.0, 1.0);

    let color = [fill_color.r, fill_color.g, fill_color.b, fill_color.a];

    vertices.append(&mut vec![
        Vertex {
            position: [top_left.x, top_left.y, top_left.z],
            size: [rectangle.width, rectangle.height],
            uv: [0.0, 0.0],
            background_color: [color[0], color[1], color[2], color[3]]
        },

        Vertex {
            position: [bottom_left.x, bottom_left.y, bottom_left.z],
            size: [rectangle.width, rectangle.height],
            uv: [0.0, 0.0],
            background_color: [color[0], color[1], color[2], color[3]]
        },

        Vertex {
            position: [top_right.x, top_right.y, top_right.z],
            size: [rectangle.width, rectangle.height],
            uv: [0.0, 0.0],
            background_color: [color[0], color[1], color[2], color[3]]
        },

        Vertex {
            position: [bottom_right.x, bottom_right.y, bottom_right.z],
            size: [rectangle.width, rectangle.height],
            uv: [0.0, 0.0],
            background_color: [color[0], color[1], color[2], color[3]]
        },
    ]);

    let next_starting_index: u32 = (indices.len() / 6) as u32 * 4;
    indices.append(&mut vec![
        next_starting_index,
        next_starting_index + 1,
        next_starting_index + 2,
        next_starting_index + 2,
        next_starting_index + 1,
        next_starting_index + 3,
    ]);
}