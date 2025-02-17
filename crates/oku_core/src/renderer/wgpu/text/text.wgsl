// Vertex shader

struct GlobalUniform {
    view_proj: mat4x4<f32>,
    is_srgb_format: u32,
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
            result = sampled_color;
        }
        case 2u: { // Rectangle (Cursor & Highlights)
            result = vec4(in.color.rgb, in.color.a);
        }
        default: {
            result = vec4(1.0, 1.0, 1.0, 0.0);
        }
    }

    return result;
}

