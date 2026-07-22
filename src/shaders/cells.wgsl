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

fn rgb_to_vec3(r: u32, g: u32, b: u32) -> vec3<f32>
{
    let rf = pow(f32(r) / 255.0, 2.2);
    let gf = pow(f32(g) / 255.0, 2.2);
    let bf = pow(f32(b) / 255.0, 2.2);
    return vec3<f32>(rf, gf, bf);
}

@fragment
fn fs_main(in: VertexShaderOutput) -> @location(0) vec4<f32> {

    let PURPLE =      rgb_to_vec3(87u,  16u,  110u); // rgb(87, 16, 110);
    let RED =         rgb_to_vec3(188u, 55u,  84u);  // rgb(188, 55, 84);
    let ORANGE =      rgb_to_vec3(249u, 142u, 9u);   // rgb(249, 142, 9);
    let YELLOW =      rgb_to_vec3(252u, 255u, 164u); // rgb(252, 255, 164);
    let OTHERYELLOW = rgb_to_vec3(252u, 253u, 228u); // rgb(255, 255, 228)
    let ANOTHERYELLOW = rgb_to_vec3(250u, 228u, 149u); // rgb(250, 228, 149)

    let k = 40.0;
    var dmin = 10000000000.0;

    let pix = 1.0;

    var p = floor(in.position.xy / pix) * pix;

    for (var i = 0; i < 32; i += 1)
    {
        var x = rand(f32(i)) * 2500;
        var y = rand(f32(i + 747457)) * 1200;

        let period = (rand(f32(i) * 90275) * 5.0) + 2.0;

        var dx = 0.0; // cos(uniform_data.time / period) * 200.0;
        var dy = sin(uniform_data.time / period) * 200.0;

        if i == 0 {
            x = uniform_data.mouse_pos.x;
            y = uniform_data.mouse_pos.y;
            dx = 0.0;
            dy = 0.0;
        }

        let r = rand(f32(i + 97212041)) * 90 + 90;
        let d = length(p - vec2<f32>(x + dx, y + dy)) - r;
        dmin = smin(dmin, d, k);
    }

    if dmin < -50.0
    {
        return vec4<f32>(PURPLE, 1.0);
    }
    else if dmin < -20.0
    {
        return vec4<f32>(RED, 1.0);
    }
    else if 0.0 < dmin && dmin < 20.0
    {
        let t = smoothstep(0.0, 20.0, dmin); // goes from 0 to 1

        let color_a = vec3<f32>(0.6, 0.2, 0.75);
        let color_b = vec3<f32>(0.2, 0.5, 0.3);

        let color = mix(color_a, color_b, t);

        return vec4<f32>(ORANGE, 1.0);
    }
    else if 50.0 < dmin && dmin < 60.0
    {
        return vec4<f32>(YELLOW, 1.0);
    }
    else if 70.0 < dmin
    {
        let h = fract(p.y / 200.0);

        if (h < 0.5)
        {
            return vec4<f32>(OTHERYELLOW, 1.0);
        }
        else
        {
            return vec4<f32>(ANOTHERYELLOW, 1.0);
        }
    }

    // let bg_color = vec3<f32>(0.75, 0.5, 0.25);
    let bg_color = vec3<f32>(0.0, 0.0, 0.0);

    return vec4<f32>(bg_color, 1.0);

    // let m = 1.0 - smoothstep(-5.0, 5.0, dmin);


    // let r = 1.0 - smoothstep(-70.0, -50.0, dmin);
    // let g = smoothstep(50.0, 60.0, dmin) * (1.0 - smoothstep(50.0, 60.0, dmin));
    // let b = smoothstep(0.0, 20.0, dmin) * (1.0 - smoothstep(0.0, 20.0, dmin));

    // let r_bg = 0.75 * (1.0 - r);
    // let g_bg = 0.5  * (1.0 - g);
    // let b_bg = 0.25 * (1.0 - b);

    // let blob_color = vec3<f32>(r, g, b);


    // let color = bg_color;

}
