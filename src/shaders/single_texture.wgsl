@group(0) @binding(0) var<uniform> transform: mat4x4<f32>;
@group(1) @binding(0) var the_texture: texture_2d<f32>;
@group(1) @binding(1) var the_sampler: sampler;

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
    return out;
}

@fragment
fn fs_main(in: VertexShaderOutput) -> @location(0) vec4<f32> {
    var c = textureSample(the_texture, the_sampler, in.tex_coord);
    c.x = pow(c.x, 2.0);
    c.y = pow(c.y, 2.0);
    c.z = pow(c.z, 2.0);
    return c;
}
