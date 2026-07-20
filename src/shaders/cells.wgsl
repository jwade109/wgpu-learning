@group(1) @binding(0) var<uniform> model: mat4x4<f32>;

struct UniformData {
    mouse_pos: vec2<f32>,
    time: f32,
}

@group(2) @binding(0) var<uniform> uniform_data: UniformData;

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

    let pix = 6.0;

    let p = floor(in.position.xy / pix) * pix;

    for (var i = 0; i < 43; i += 1)
    {
        var x = rand(f32(i)) * 2500;
        var y = rand(f32(i + 747457)) * 1200;

        let period = (rand(f32(i) * 90275) * 5.0) + 2.0;

        var dx = cos(uniform_data.time / period) * 200.0;
        var dy = sin(uniform_data.time / period) * 200.0;

        if i == 0 {
            x = uniform_data.mouse_pos.x;
            y = uniform_data.mouse_pos.y;
            dx = 0.0;
            dy = 0.0;
        }

        let r = rand(f32(i + 97212041)) * 100 + 30;
        let d = length(p - vec2<f32>(x + dx, y + dy)) - r;
        dmin = smin(dmin, d, k);
    }

    let m = 1.0 - smoothstep(-5.0, 5.0, dmin);


    let r = 1.0 - smoothstep(-200.0, -20.0, dmin);
    let g = smoothstep(50.0, 60.0, dmin) * (1.0 - smoothstep(50.0, 60.0, dmin));
    let b = smoothstep(0.0, 20.0, dmin) * (1.0 - smoothstep(0.0, 20.0, dmin));

    return vec4<f32>(r, g, b, 1.0);
}
