use crate::elements::element::ElementState;
use crate::engine::renderer::color::Color;
use crate::engine::renderer::renderer::Rectangle;
use crate::engine::renderer::wgpu::camera::Camera;
use crate::engine::renderer::wgpu::context::Context;
use crate::engine::renderer::wgpu::texture::Texture;
use crate::engine::renderer::wgpu::uniform::GlobalUniform;
use crate::engine::renderer::wgpu::vertex::Vertex;
use crate::platform::resource_manager::{ResourceIdentifier, ResourceManager};
use crate::components::component::{ComponentId, GenericUserState};
use crate::elements::text::TextState;
use cosmic_text::{BufferRef, Edit, FontSystem, SwashCache};
use glyphon::{TextArea, TextBounds};
use std::collections::HashMap;
use tokio::sync::RwLockReadGuard;
use wgpu::util::DeviceExt;
use crate::platform::resource_manager::resource::Resource;
use crate::elements::text_input::TextInputState;

fn bind_group_from_2d_texture(
    device: &wgpu::Device,
    texture_bind_group_layout: &wgpu::BindGroupLayout,
    texture: &Texture,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: texture_bind_group_layout,
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
    })
}

pub struct RectangleBatch {
    texture_path: Option<ResourceIdentifier>,
    rectangle_vertices: Vec<Vertex>,
    rectangle_indices: Vec<u32>,
}

pub struct TextRenderInfo {
    element_id: ComponentId,
    rectangle: Rectangle,
    fill_color: Color
}

pub struct Pipeline2D {
    pub(crate) camera: Camera,
    pub(crate) global_uniform: GlobalUniform,
    pub(crate) global_buffer: wgpu::Buffer,
    pub(crate) global_bind_group: wgpu::BindGroup,
    pub(crate) pipeline: wgpu::RenderPipeline,
    pub(crate) texture_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) rectangle_batch: Vec<RectangleBatch>,
    pub(crate) text_areas: Vec<TextRenderInfo>,
    pub(crate) textures: HashMap<ResourceIdentifier, Texture>,
    pub(crate) swash_cache: SwashCache
}

