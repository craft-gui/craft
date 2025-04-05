// Vertex shader

struct GlobalUniform {
    is_surface_srgb_format: u32,
    view_proj: mat4x4<f32>,
};

@group(1) @binding(0)
var<uniform> global: GlobalUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) texture_coordinates: vec2<f32>,
    @location(2) color: vec4<f32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) texture_coordinates: vec2<f32>
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.texture_coordinates = model.texture_coordinates;
    out.clip_position = global.view_proj * vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader

@group(0) @binding(0)
var texture_view: texture_2d<f32>;
@group(0) @binding(1)
var texture_sampler: sampler;

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
    // NOTE: This returns a linear color.
    let color = textureSample(texture_view, texture_sampler, in.texture_coordinates);

    if (global.is_surface_srgb_format == 1) {
        // Do nothing, the surface will convert our linear color to sRGB later.
        return color;
    } else {
        // Manually convert the linear color to sRGB because the window surface
        // is linear and will not apply any transformations.
        return vec4(to_srgb(color.rgb), color.a);
    }
}
