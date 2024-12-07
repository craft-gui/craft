struct GlobalUniform {
    view_proj: mat4x4<f32>,
    is_srgb_format: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) v_xyz: vec3<f32>,
    @location(1) v_size: vec2<f32>,
    @location(2) v_background_color: vec4<f32>,
    @location(3) v_border_top_color: vec4<f32>,
    @location(4) v_border_right_color: vec4<f32>,
    @location(5) v_border_bottom_color: vec4<f32>,
    @location(6) v_border_left_color: vec4<f32>,
    @location(7) v_border_radius: vec4<f32>,
    @location(8) v_center: vec2<f32>,
    @location(9) v_border_thickness: vec4<f32>
};

struct VertexInput {
   @location(0) a_xyz: vec3<f32>,
   @location(1) a_size: vec2<f32>,
   @location(2) a_background_color: vec4<f32>,
   @location(3) a_border_top_color: vec4<f32>,
   @location(4) a_border_right_color: vec4<f32>,
   @location(5) a_border_bottom_color: vec4<f32>,
   @location(6) a_border_left_color: vec4<f32>,
   @location(7) a_border_radius: vec4<f32>,
   @location(8) a_border_thickness: vec4<f32>,
   @builtin(vertex_index) vertex_index: u32,
};


@group(0) @binding(0)
var<uniform> global: GlobalUniform;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {

    var out: VertexOutput;

    out.v_xyz = in.a_xyz;
    out.v_size = in.a_size;
    out.v_background_color = in.a_background_color;
    out.v_border_top_color = in.a_border_top_color;
    out.v_border_right_color = in.a_border_right_color;
    out.v_border_bottom_color = in.a_border_bottom_color;
    out.v_border_left_color = in.a_border_left_color;
    out.v_border_radius = in.a_border_radius;
    out.v_border_thickness = in.a_border_thickness;

    let current_vertex: i32 = i32(in.vertex_index) % 4;
    switch (current_vertex) {
        case 0: {
            out.v_center = vec2<f32>(in.a_xyz.x + in.a_size.x / 2.0, in.a_xyz.y + in.a_size.y / 2.0);
        }
        case 1: {
            out.v_center = vec2<f32>(in.a_xyz.x + in.a_size.x / 2.0, in.a_xyz.y - in.a_size.y / 2.0);
        }
        case 2: {
            out.v_center = vec2<f32>(in.a_xyz.x - in.a_size.x / 2.0, in.a_xyz.y + in.a_size.y / 2.0);
        }
        case 3: {
            out.v_center = vec2<f32>(in.a_xyz.x - in.a_size.x / 2.0, in.a_xyz.y - in.a_size.y / 2.0);
        }
        default: {
            out.v_center = vec2<f32>(0.0, 0.0);
        }
    }

    out.clip_position = global.view_proj * vec4<f32>(in.a_xyz, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let center = in.v_center;
    let halved_size = in.v_size / 2.0;
    let pos_relative_to_origin = in.clip_position.xy - center;

    let sdf = sdf_rounded_rectangle(pos_relative_to_origin, halved_size, in.v_border_radius);
    let sdf_debug_color = sdf / length(halved_size);

    let distance_from_border = abs(pos_relative_to_origin) - halved_size;
    let is_vertical_border = abs(distance_from_border.y) <= abs(distance_from_border.x);
    let is_horizontal_border = abs(distance_from_border.x) <= abs(distance_from_border.y);

    var is_top_border = false;
    var is_bottom_border = false;
    var is_left_border = false;
    var is_right_border = false;

    if is_vertical_border {
        // Top border
        if (pos_relative_to_origin.y < 0.0 && sdf <= 0.0 && sdf >= -in.v_border_thickness.x) {
            is_top_border = true;
        }
        // Bottom border
        if (pos_relative_to_origin.y > 0.0 && sdf <= 0.0 && sdf >= -in.v_border_thickness.y) {
            is_bottom_border = true;
        }
    } else if is_horizontal_border {
        // Left border
        if (pos_relative_to_origin.x < 0.0 && sdf <= 0.0 && sdf >= -in.v_border_thickness.z) {
            is_left_border = true;
        }
        // Right border
        if (pos_relative_to_origin.x > 0.0 && sdf <= 0.0 && sdf >= -in.v_border_thickness.w) {
            is_right_border = true;
        }
    }

    // On the border.
    if is_top_border {
        return in.v_border_top_color / 255.0;
    } else if is_right_border {
        return in.v_border_right_color / 255.0;
    } else if is_bottom_border {
        return in.v_border_bottom_color / 255.0;
    } else if is_left_border {
        return in.v_border_left_color / 255.0;
    } else if sdf < 0.0 { // Inside the shape.
        return in.v_background_color / 255.0;
    } else { // Outside the shape.
        discard;
        // return vec4(1.0, 0.0, 1.0, 1.0);
    }
}

fn sdf_rounded_rectangle(point_test: vec2<f32>, b: vec2<f32>, border_radius: vec4<f32>) -> f32 {
  let conditional_radius = vec2<f32>(
    select(border_radius.x, border_radius.z, point_test.x > 0.0),
    select(border_radius.y, border_radius.w, point_test.y > 0.0)
  );

  let q = abs(point_test) - b + conditional_radius.x;
  return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - conditional_radius.x;
}