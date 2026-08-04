@group(0) @binding(0) var<uniform> transform: mat4x4<f32>;
@group(1) @binding(0) var the_texture: texture_2d<f32>;
@group(1) @binding(1) var the_sampler: sampler;
@group(2) @binding(0) var<uniform> sample_data: SampleInfo;

struct SampleInfo {
    origin_x: u32,
    origin_y: u32,
    sample_width: u32,
    sample_height: u32,
    image_width: u32,
    image_height: u32,
};

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) tex_coord: vec2<f32>,
};

struct VertexShaderOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};

@vertex
fn vs_main(vertex: Vertex) -> VertexShaderOutput {
    var out: VertexShaderOutput;
    out.position = transform * vec4<f32>(vertex.position, 1.0);
    out.tex_coord = vertex.tex_coord;
    out.tex_coord.y = 1.0 - out.tex_coord.y;

    let start_x = f32(sample_data.origin_x) / f32(sample_data.image_width);
    let start_y = f32(sample_data.origin_y) / f32(sample_data.image_height);

    let dims_x = f32(sample_data.sample_width) / f32(sample_data.image_width);
    let dims_y = f32(sample_data.sample_height) / f32(sample_data.image_height);

    out.tex_coord.x = start_x + dims_x * out.tex_coord.x;
    out.tex_coord.y = start_y + dims_y * out.tex_coord.y;

    return out;
}

@fragment
fn fs_main(in: VertexShaderOutput) -> @location(0) vec4<f32> {
    var c = textureSample(the_texture, the_sampler, in.tex_coord);
    c.x = pow(c.x, 2.0);
    c.y = pow(c.y, 2.0);
    c.z = pow(c.z, 2.0);

    let l = length(c.xyz);

    // for debug: highlight the background
    // if l < 0.3 {
    //     return vec4<f32>(1.0, 0.4, 0.4, 0.4);
    // }

    return vec4<f32>(1.0, 1.0, 1.0, smoothstep(0.38, 0.43, l));
}
