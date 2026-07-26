@group(0) @binding(0) var<uniform> uniform_data: ShaderParams;

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
};

struct VertexShaderOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(vertex: Vertex) -> VertexShaderOutput {
    var out: VertexShaderOutput;
    out.position = vec4<f32>(vertex.position, 1.0);
    out.color = vertex.color;
    return out;
}

fn sdf_ring_t(p: vec2<f32>, center: vec2<f32>, r_center: f32, thickness: f32) -> f32 {
    let d = length(p - center) - r_center;
    return abs(d) - thickness;
}

fn sdf_ring_ul(p: vec2<f32>, center: vec2<f32>, r_lower: f32, r_upper: f32) -> f32 {
    let r_center = (r_lower + r_upper) / 2.0;
    let thickness = r_upper - r_lower;
    return sdf_ring_t(p, center, r_center, thickness);
}

fn sdf_ellipse(p: vec2<f32>, center: vec2<f32>, r: vec2<f32>) -> f32 {
    let d = p - center;
    let k = length(d / r);
    return (k - 1.0) * min(r.x, r.y);
}

fn loop_anim(t: f32, dur: f32) -> f32 {
    return fract(t / dur) * dur;
}

fn smin( a: f32, b: f32, k: f32 ) -> f32
{
    let r = exp2(-a/k) + exp2(-b/k);
    return -k*log2(r);
}

@fragment
fn fs_main(in: VertexShaderOutput) -> @location(0) vec4<f32> {

    let t = loop_anim(uniform_data.time, 3.0) - 1.0;

    let v = 30.0;

    var d = 10000000000.0;
    let p = in.position.xy;

    {
        let center = vec2<f32>(800.0, 800.0);
        let r_upper = 100.0 + t * 10.0 * v;
        let r_lower = 20.0 + t * 12.0 * v;
        let d1 = sdf_ring_ul(p, center, r_lower, r_upper);
        d = smin(d, d1, 15.0);
    }

    {
        let center = vec2<f32>(1100.0, 900.0);
        let r = vec2<f32>(300.0, 170.0);
        let d2 = sdf_ellipse(p, center, r);
        d = smin(d, d2, 15.0);
    }

    let c = 1.0 - smoothstep(0.0, 5.0, d);

    return vec4<f32>(c, c, c, 1.0);
}
