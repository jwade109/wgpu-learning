@group(0) @binding(0) var inputTex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
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
    out.position = vec4<f32>(vertex.position, 1.0);
    out.tex_coord = vertex.tex_coord;
    out.tex_coord.y = 1.0 - out.tex_coord.y;
    return out;
}

@fragment
fn fs_main(in: VertexShaderOutput) -> @location(0) vec4<f32> {

    var color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    // // Example simple 5-tap horizontal offset weights
    const weights = array<f32, 3>(0.227027, 0.316216, 0.070270);
    // color += textureSample(inputTex, samp, in.tex_coord) * weights[0];

    var uv = (in.tex_coord * 1.0) / 1.0;
    let off = 0.002;
    let n = 20;
    let w = 1.0 / f32(n * 4);

    // uv.x = pow(uv.x, 2.0);
    // uv.y = pow(uv.y, 2.0);

    // color = textureSample(inputTex, samp, uv);

    for (var i = 0; i < n; i += 1) {
        let offset = vec2<f32>(off * f32(i), 0.0);
        color += textureSample(inputTex, samp, uv + offset) * w;
        color += textureSample(inputTex, samp, uv - offset) * w;
    }

    for (var i = 0; i < n; i += 1) {
        let offset = vec2<f32>(0.0, off * f32(i));
        color += textureSample(inputTex, samp, uv + offset) * w;
        color += textureSample(inputTex, samp, uv - offset) * w;
    }

    // // for (var i = 0; i < 3; i += 1) {
    // //     let offset = vec2<f32>(0.0, w * f32(i));
    // //     color += textureSample(inputTex, samp, in.tex_coord + offset) * 0.1;
    // //     color += textureSample(inputTex, samp, in.tex_coord - offset) * 0.1;
    // // }

    color.w = 1.0;

    return color;
}