impl Pipeline2D {
    pub fn new(context: &Context) -> Pipeline2D {
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

        let camera = Camera {
            width: context.surface_config.width as f32,
            height: context.surface_config.height as f32,
            z_near: 0.0,
            z_far: 100.0,
        };

        let mut global_uniform = GlobalUniform::new();
        global_uniform.set_view_proj(&camera);

        let global_buffer = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Global Buffer"),
            contents: bytemuck::bytes_of(&global_uniform),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let global_bind_group_layout = context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("global_bind_group_layout"),
        });

        let global_bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &global_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: global_buffer.as_entire_binding(),
            }],
            label: Some("global_bind_group"),
        });

        let shader = context.device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let render_pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &global_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Vertex::description()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: context.surface_config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),

            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Pipeline2D {
            camera,
            global_uniform,
            global_buffer,
            global_bind_group,
            pipeline: render_pipeline,
            texture_bind_group_layout,
            rectangle_batch: vec![],
            text_areas: vec![],
            textures: Default::default(),
            swash_cache: SwashCache::new(),
        }
    }

    pub fn draw_rect(&mut self, rectangle: Rectangle, fill_color: Color) {
        let x = rectangle.x;
        let y = rectangle.y;
        let width = rectangle.width;
        let height = rectangle.height;

        let top_left = glam::vec4(x, y, 0.0, 1.0);
        let bottom_left = glam::vec4(x, y + height, 0.0, 1.0);
        let top_right = glam::vec4(x + width, y, 0.0, 1.0);
        let bottom_right = glam::vec4(x + width, y + height, 0.0, 1.0);

        let color = [fill_color.r, fill_color.g, fill_color.b, fill_color.a];

        // Append to the current batch or create a new one.
        let current_batch =
            if !self.rectangle_batch.is_empty() && self.rectangle_batch.last().unwrap().texture_path.is_none() {
                self.rectangle_batch.last_mut().unwrap()
            } else {
                self.rectangle_batch.push(RectangleBatch {
                    texture_path: None,
                    rectangle_vertices: vec![],
                    rectangle_indices: vec![],
                });
                self.rectangle_batch.last_mut().unwrap()
            };

        current_batch.rectangle_vertices.append(&mut vec![
            Vertex {
                position: [top_left.x, top_left.y, top_left.z],
                color,
                tex_coords: [0.0, 0.0],
            },
            Vertex {
                position: [bottom_left.x, bottom_left.y, bottom_left.z],
                color,
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                position: [top_right.x, top_right.y, top_right.z],
                color,
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                position: [bottom_right.x, bottom_right.y, bottom_right.z],
                color,
                tex_coords: [1.0, 1.0],
            },
        ]);

        let next_starting_index: u32 = (current_batch.rectangle_indices.len() / 6) as u32 * 4;
        current_batch.rectangle_indices.append(&mut vec![
            next_starting_index,
            next_starting_index + 1,
            next_starting_index + 2,
            next_starting_index + 2,
            next_starting_index + 1,
            next_starting_index + 3,
        ]);
    }

    pub fn draw_rect_outline(&mut self, rectangle: Rectangle, outline_color: Color) {
        // vertical left, vertical right, top horizontal, bottom horizontal

        let thickness: f32 = 2.0;

        let vertical_left = Rectangle::new(rectangle.x, rectangle.y, thickness, rectangle.height + thickness);

        let vertical_right = Rectangle::new(
            rectangle.x + rectangle.width - thickness,
            rectangle.y,
            thickness,
            rectangle.height + thickness,
        );

        let horizontal_top = Rectangle::new(rectangle.x, rectangle.y, rectangle.width, thickness);

        let horizontal_bottom = Rectangle::new(rectangle.x, rectangle.y + rectangle.height, rectangle.width, thickness);

        self.draw_rect(vertical_left, outline_color);
        self.draw_rect(vertical_right, outline_color);
        self.draw_rect(horizontal_top, outline_color);
        self.draw_rect(horizontal_bottom, outline_color);
    }

    pub(crate) fn draw_text(&mut self, element_id: ComponentId, rectangle: Rectangle, fill_color: Color) {
        self.text_areas.push(TextRenderInfo {
            element_id,
            rectangle,
            fill_color
        });
    }

    pub fn draw_image(&mut self, rectangle: Rectangle, resource_identifier: ResourceIdentifier) {
        let x = rectangle.x;
        let y = rectangle.y;
        let width = rectangle.width;
        let height = rectangle.height;

        let top_left = glam::vec4(x, y, 0.0, 1.0);
        let bottom_left = glam::vec4(x, y + height, 0.0, 1.0);
        let top_right = glam::vec4(x + width, y, 0.0, 1.0);
        let bottom_right = glam::vec4(x + width, y + height, 0.0, 1.0);

        let color = [255.0, 255.0, 255.0, 255.0];

        // For now, always create a new batch when rendering an image
        let current_batch = {
            self.rectangle_batch.push(RectangleBatch {
                texture_path: Some(resource_identifier),
                rectangle_vertices: vec![],
                rectangle_indices: vec![],
            });
            self.rectangle_batch.last_mut().unwrap()
        };

        current_batch.rectangle_vertices.append(&mut vec![
            Vertex {
                position: [top_left.x, top_left.y, top_left.z],
                color,
                tex_coords: [0.0, 0.0],
            },
            Vertex {
                position: [bottom_left.x, bottom_left.y, bottom_left.z],
                color,
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                position: [top_right.x, top_right.y, top_right.z],
                color,
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                position: [bottom_right.x, bottom_right.y, bottom_right.z],
                color,
                tex_coords: [1.0, 1.0],
            },
        ]);

        let next_starting_index: u32 = (current_batch.rectangle_indices.len() / 6) as u32 * 4;
        current_batch.rectangle_indices.append(&mut vec![
            next_starting_index,
            next_starting_index + 1,
            next_starting_index + 2,
            next_starting_index + 2,
            next_starting_index + 1,
            next_starting_index + 3,
        ]);
    }

    pub fn submit(
        &mut self,
        context: &mut Context<'_>,
        resource_manager: RwLockReadGuard<'_, ResourceManager>,
        font_system: &mut FontSystem,
        element_state: &HashMap<ComponentId, Box<ElementState>>,
    ) {
        let mut encoder = context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let output = context.surface.get_current_texture().unwrap();
        let texture_view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut vertex_buffers: Vec<wgpu::Buffer> = vec![];
        let mut index_buffers: Vec<wgpu::Buffer> = vec![];
        let mut bind_groups: Vec<wgpu::BindGroup> = vec![];

        {
            for batch in self.rectangle_batch.iter_mut() {
                // Get the batch texture or use the default white texture if we cannot find the batch texture.
                let texture = if let Some(texture_path) = batch.texture_path.clone() {
                    // RESOURCE_MANAGER.resources.

                    if let Some(texture) = self.textures.get(&texture_path) {
                        texture
                    } else {
                        // If we were given an image path, but it isn't in our texture cache then try to load the image from the filesystem.
                        // Fallback to the default texture if that fails.
                        let mut rectangle_texture: &Texture = &context.default_texture;

                        let resource_identifier = &batch.texture_path.clone().unwrap();
                        let resource = resource_manager.resources.get(resource_identifier);

                        if let Some(Resource::Image(resource)) = resource {
                            let label = resource.common_data.resource_identifier.to_string();
                            let texture = Texture::from_image(&context.device, &context.queue, &resource.image, Some(label.as_str()));
                            if let Some(texture) = texture {
                                self.textures.insert(texture_path.clone(), texture);
                                rectangle_texture = self.textures.get(&texture_path.clone()).unwrap();
                            }
                        }

                        rectangle_texture
                    }
                } else {
                    &context.default_texture
                };

                let oku_bind_group =
                    bind_group_from_2d_texture(&context.device, &self.texture_bind_group_layout, texture);

                let vertex_buffer = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&batch.rectangle_vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });

                let index_buffer = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(&batch.rectangle_indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

                // Don't cache the textures for now.
                // self.textures.clear();

                vertex_buffers.push(vertex_buffer);
                index_buffers.push(index_buffer);
                bind_groups.push(oku_bind_group);
            }
        }

        {
            context.glyphon_viewport.update(
                &context.queue,
                glyphon::Resolution {
                    width: context.surface_config.width,
                    height: context.surface_config.height,
                },
            );

            let mut text_areas: Vec<TextArea> = Vec::new();

            for text_area in self.text_areas.iter() {

                if let Some(text_context) = element_state.get(&text_area.element_id).unwrap().downcast_ref::<TextInputState>() {

                    let text_buffer = match text_context.editor.buffer_ref() {
                        BufferRef::Owned(buffer) => buffer,
                        BufferRef::Borrowed(_) => panic!("Editor must own buffer."),
                        BufferRef::Arc(_) =>  panic!("Editor must own buffer.")
                    };
                    
                    let text_area_position = glam::vec4(text_area.rectangle.x, text_area.rectangle.y, 0.0, 1.0);

                    text_areas.push(TextArea {
                        buffer: text_buffer,
                        left: text_area_position.x,
                        top: text_area_position.y,
                        scale: 1.0,
                        bounds: TextBounds {
                            left: text_area_position.x as i32,
                            top: text_area_position.y as i32,
                            right: (text_area_position.x + text_area.rectangle.width) as i32,
                            bottom: (text_area_position.y + text_area.rectangle.height) as i32,
                        },
                        default_color: glyphon::Color::rgba(
                            text_area.fill_color.r_u8(),
                            text_area.fill_color.g_u8(),
                            text_area.fill_color.b_u8(),
                            text_area.fill_color.a_u8(),
                        ),
                        custom_glyphs: &[],
                    });
                } else if let Some(text_context) = element_state.get(&text_area.element_id).unwrap().downcast_ref::<TextState>() {

                    let text_buffer = &text_context.buffer;
                    let text_area_position = glam::vec4(text_area.rectangle.x, text_area.rectangle.y, 0.0, 1.0);

                    text_areas.push(TextArea {
                        buffer: text_buffer,
                        left: text_area_position.x,
                        top: text_area_position.y,
                        scale: 1.0,
                        bounds: TextBounds {
                            left: text_area_position.x as i32,
                            top: text_area_position.y as i32,
                            right: (text_area_position.x + text_area.rectangle.width) as i32,
                            bottom: (text_area_position.y + text_area.rectangle.height) as i32,
                        },
                        default_color: glyphon::Color::rgba(
                            text_area.fill_color.r_u8(),
                            text_area.fill_color.g_u8(),
                            text_area.fill_color.b_u8(),
                            text_area.fill_color.a_u8(),
                        ),
                        custom_glyphs: &[],
                    });
                } else {
                    panic!("Unknown state provided to the renderer!");
                }
            }

            context
                .glyphon_text_renderer
                .prepare(
                    &context.device,
                    &context.queue,
                    font_system,
                    &mut context.glyphon_atlas,
                    &context.glyphon_viewport,
                    text_areas,
                    &mut self.swash_cache,
                )
                .unwrap();
        }

        //let r = ((self.surface_clear_color.r / 255.0 + 0.055) / 1.055).powf(2.4);
        //let g = ((self.surface_clear_color.g / 255.0 + 0.055) / 1.055).powf(2.4);
        //let b = ((self.surface_clear_color.b / 255.0 + 0.055) / 1.055).powf(2.4);
        let r = context.surface_clear_color.r / 255.0;
        let g = context.surface_clear_color.g / 255.0;
        let b = context.surface_clear_color.b / 255.0;

        {
            let mut _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: r as f64,
                            g: g as f64,
                            b: b as f64,
                            a: context.surface_clear_color.a as f64 / 255.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            for (index, batch) in self.rectangle_batch.iter_mut().enumerate() {
                {
                    _render_pass.set_pipeline(&self.pipeline);
                    _render_pass.set_bind_group(0, Some(&bind_groups[index]), &[]);
                    _render_pass.set_bind_group(1, Some(&self.global_bind_group), &[]);
                    _render_pass.set_vertex_buffer(0, vertex_buffers.get(index).unwrap().slice(..));
                    _render_pass
                        .set_index_buffer(index_buffers.get(index).unwrap().slice(..), wgpu::IndexFormat::Uint32);
                    _render_pass.draw_indexed(0..(batch.rectangle_indices.len() as u32), 0, 0..1);
                }
            }

            context
                .glyphon_text_renderer
                .render(&context.glyphon_atlas, &context.glyphon_viewport, &mut _render_pass)
                .unwrap();
        }

        context.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        context.glyphon_atlas.trim();
        self.rectangle_batch.clear();
        self.text_areas.clear();
    }
}
