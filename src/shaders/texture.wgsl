@group(0) @binding(0) var<uniform> transform: mat4x4<f32>;
@group(1) @binding(0) var the_texture: texture_2d<f32>;
@group(1) @binding(1) var the_sampler: sampler;
@group(2) @binding(0) var<uniform> params: ShaderParams;
@group(3) @binding(0) var<uniform> camera_projection: mat4x4<f32>;

struct ShaderParams {
    mouse_pos: vec2<f32>,
    resolution: vec2<f32>,
    camera_offset_x: f32,
    camera_offset_y: f32,
    camera_offset_z: f32,
    time: f32,
}

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) tex_coord: vec2<f32>,
};

struct VertexShaderOut {
    @builtin(position) position: vec4<f32>,
    @location(1) color: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};

struct FragmentShaderOut {
    @location(0) color: vec4<f32>,
}

// fn get_projection_matrix() -> mat4x4<f32> {
//     let tx = params.camera_offset_x;
//     let ty = params.camera_offset_y;
//     let tz = params.camera_offset_z;

//     let x = vec4<f32>(1.0, 0.0, 0.0, 0.0);
//     let y = vec4<f32>(0.0, 1.0, 0.0, 0.0);
//     let z = vec4<f32>(0.0, 0.0, 1.0, 0.0);
//     let t = vec4<f32>(-tx, -ty, 0.0, 1.0);

//     return mat4x4<f32>(x, y, z, t);
// }

@vertex
fn vs_main(vertex: Vertex) -> VertexShaderOut {
    var out: VertexShaderOut;
    out.position = camera_projection * transform * vec4<f32>(vertex.position, 1.0);
    out.tex_coord = vec2<f32>(vertex.tex_coord.x, 1.0 - vertex.tex_coord.y);
    out.color = vec4<f32>(vertex.color, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexShaderOut) -> FragmentShaderOut {
    var out: FragmentShaderOut;
    // var c = textureSample(the_texture, the_sampler, in.tex_coord);
    // c.x = pow(c.x, 2.0);
    // c.y = pow(c.y, 2.0);
    // c.z = pow(c.z, 2.0);
    out.color = in.color;
    return out;
}
