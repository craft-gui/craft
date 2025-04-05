struct GlobalUniform {
    is_surface_srgb_format: u32,
    view_proj: mat4x4<f32>,
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

fn to_srgb(linear_color: vec3<f32>) -> vec3<f32> {
    let cutoff = 0.0031308;
    let linear_portion = linear_color * 12.92;
    let gamma_portion = 1.055 * pow(linear_color, vec3<f32>(1.0 / 2.4)) - 0.055;
    return select(gamma_portion, linear_portion, linear_color < vec3<f32>(cutoff));
}

fn to_linear_from_srgb(srgb: vec3<f32>) -> vec3<f32> {
    let cutoff = 0.04045;
    let linear_portion = srgb / 12.92;
    let gamma_portion = pow((srgb + 0.055) / 1.055, vec3<f32>(2.4));
    return select(gamma_portion, linear_portion, srgb < vec3<f32>(cutoff));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Vertex Input Color: sRGB.
    if (global.is_surface_srgb_format == 1) {
       // Convert to a linear color, the surface will convert this back to sRGB later.
       return vec4(to_linear_from_srgb(in.v_color.rgb), in.v_color.a);
    } else {
       // If the surface is linear, do nothing this is already a sRGB color.
       return in.v_color;
    }
}