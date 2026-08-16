// import("common.wgsl")

@group(0) @binding(0) var texture: texture_2d<f32>;
@group(0) @binding(1) var sample: sampler;
@group(1) @binding(0) var<uniform> params: ShaderParams;

struct ShaderParams {
    mouse_pos: vec2<f32>,
    resolution: vec2<f32>,
    time: f32,
}

struct Vertex {
    @location(0) position: vec3<f32>,
};

struct VertexShaderOutput {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_main(vertex: Vertex) -> VertexShaderOutput {
    var out: VertexShaderOutput;
    out.position = vec4<f32>(vertex.position, 1.0);
    return out;
}

fn sdf_circle(p: vec2<f32>, center: vec2<f32>, radius: f32) -> f32 {
    let d = length(p - center);
    return d - radius;
}

fn sdf_line(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / dot(ba,ba), 0.0, 1.0);
    return length(pa - ba*h);
}

fn sdf_capsule(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>, padding: f32) -> f32 {
    let r = sdf_line(p, a, b);
    return r - padding;
}

fn hill(p: vec2<f32>, peak: vec2<f32>, height: f32) -> f32 {
    let d = length(p - peak);
    let z = height / (1.0 + d / height);
    return z;
}

fn range(p: vec2<f32>, p1: vec2<f32>, p2: vec2<f32>, height: f32) -> f32 {
    let d = sdf_line(p, p1, p2);
    let z = height / (1.0 + d / height);
    return z;
}

fn sinusoid(p: vec2<f32>) -> f32 {
    return sin(p.x);
}

const p1 = vec2<f32>(700.0, 800.0);
const p2 = vec2<f32>(1500.0, 1200.0);
const p3 = vec2<f32>(1700.0, 600.0);

fn height_func(p: vec2<f32>) -> f32 {

    return 1.0 - length(textureSample(texture, sample, p / params.resolution).xyz);
}

fn is_in_shadow(pz: vec3<f32>, sun: vec3<f32>) -> bool {
    var sample = pz;
    let u = normalize(sun - sample);

    var i = 0;

    while (length(sample - sun) > 5.0)
    {
        i += 1;
        if (i > 200)
        {
            break;
        }

        sample += u * 1.0;
        let z_sample = height_func(sample.xy);
        if (z_sample > sample.z)
        {
            return true;
        }
    }

    return false;
}

@fragment
fn fs_main(in: VertexShaderOutput) -> @location(0) vec4<f32> {

    let sun = vec3<f32>(params.mouse_pos, 3.0 + sin(params.time));

    // let pix = 1.0;
    let p = in.position.xy; // floor(in.position.xy / pix) * pix;

    let color_here = textureSample(texture, sample, p / params.resolution);

    let z = height_func(p);

    let pz = vec3<f32>(p, z);

    let tide_level = 0.0; // 20.0 + 3.0 * sin(params.time / 3.0);

    let is_in_shadow = is_in_shadow(pz, sun);

    var color = 0.0;
    let tol = 0.1;

    for (var level = 5; level < 100; level += 5)
    {
        let l = f32(level);
        color += 0.05 * smoothstep(l - tol, l + tol, z);
    }

    var r = pow(color_here.r, 2.2);
    var g = pow(color_here.g, 2.2);
    var b = pow(color_here.b, 2.2);

    let point_of_interest = params.mouse_pos;

    let sdf_d = sdf_circle(p, point_of_interest, 4.0);

    if (is_in_shadow)
    {
        r *= 0.2;
        g *= 0.2;
        b *= 0.2;
    }

    if (sdf_d < 0.0)
    {
        r = 1.0;
        g = 0.0;
        b = 0.0;
    }

    return vec4<f32>(r, g, b, 1.0);
}
