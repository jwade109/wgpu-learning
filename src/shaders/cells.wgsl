@group(1) @binding(0) var<uniform> model: mat4x4<f32>;

struct Vertex {
    @location(0) position: vec3<f32>,
};

struct VertexShaderOutput {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_main(vertex: Vertex) -> VertexShaderOutput {
    var out: VertexShaderOutput;
    out.position = model * vec4<f32>(vertex.position, 1.0);
    return out;
}

fn smin( a: f32, b: f32, k: f32 ) -> f32
{
    let r = exp2(-a/k) + exp2(-b/k);
    return -k*log2(r);
}

fn rand(x: f32) -> f32 {
    let v = vec2<f32>(x, x);
    return fract(sin(dot(v, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

@fragment
fn fs_main(in: VertexShaderOutput) -> @location(0) vec4<f32> {

    let k = 5.0;
    var dmin = 10000000000.0;

    let pix = 10.0;

    let p = floor(in.position.xy / pix) * pix;

    for (var i = 0; i < 70; i += 1)
    {
        let x = rand(f32(i)) * 2500;
        let y = rand(f32(i + 747457)) * 1200;
        let r = rand(f32(i + 97212041)) * 100 + 30;
        let d = length(p - vec2<f32>(x, y)) - r;
        dmin = smin(dmin, d, k);
    }

    let m = 1.0 - smoothstep(-5.0, 5.0, dmin);


    let r = 1.0 - smoothstep(-200.0, -20.0, dmin);
    let g = smoothstep(50.0, 60.0, dmin) * (1.0 - smoothstep(50.0, 60.0, dmin));
    let b = smoothstep(0.0, 20.0, dmin) * (1.0 - smoothstep(0.0, 20.0, dmin));

    return vec4<f32>(r, g, b, 1.0);
}
