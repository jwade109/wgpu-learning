@group(2) @binding(0) var the_texture: texture_2d<f32>;
@group(2) @binding(1) var the_sampler: sampler;

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) tex_coord: vec2<f32>,
};

struct VertexShaderOut {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};

struct FragmentShaderOut {
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(vertex: Vertex) -> VertexShaderOut {
    var out: VertexShaderOut;
    out.position = vec4<f32>(vertex.position, 1.0);
    out.tex_coord = vec2<f32>(vertex.tex_coord.x, 1.0 - vertex.tex_coord.y);
    return out;
}

@fragment
fn fs_main(in: VertexShaderOut) -> FragmentShaderOut {
    var out: FragmentShaderOut;
    out.color = textureSample(the_texture, the_sampler, in.tex_coord);
    return out;
}
