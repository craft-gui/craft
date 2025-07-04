use craft_renderer::vello::VelloRenderer;
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct TransformUniform {
    matrix: [[f32; 4]; 4],
}

pub(crate) fn draw_gui_texture_and_canvas(renderer: &mut VelloRenderer, pos_x: f32, pos_y: f32, size_width: f32, size_height: f32, rotation_radians: f32) {
    if !renderer.render_into_texture {
        return;
    }

    let vertices = [
        Vertex { position: [0.0, 0.0, 0.0, 1.0] },
        Vertex { position: [1.0, 0.0, 0.0, 1.0] },
        Vertex { position: [0.0, 1.0, 0.0, 1.0] },
        Vertex { position: [1.0, 1.0, 0.0, 1.0] },
    ];

    let indices = [0u16, 1, 2, 2, 1, 3];

    let vertex_buffer = renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Triangle Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let vertex_buffer_layout = wgpu::VertexBufferLayout {
        array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x4,
            },
        ],
    };

    let index_buffer = renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Quad Index Buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    let ortho = Mat4::orthographic_rh_gl(0.0, renderer.render_surface.width() as f32, renderer.render_surface.height() as f32, 0.0, -1.0, 1.0);
    let model =
        Mat4::from_translation(Vec3::new(pos_x + size_width / 2.0, pos_y + size_height / 2.0, 0.0)) *
            Mat4::from_rotation_z(rotation_radians) *
            Mat4::from_translation(Vec3::new(-size_width / 2.0, -size_height / 2.0, 0.0)) *
            Mat4::from_scale(Vec3::new(size_width, size_height, 1.0));
    let mvp = ortho * model;

    let transform_data = TransformUniform {
        matrix: mvp.to_cols_array_2d(),
    };

    let transform_buffer = renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Transform Uniform Buffer"),
        contents: bytemuck::bytes_of(&transform_data),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let transform_bind_group_layout = renderer.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Transform Bind Group Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let transform_bind_group = renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &transform_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: transform_buffer.as_entire_binding(),
        }],
        label: Some("Transform Bind Group"),
    });
    
    let triangle_shader = renderer.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Triangle Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("triangle.wgsl").into()),
    });
    

    let triangle_pipeline_layout = renderer.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Triangle Pipeline Layout"),
        bind_group_layouts: &[&transform_bind_group_layout],
        push_constant_ranges: &[],
    });

    let triangle_pipeline = renderer.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Triangle Pipeline"),
        layout: Some(&triangle_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &triangle_shader,
            entry_point: Some("vs_main"),
            buffers: &[vertex_buffer_layout],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &triangle_shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(renderer.render_surface.surface_config.format.into())],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    let swapchain_surface_texture = renderer.render_surface.get_swapchain_surface_texture(&renderer.device, renderer.render_surface.width(), renderer.render_surface.height());
    let swapchain_surface_texture_view = swapchain_surface_texture
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Blit + Triangle Encoder"),
    });
    
    renderer.texture_blitter.copy(&renderer.device, &mut encoder, &renderer.render_surface.surface_view, &swapchain_surface_texture_view);
    {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Final Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &swapchain_surface_texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            ..Default::default()
        });
        
        rpass.set_pipeline(&triangle_pipeline);
        rpass.set_bind_group(0, &transform_bind_group, &[]);
        rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
        rpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        rpass.draw_indexed(0..6, 0, 0..1);
    }

    renderer.queue.submit(Some(encoder.finish()));
    swapchain_surface_texture.present();
}
