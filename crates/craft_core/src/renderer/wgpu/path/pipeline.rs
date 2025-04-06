use crate::renderer::wgpu::context::Context;
use crate::renderer::wgpu::path::vertex::PathVertex;

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct PathPipelineConfig {
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

pub(crate) const DEFAULT_PATH_PIPELINE_CONFIG: PathPipelineConfig = PathPipelineConfig {
    blend_state: DEFAULT_BLEND_STATE,
};

pub struct PathPipeline {
    pub(crate) pipeline: wgpu::RenderPipeline,
}

impl PathPipeline {
    pub fn new_pipeline_with_configuration(context: &Context, config: PathPipelineConfig) -> Self {
        let shader = context.device.create_shader_module(wgpu::include_wgsl!("./path.wgsl"));
        let render_pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Path Render Pipeline Layout"),
            bind_group_layouts: &[&context.global_buffer.bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Path Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[PathVertex::description()],
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
            pipeline: render_pipeline,
        }
    }
}
