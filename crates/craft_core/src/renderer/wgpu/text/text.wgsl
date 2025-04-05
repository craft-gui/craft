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
    @location(2) color: vec4<f32>,
    @location(3) @interpolate(flat) content_type: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) texture_coordinates: vec2<f32>,
    @location(2) @interpolate(flat) content_type: u32,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.texture_coordinates = model.texture_coordinates;
    out.clip_position = global.view_proj * vec4<f32>(model.position, 1.0);
    out.content_type = model.content_type;
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
    // Always sample the texture unconditionally
    let sampled_color = textureSample(texture_view, texture_sampler, in.texture_coordinates);

    var result: vec4<f32>;

    switch (in.content_type) {
        case 0u: { // Mask
            result = vec4(in.color.rgb, in.color.a * sampled_color.a);
        }
        case 1u: { // Emoji (Color)
             // Convert our sampled linear color to sRGB.
             // We will convert it here to be consistent, because our vertex color is in sRGB.
             result = vec4(to_srgb(sampled_color.rgb), sampled_color.a);
        }
        case 2u: { // Rectangle (Cursor & Highlights)
            result = vec4(in.color.rgb, in.color.a);
        }
        default: {
            result = vec4(1.0, 1.0, 1.0, 0.0);
        }
    }

    // Vertex Input Color: sRGB.
    if (global.is_surface_srgb_format == 1) {
        // Convert to a linear color, the surface will convert this back to sRGB later.
        return vec4(to_linear_from_srgb(result.rgb), result.a);
    } else {
        // If the surface is linear, do nothing this is already a sRGB color.
        return result.rgba;
    }
}

