use std::collections::HashMap;
use wgpu::RenderPass;
use wgpu::util::DeviceExt;
use crate::renderer::color::Color;
use crate::renderer::renderer::Rectangle;
use crate::renderer::wgpu::context::Context;
use crate::renderer::wgpu::rectangle::pipeline::{RectanglePipeline, RectanglePipelineConfig, DEFAULT_RECTANGLE_PIPELINE_CONFIG};
use crate::renderer::wgpu::rectangle::vertex::RectangleVertex;
use crate::resource_manager::ResourceIdentifier;

pub(crate) mod pipeline;
mod vertex;

pub struct RectangleRenderer {
    pub(crate) cached_pipelines: HashMap<RectanglePipelineConfig, RectanglePipeline>,
    pub(crate) vertices: Vec<RectangleVertex>,
    pub(crate) indices: Vec<u32>,
}

pub struct ImagePerFrameData {
    pub(crate) vertex_buffers: Vec<wgpu::Buffer>,
    pub(crate) index_buffers: Vec<wgpu::Buffer>,
    pub resource_identifiers: Vec<Option<ResourceIdentifier>>,
}

pub struct PerFrameData {
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: wgpu::Buffer,
}

impl RectangleRenderer {
    pub fn new(context: &Context) -> Self {
        let mut renderer = RectangleRenderer {
            cached_pipelines: HashMap::new(),
            vertices: vec![],
            indices: vec![],
        };
        
        renderer.cached_pipelines.insert(
            DEFAULT_RECTANGLE_PIPELINE_CONFIG,
            RectanglePipeline::new_pipeline_with_configuration(context, DEFAULT_RECTANGLE_PIPELINE_CONFIG)
        );
        
        renderer
    }
    pub fn build(&mut self, rectangle: Rectangle, fill_color: Color) {
        let x = rectangle.x;
        let y = rectangle.y;
        let width = rectangle.width;
        let height = rectangle.height;

        let top_left = glam::vec4(x, y, 0.0, 1.0);
        let bottom_left = glam::vec4(x, y + height, 0.0, 1.0);
        let top_right = glam::vec4(x + width, y, 0.0, 1.0);
        let bottom_right = glam::vec4(x + width, y + height, 0.0, 1.0);

        let color = [fill_color.r, fill_color.g, fill_color.b, fill_color.a];
        let mut border_color =[
            [0.0, 0.0, 0.0, 255.0],
            [0.0, 0.0, 0.0, 255.0],
            [0.0, 0.0, 0.0, 255.0],
            [0.0, 0.0, 0.0, 255.0]
        ];
        // let border_radius = [10.0, 10.0, 10.0, 10.0];
        // let border_thickness = [10.0, 10.0, 10.0, 10.0];
        let border_radius = [0.0, 0.0, 0.0, 0.0];
        let border_thickness = [0.0, 0.0, 0.0, 0.0];

        self.vertices.append(&mut vec![
            RectangleVertex {
                position: [top_left.x, top_left.y, top_left.z],
                size: [rectangle.width, rectangle.height],
                background_color: [color[0], color[1], color[2], color[3]],
                border_color,
                border_radius,
                border_thickness,
            },

            RectangleVertex {
                position: [bottom_left.x, bottom_left.y, bottom_left.z],
                size: [rectangle.width, rectangle.height],
                background_color: [color[0], color[1], color[2], color[3]],
                border_color,
                border_radius,
                border_thickness,
            },

            RectangleVertex {
                position: [top_right.x, top_right.y, top_right.z],
                size: [rectangle.width, rectangle.height],
                background_color: [color[0], color[1], color[2], color[3]],
                border_color,
                border_radius,
                border_thickness,
            },

            RectangleVertex {
                position: [bottom_right.x, bottom_right.y, bottom_right.z],
                size: [rectangle.width, rectangle.height],
                background_color: [color[0], color[1], color[2], color[3]],
                border_color,
                border_radius,
                border_thickness,
            },
        ]);

        let next_starting_index: u32 = (self.indices.len() / 6) as u32 * 4;
        self.indices.append(&mut vec![
            next_starting_index,
            next_starting_index + 1,
            next_starting_index + 2,
            next_starting_index + 2,
            next_starting_index + 1,
            next_starting_index + 3,
        ]);
    }

    
    pub fn prepare(&self, context: &Context) -> PerFrameData {
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

    pub fn draw(
        &mut self,
        context: &Context,
        render_pass: &mut RenderPass,
        per_frame_data: PerFrameData
    ) {
        if self.vertices.is_empty() {
            return;
        }
        let rectangle_pipeline = self.cached_pipelines.get(&DEFAULT_RECTANGLE_PIPELINE_CONFIG).unwrap();
        render_pass.set_pipeline(&rectangle_pipeline.pipeline);
        render_pass.set_bind_group(0, Some(&context.global_buffer.bind_group), &[]);
        render_pass.set_vertex_buffer(0, per_frame_data.vertex_buffer.slice(..));
        render_pass.set_index_buffer(per_frame_data.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..(self.indices.len() as u32), 0, 0..1);
        self.vertices.clear();
        self.indices.clear();
    }
}

