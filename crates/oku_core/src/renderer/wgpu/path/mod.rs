use crate::geometry::Rectangle;
use crate::renderer::color::Color;
use crate::renderer::wgpu::context::Context;
use crate::renderer::wgpu::path::pipeline::{PathPipeline, PathPipelineConfig, DEFAULT_PATH_PIPELINE_CONFIG};
use crate::renderer::wgpu::path::vertex::PathVertex;
use crate::renderer::wgpu::PerFrameData;
use lyon::path::Path;
use std::collections::HashMap;
use wgpu::util::DeviceExt;
use wgpu::RenderPass;

pub(crate) mod pipeline;
mod vertex;

pub struct PathRenderer {
    pub(crate) cached_pipelines: HashMap<PathPipelineConfig, PathPipeline>,
    pub(crate) vertices: Vec<PathVertex>,
    pub(crate) indices: Vec<u32>,
}

impl PathRenderer {
    pub fn new(context: &Context) -> Self {
        let mut renderer = Self {
            cached_pipelines: HashMap::new(),
            vertices: vec![],
            indices: vec![],
        };
        
        renderer.cached_pipelines.insert(
            DEFAULT_PATH_PIPELINE_CONFIG,
            PathPipeline::new_pipeline_with_configuration(context, DEFAULT_PATH_PIPELINE_CONFIG)
        );
        
        renderer
    }
    
    pub fn build_rectangle(&mut self, rectangle: Rectangle, color: Color) {
        let x = rectangle.x;
        let y = rectangle.y;
        let width = rectangle.width;
        let height = rectangle.height;

        let top_left = glam::vec4(x, y, 0.0, 1.0);
        let bottom_left = glam::vec4(x, y + height, 0.0, 1.0);
        let top_right = glam::vec4(x + width, y, 0.0, 1.0);
        let bottom_right = glam::vec4(x + width, y + height, 0.0, 1.0);
        
        let color = color.components;
        let next_starting_index: u32 = self.vertices.len() as u32;
        
        self.vertices.extend(vec![
            PathVertex {
                position: [top_left.x, top_left.y, top_left.z],
                color,
            },
            PathVertex {
                position: [bottom_left.x, bottom_left.y, bottom_left.z],
                color,
            },
            PathVertex {
                position: [top_right.x, top_right.y, top_right.z],
                color,
            },
            PathVertex {
                position: [bottom_right.x, bottom_right.y, bottom_right.z],
                color,
            },
        ]);
        
        self.indices.extend(vec![
            next_starting_index,
            next_starting_index + 1,
            next_starting_index + 2,
            next_starting_index + 2,
            next_starting_index + 1,
            next_starting_index + 3,
        ]);
    }
    
    pub fn build(&mut self, path: Path, fill_color: Color) {
        let mut geometry: lyon::tessellation::VertexBuffers<PathVertex, u32> = lyon::tessellation::VertexBuffers::new();
        let mut tessellator = lyon::tessellation::FillTessellator::new();
        {
            tessellator.tessellate_path(
                &path,
                &lyon::tessellation::FillOptions::default(),
                &mut lyon::tessellation::BuffersBuilder::new(&mut geometry, |vertex: lyon::tessellation::FillVertex| {
                    let position = vertex.position();
                    let color = fill_color.components;
                    PathVertex {
                        position: [position.x, position.y, 0.0],
                        color,
                    }
                }),
            ).unwrap();
        }

        let vertex_offset = self.vertices.len() as u32;
        self.vertices.extend(geometry.vertices);
        self.indices.extend(geometry.indices.iter().map(|&i| i + vertex_offset));
    }

    
    pub fn prepare(&mut self, context: &Context) -> Option<PerFrameData> {
        let indices = self.indices.len();
        
        if indices == 0 {
            return None;
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

        self.vertices.clear();
        self.indices.clear();

        Some(PerFrameData {
            vertex_buffer,
            index_buffer,
            indices
        })
    }

    pub fn draw(
        &mut self,
        context: &Context,
        render_pass: &mut RenderPass,
        per_frame_data: &PerFrameData
    ) {
        let rectangle_pipeline = self.cached_pipelines.get(&DEFAULT_PATH_PIPELINE_CONFIG).unwrap();
        render_pass.set_pipeline(&rectangle_pipeline.pipeline);
        render_pass.set_bind_group(0, Some(&context.global_buffer.bind_group), &[]);
        render_pass.set_vertex_buffer(0, per_frame_data.vertex_buffer.slice(..));
        render_pass.set_index_buffer(per_frame_data.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..(per_frame_data.indices as u32), 0, 0..1);
    }
}

