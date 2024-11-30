use wgpu::util::DeviceExt;
use crate::engine::renderer::wgpu::camera::Camera;
use crate::engine::renderer::wgpu::context::Context;
use crate::engine::renderer::wgpu::rectangle::vertex::Vertex;
use crate::engine::renderer::wgpu::uniform::GlobalUniform;

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct RectanglePipelineConfig {
    pub(crate) blend_state: wgpu::BlendState,
}

pub(crate) const DEFAULT_BLEND_STATE: wgpu::BlendState = wgpu::BlendState {
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
};

pub(crate) const DEFAULT_PIPELINE_CONFIG: RectanglePipelineConfig = RectanglePipelineConfig {
    blend_state: DEFAULT_BLEND_STATE
};

pub struct RectanglePipeline {
    pub(crate) global_uniform: GlobalUniform,
    pub(crate) global_buffer: wgpu::Buffer,
    pub(crate) global_bind_group: wgpu::BindGroup,
    pub(crate) pipeline: wgpu::RenderPipeline,
}

impl RectanglePipeline {
    pub fn new_pipeline_with_configuration(context: &Context, config: RectanglePipelineConfig) -> Self {
        
        let camera = Camera {
            width: context.surface_config.width as f32,
            height: context.surface_config.height as f32,
            z_near: 0.0,
            z_far: 100.0,
        };

        let mut global_uniform = GlobalUniform::new();
        global_uniform.set_view_proj_with_camera(&camera);

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

        let shader = context.device.create_shader_module(wgpu::include_wgsl!("./rectangle.wgsl"));
        let render_pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Rectangle Render Pipeline Layout"),
            bind_group_layouts: &[&global_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Rectangle Render Pipeline"),
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
                    blend: Some(config.blend_state),
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

        Self {
            global_uniform,
            global_buffer,
            global_bind_group,
            pipeline: render_pipeline,
        }
        
    }
}