// Vertex shader

struct GlobalUniform {
    view_proj: mat4x4<f32>,
    is_srgb_format: u32,
};

@group(1) @binding(0)
var<uniform> global: GlobalUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) texture_coordinates: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) texture_coordinates: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
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

fn to_srgb(color: vec4<f32>) -> vec4<f32> {
    var color_s1 = (color.xyz / 255 + 0.055) / 1.055;
    var srgb_color = vec3(pow(color_s1.x, 2.4), pow(color_s1.y, 2.4), pow(color_s1.z, 2.4));
    return vec4(srgb_color.xyz, color.w / 255);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    var color = in.color;
    if !bool(global.is_srgb_format) {
         // color = to_srgb(color);
         color = color / 255.0;
    } else {
        color = color / 255.0;
    }
    return textureSample(texture_view, texture_sampler, in.texture_coordinates) * color;
}
