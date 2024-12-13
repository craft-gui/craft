use wgpu::util::DeviceExt;
use crate::renderer::wgpu::camera::Camera;
use crate::renderer::wgpu::context::Context;
use crate::renderer::wgpu::text::vertex::TextVertex;
use crate::renderer::wgpu::globals::GlobalUniform;

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct TextPipelineConfig {
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

pub(crate) const DEFAULT_TEXT_PIPELINE_CONFIG: TextPipelineConfig = TextPipelineConfig {
    blend_state: DEFAULT_BLEND_STATE
};

pub struct TextPipeline {
    pub(crate) pipeline: wgpu::RenderPipeline,
}

impl TextPipeline {
    pub fn new_pipeline_with_configuration(context: &Context, config: TextPipelineConfig) -> Self {

        let camera = Camera {
            width: context.surface_config.width as f32,
            height: context.surface_config.height as f32,
            z_near: 0.0,
            z_far: 100.0,
        };

        let mut global_uniform = GlobalUniform::new();
        global_uniform.set_view_proj_with_camera(&camera);
        
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

        let shader = context.device.create_shader_module(wgpu::include_wgsl!("./text.wgsl"));
        let render_pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Text Renderer Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &context.global_buffer.bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Text Renderer Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[TextVertex::description()],
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