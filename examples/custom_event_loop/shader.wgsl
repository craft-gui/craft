@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    var pos = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0), // bottom left
        vec2<f32>(1.0, -1.0),  // bottom right
        vec2<f32>(-1.0, 1.0),  // top left
        vec2<f32>(-1.0, 1.0),  // top left
        vec2<f32>(1.0, -1.0),  // bottom right
        vec2<f32>(1.0, 1.0)    // top right
    );
    return vec4<f32>(pos[vertex_index], 0.0, 1.0);
}

@group(0) @binding(0)
var my_texture: texture_2d<f32>;

@group(0) @binding(1)
var my_sampler: sampler;

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = pos.xy / vec2<f32>(textureDimensions(my_texture, 0));
    return textureSample(my_texture, my_sampler, uv);
}
