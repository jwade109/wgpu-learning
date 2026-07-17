@group(0) @binding(0) var myTexture: texture_2d<f32>;
@group(0) @binding(1) var mySampler: sampler;
@group(1) @binding(0) var<uniform> model: mat4x4<f32>;

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexPayload {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) texCoord: vec2<f32>,
};

@vertex
fn vs_main(vertex: Vertex) -> VertexPayload {
    var out: VertexPayload;
    out.position = model * vec4<f32>(vertex.position, 1.0);
    out.color = vertex.color;
    out.texCoord = vec2<f32>(0.5 * (vertex.position.x + 1f), -0.5 * (vertex.position.y + 1f));
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
fn fs_main(in: VertexPayload) -> @location(0) vec4<f32> {

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

    // let color = vec4<f32>(in.color, 1.0) * textureSample(myTexture, mySampler, in.texCoord);
    // return vec4<f32>(r, g, b, 1.0);
    return vec4<f32>(r, g, b, 1.0);
    // return vec4<f32>(in.color, 1.0);
}