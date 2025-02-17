struct GlobalUniform {
    view_proj: mat4x4<f32>,
    is_srgb_format: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) v_color: vec4<f32>,
};

struct VertexInput {
    @location(0) v_xyz: vec3<f32>,
    @location(1) v_color: vec4<f32>,
};


@group(0) @binding(0)
var<uniform> global: GlobalUniform;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.v_color = in.v_color;
    out.clip_position = global.view_proj * vec4<f32>(in.v_xyz, 1.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.v_color;
}