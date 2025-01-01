use crate::components::ComponentId;
use crate::elements::text::TextState;
use crate::elements::text_input::TextInputState;
use crate::geometry::Rectangle;
use crate::reactive::state_store::StateStore;
use crate::renderer::color::Color;
use crate::renderer::wgpu::context::Context;
use crate::renderer::wgpu::rectangle::PerFrameData;
use crate::renderer::wgpu::text::caching::{GlyphInfo, TextAtlas};
use crate::renderer::wgpu::text::pipeline::{TextPipeline, TextPipelineConfig, DEFAULT_TEXT_PIPELINE_CONFIG};
use crate::renderer::wgpu::text::vertex::TextVertex;
use cosmic_text::{BufferRef, Edit, FontSystem, SwashCache};
use std::collections::HashMap;
use wgpu::util::DeviceExt;
use wgpu::RenderPass;

pub struct TextRenderInfo {
    pub(crate) element_id: ComponentId,
    pub(crate) rectangle: Rectangle,
    pub(crate) fill_color: Color,
}

pub(crate) struct TextRenderer {
    pub(crate) cached_pipelines: HashMap<TextPipelineConfig, TextPipeline>,
    pub(crate) text_areas: Vec<TextRenderInfo>,
    pub(crate) swash_cache: SwashCache,
    pub(crate) text_atlas: TextAtlas,
    pub(crate) vertices: Vec<TextVertex>,
    pub(crate) indices: Vec<u32>,
}

impl TextRenderer {
    pub(crate) fn new(context: &Context) -> Self {
        let max_texture_size = context.device.limits().max_texture_dimension_2d;
        let mut renderer = TextRenderer {
            cached_pipelines: Default::default(),
            text_areas: vec![],
            swash_cache: SwashCache::new(),
            text_atlas: TextAtlas::new(&context.device, max_texture_size, max_texture_size),
            vertices: vec![],
            indices: vec![],
        };

        renderer.cached_pipelines.insert(
            DEFAULT_TEXT_PIPELINE_CONFIG,
            TextPipeline::new_pipeline_with_configuration(context, DEFAULT_TEXT_PIPELINE_CONFIG)
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

    pub(crate) fn prepare(&mut self, context: &Context, font_system: &mut FontSystem, element_state: &StateStore) -> PerFrameData {

        for text_area in self.text_areas.iter() { 
            if let Some(text_context) = element_state.storage.get(&text_area.element_id).unwrap().downcast_ref::<TextInputState>() {
                let text_buffer = match text_context.editor.buffer_ref() {
                    BufferRef::Owned(buffer) => buffer,
                    BufferRef::Borrowed(_) => panic!("Editor must own buffer."),
                    BufferRef::Arc(_) => panic!("Editor must own buffer."),
                };
            } else if let Some(text_context) = element_state.storage.get(&text_area.element_id).unwrap().downcast_ref::<TextState>() {
                for run in text_context.buffer.layout_runs() {
                    for glyph in run.glyphs.iter() {
                        let physical_glyph = glyph.physical((0., 0.), 1.0);

                        let glyph_color = match glyph.color_opt {
                            Some(some) => Color::rgba(some.r(), some.g(), some.b(), some.a()),
                            None => text_area.fill_color,
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
                            build_glyph_rectangle(self.text_atlas.texture_width, self.text_atlas.texture_height, glyph_info.clone(), Rectangle {
                                x: text_area.rectangle.x + rel_gylh_x as f32,
                                y: text_area.rectangle.y + rel_gylh_y as f32,
                                width: glyph_info.width as f32,
                                height: glyph_info.height as f32,
                            }, glyph_color, &mut self.vertices, &mut self.indices);   
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
    
    pub(crate) fn draw(&mut self, context: &mut Context, render_pass: &mut RenderPass, per_frame_data: &PerFrameData) {
        if self.vertices.is_empty() {
            return;
        }
        let text_pipeline = self.cached_pipelines.get(&DEFAULT_TEXT_PIPELINE_CONFIG).unwrap();

        let texture_bind_group_layout = context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });
        
        let texture_bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.text_atlas.texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.text_atlas.texture_sampler),
                },
            ],
            label: Some("oku_bind_group"),
        });
        
        render_pass.set_pipeline(&text_pipeline.pipeline);
        render_pass.set_bind_group(0, Some(&texture_bind_group), &[]);
        render_pass.set_bind_group(1, Some(&context.global_buffer.bind_group), &[]);
        render_pass.set_vertex_buffer(0, per_frame_data.vertex_buffer.slice(..));
        render_pass.set_index_buffer(per_frame_data.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..(self.indices.len() as u32), 0, 0..1);
        self.vertices.clear();
        self.indices.clear();
        self.text_areas.clear();
    }
}

pub(crate) fn build_glyph_rectangle(
                                    text_atlas_texture_width: u32,
                                    text_atlas_texture_height: u32,
                                    glyph_info: GlyphInfo,
                                    rectangle: Rectangle,
                                    fill_color: Color,
                                    vertices: &mut Vec<TextVertex>,
                                    indices: &mut Vec<u32>) {
    let x = rectangle.x;
    let y = rectangle.y;
    let width = rectangle.width;
    let height = rectangle.height;

    let top_left = glam::vec4(x, y, 0.0, 1.0);
    let bottom_left = glam::vec4(x, y + height, 0.0, 1.0);
    let top_right = glam::vec4(x + width, y, 0.0, 1.0);
    let bottom_right = glam::vec4(x + width, y + height, 0.0, 1.0);

    let color = [fill_color.r, fill_color.g, fill_color.b, fill_color.a];
    
    let left_text_corod = glyph_info.texture_coordinate_x as f32 / text_atlas_texture_width as f32;
    let top_tex_coord = glyph_info.texture_coordinate_y as f32 / text_atlas_texture_height as f32;
    
    vertices.append(&mut vec![
        TextVertex {
            position: [top_left.x, top_left.y, top_left.z],
            uv: [left_text_corod, top_tex_coord],
            background_color: [color[0], color[1], color[2], color[3]],
            content_type: glyph_info.content_type
        },

        TextVertex {
            position: [bottom_left.x, bottom_left.y, bottom_left.z],
            uv: [left_text_corod, top_tex_coord + (rectangle.height / text_atlas_texture_height as f32)],
            background_color: [color[0], color[1], color[2], color[3]],
            content_type: glyph_info.content_type
        },

        TextVertex {
            position: [top_right.x, top_right.y, top_right.z],
            uv: [left_text_corod + (rectangle.width / text_atlas_texture_width as f32), top_tex_coord],
            background_color: [color[0], color[1], color[2], color[3]],
            content_type: glyph_info.content_type
        },

        TextVertex {
            position: [bottom_right.x, bottom_right.y, bottom_right.z],
            uv: [left_text_corod + (rectangle.width / text_atlas_texture_width as f32), top_tex_coord + (rectangle.height / text_atlas_texture_height as f32)],
            background_color: [color[0], color[1], color[2], color[3]],
            content_type: glyph_info.content_type
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