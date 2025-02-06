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
    @location(3) content_type: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) texture_coordinates: vec2<f32>,
    @location(2) content_type: u32,
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

    switch (in.content_type) {
        // Content Type: Mask
        case 0u: {
            var color = in.color;
            var sampled_color = textureSample(texture_view, texture_sampler, in.texture_coordinates);
            return vec4(color.rgb, color.a * sampled_color.a);
        }
        // Content Type: Color
        // This is for emojis.
        case 1u: {
            return textureSample(texture_view, texture_sampler, in.texture_coordinates);
        }
        default: {
            return vec4(1.0, 1.0, 1.0, 0.0);
        }
    }

}
