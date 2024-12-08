use crate::resource_manager::resource::Resource;
use std::collections::HashMap;
use cosmic_text::{Buffer, BufferRef, CacheKey, Edit, FontSystem, SwashCache};
use cosmic_text::fontdb::ID;
use image::{ColorType, DynamicImage, GenericImage, GrayImage, Luma};
use tokio::sync::RwLockReadGuard;
use wgpu::{MultisampleState, RenderPass};
use wgpu::util::DeviceExt;
use crate::components::ComponentId;
use crate::renderer::color::Color;
use crate::renderer::renderer::{Rectangle, RenderCommand};
use crate::renderer::wgpu::context::Context;
use crate::renderer::wgpu::image::pipeline::{ImagePipeline, ImagePipelineConfig, DEFAULT_IMAGE_PIPELINE_CONFIG};
use crate::renderer::wgpu::image::vertex::ImageVertex;
use crate::renderer::wgpu::rectangle::{ImagePerFrameData, PerFrameData};
use crate::renderer::wgpu::render_group::{ClipRectangle, RenderGroup};
use crate::renderer::wgpu::texture::Texture;
use crate::resource_manager::{ResourceIdentifier, ResourceManager};
use crate::reactive::state_store::StateStore;

pub struct ImageBatch {
    texture_path: Option<ResourceIdentifier>,
    vertices: Vec<ImageVertex>,
    indices: Vec<u32>,
}

pub(crate) struct ImageRenderer {
    pub(crate) cached_textures: HashMap<ResourceIdentifier, Texture>,
    pub(crate) cached_pipelines: HashMap<ImagePipelineConfig, ImagePipeline>,
    pub(crate) image_batch: Vec<ImageBatch>,
}

impl ImageRenderer {
    pub(crate) fn new(context: &Context) -> Self {
        let mut renderer = ImageRenderer {
            cached_textures: Default::default(),
            cached_pipelines: Default::default(),
            image_batch: vec![],
        };

        renderer.cached_pipelines.insert(
            DEFAULT_IMAGE_PIPELINE_CONFIG,
            ImagePipeline::new_pipeline_with_configuration(context, DEFAULT_IMAGE_PIPELINE_CONFIG)
        );

        renderer
    }

    pub(crate) fn build(&mut self, rectangle: Rectangle, resource_identifier: ResourceIdentifier, color: Color) {
        self.image_batch.push(ImageBatch {
            texture_path: Some(resource_identifier),
            vertices: vec![],
            indices: vec![],
        });
        
        let current_batch = self.image_batch.last_mut().unwrap();
        ImageRenderer::build_texture_rectangle(rectangle, color, &mut current_batch.vertices, &mut current_batch.indices);
    }

    pub(crate) fn prepare(&mut self, context: &Context, font_system: &mut FontSystem, element_state: &StateStore, clip_rectangle: ClipRectangle) -> ImagePerFrameData {

        let mut vertex_buffers = Vec::new();
        let mut index_buffers = Vec::new();
        let mut resource_identifiers: Vec<Option<ResourceIdentifier>> = Vec::new();
        for batch in self.image_batch.iter_mut() {
            let vertex_buffer = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&batch.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

            let index_buffer = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&batch.indices),
                usage: wgpu::BufferUsages::INDEX,
            });
            vertex_buffers.push(vertex_buffer);
            index_buffers.push(index_buffer);
            resource_identifiers.push(batch.texture_path.clone());
        }

        ImagePerFrameData {
            vertex_buffers,
            index_buffers,
            resource_identifiers
        }
    }

    pub(crate) fn draw(&mut self, context: &mut Context, resource_manager: &RwLockReadGuard<ResourceManager>, render_pass: &mut RenderPass, per_frame_data: &ImagePerFrameData) {
        if self.image_batch.is_empty() {
            return;
        }

        let image_pipeline = self.cached_pipelines.get(&DEFAULT_IMAGE_PIPELINE_CONFIG).unwrap();

        for (index, batch) in self.image_batch.iter().enumerate() {
            
            let texture_path = per_frame_data.resource_identifiers.get(index).unwrap();
            let vertex_buffer = per_frame_data.vertex_buffers.get(index).unwrap();
            let index_buffer = per_frame_data.index_buffers.get(index).unwrap();
            
            
            // Get the batch texture or use the default white texture if we cannot find the batch texture.
            let texture = if let Some(texture_path) = texture_path {
                
                if let Some(texture) = self.cached_textures.get(&texture_path) {
                    texture
                } else {
                    // If we were given an image path, but it isn't in our texture cache then try to load the image from the filesystem.
                    // Fallback to the default texture if that fails.

                    let resource_identifier = &batch.texture_path.clone().unwrap();
                    let resource = resource_manager.resources.get(resource_identifier);

                    let texture = if let Some(Resource::Image(resource)) = resource {
                        let label = resource.common_data.resource_identifier.to_string();
                        let texture = Texture::from_image(
                            &context.device,
                            &context.queue,
                            &resource.image,
                            Some(label.as_str()),
                        ).unwrap();
                        self.cached_textures.insert(texture_path.clone(), texture);
                        self.cached_textures.get(&texture_path.clone()).unwrap()
                    } else {
                        panic!("Handle invalid textures");
                    };

                    texture
                }
            } else {
                panic!("Handle invalid textures");
            };

            render_pass.set_pipeline(&image_pipeline.pipeline);

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
                        resource: wgpu::BindingResource::TextureView(&texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&texture.sampler),
                    },
                ],
                label: Some("oku_bind_group"),
            });
            
            render_pass.set_bind_group(0, Some(&texture_bind_group), &[]);
            render_pass.set_bind_group(1, Some(&context.global_buffer.bind_group), &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..(batch.indices.len() as u32), 0, 0..1);
        }
        self.image_batch.clear();
    }

    pub(crate) fn build_texture_rectangle(rectangle: Rectangle, fill_color: Color, vertices: &mut Vec<ImageVertex>, indices: &mut Vec<u32>) {
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
            ImageVertex {
                position: [top_left.x, top_left.y, top_left.z],
                uv: [0.0, 0.0],
                background_color: [color[0], color[1], color[2], color[3]],
            },
            ImageVertex {
                position: [bottom_left.x, bottom_left.y, bottom_left.z],
                uv: [0.0, 1.0],
                background_color: [color[0], color[1], color[2], color[3]],
            },
            ImageVertex {
                position: [top_right.x, top_right.y, top_right.z],
                uv: [1.0, 0.0],
                background_color: [color[0], color[1], color[2], color[3]],
            },
            ImageVertex {
                position: [bottom_right.x, bottom_right.y, bottom_right.z],
                uv: [1.0, 1.0],
                background_color: [color[0], color[1], color[2], color[3]],
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
}