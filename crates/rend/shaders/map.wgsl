// import("common.wgsl")

@group(0) @binding(0) var<uniform> params: ShaderParams;

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

    let screen_center = params.resolution / 2.0;
    let l = length(p - screen_center);
    if l < 200.0 {
        return 40.0 * (1.0 - smoothstep(180.0, 200.0, l));
    }

    let n1 = perlinNoise2(p / 30.0);
    let n2 = perlinNoise2(p / 60.0);
    let n3 = perlinNoise2(p / 120.0);
    return n1 * 2.0 + n2 * 5.0 + n3 * 20.0; // (/*hill(p, params.mouse_pos, 60.0) + */
        //    hill(p, vec2<f32>(1400.0, 700.0), 45.0) +
        //    range(p, p1, p2, 53.0) +
        //    range(p, p2, p3, 40.0)) * (sinusoid(p / 100.0) * 0.2 + 0.8);
}

fn is_in_shadow(pz: vec3<f32>, sun: vec3<f32>) -> bool {
    var sample = pz;
    let u = normalize(sun - sample);

    var i = 0;

    while (length(sample - sun) > 5.0)
    {
        i += 1;
        if (i > 40)
        {
            break;
        }

        sample += u * 5.0;
        let z_sample = height_func(sample.xy);
        if (z_sample > sample.z)
        {
            return true;
        }
    }

    return false;
}

fn permute4(x: vec4f) -> vec4f { return ((x * 34. + 1.) * x) % vec4f(289.); }
fn fade2(t: vec2f) -> vec2f { return t * t * t * (t * (t * 6. - 15.) + 10.); }

fn perlinNoise2(P: vec2f) -> f32 {
    var Pi: vec4f = floor(P.xyxy) + vec4f(0., 0., 1., 1.);
    let Pf = fract(P.xyxy) - vec4f(0., 0., 1., 1.);
    Pi = Pi % vec4f(289.); // To avoid truncation effects in permutation
    let ix = Pi.xzxz;
    let iy = Pi.yyww;
    let fx = Pf.xzxz;
    let fy = Pf.yyww;
    let i = permute4(permute4(ix) + iy);
    var gx: vec4f = 2. * fract(i * 0.0243902439) - 1.; // 1/41 = 0.024...
    let gy = abs(gx) - 0.5;
    let tx = floor(gx + 0.5);
    gx = gx - tx;
    var g00: vec2f = vec2f(gx.x, gy.x);
    var g10: vec2f = vec2f(gx.y, gy.y);
    var g01: vec2f = vec2f(gx.z, gy.z);
    var g11: vec2f = vec2f(gx.w, gy.w);
    let norm = 1.79284291400159 - 0.85373472095314 *
        vec4f(dot(g00, g00), dot(g01, g01), dot(g10, g10), dot(g11, g11));
    g00 = g00 * norm.x;
    g01 = g01 * norm.y;
    g10 = g10 * norm.z;
    g11 = g11 * norm.w;
    let n00 = dot(g00, vec2f(fx.x, fy.x));
    let n10 = dot(g10, vec2f(fx.y, fy.y));
    let n01 = dot(g01, vec2f(fx.z, fy.z));
    let n11 = dot(g11, vec2f(fx.w, fy.w));
    let fade_xy = fade2(Pf.xy);
    let n_x = mix(vec2f(n00, n01), vec2f(n10, n11), vec2f(fade_xy.x));
    let n_xy = mix(n_x.x, n_x.y, fade_xy.y);
    return 2.3 * n_xy;
}

@fragment
fn fs_main(in: VertexShaderOutput) -> @location(0) vec4<f32> {

    let sun = vec3<f32>(params.mouse_pos, 50.0);

    // let pix = 1.0;
    let p = in.position.xy; // floor(in.position.xy / pix) * pix;

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

    var r = color;
    var g = color;
    var b = 1.0;

    if (z < tide_level)
    {
        // in the ocean!
        r = color;
        g = color;
        b = 1.0;
    }
    else
    {
        r = color * 0.5;
        g = 0.5 + 0.5 * color;
        b = color * 0.5;
    }

    // let point_of_interest = vec2<f32>(1300.0, 900);
    let point_of_interest = params.mouse_pos;

    let sdf_d = sdf_circle(p, point_of_interest, 4.0);
    // let sdf_line = sdf_capsule(p, p1, p2, 3.0);

    if (sdf_d < 0.0)
    {
        r = 1.0;
        g = 0.0;
        b = 0.0;
    }

    // if (sdf_line < 0.0)
    // {
    //     r = 1.0;
    //     g = 0.5;
    //     b = 0.3;
    // }

    if (z > 49.5 && z < 50.5)
    {
        r = 0.6;
        g = 0.3;
        b = 0.3;
    }

    if (z > 29.5 && z < 30.5)
    {
        r = 0.6;
        g = 0.3;
        b = 0.3;
    }

    if (is_in_shadow)
    {
        r *= 0.4;
        g *= 0.4;
        b *= 0.4;
    }

    // for boundary

    // for (var line_dist = 10; line_dist < 100; line_dist += 10)
    // {
    //     let line_boundary = step(f32(line_dist), sdf_line) * (1.0 - step(f32(line_dist + 2), sdf_line));
    //     if (line_boundary > 0)
    //     {
    //         r = 0.0;
    //         g = 1.0;
    //         b = 0.0;
    //     }
    // }

    return vec4<f32>(r, g, b, 1.0);
}
