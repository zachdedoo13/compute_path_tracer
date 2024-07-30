
struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) uv: vec2<f32>, // Add this line
};

@vertex
fn vs_main(model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = vec4<f32>((model.position), 1.0);
    out.uv = model.position.xy;
    return out;
}



fn LessThan(f: vec3<f32>, value: f32) -> vec3<f32> {
    var back: vec3<f32>;
    if (f.x < value) { back.x = 1.0; } else { back.x = 0.0; }
    if (f.y < value) { back.y = 1.0; } else { back.y = 0.0; }
    if (f.z < value) { back.z = 1.0; } else { back.z = 0.0; }
    return back;
}

fn LinearToSRGB(rgb: vec3<f32>) -> vec3<f32> {
    let clamped_rgb = clamp(rgb, vec3(0.0), vec3(1.0));
    return mix(
        pow(clamped_rgb, vec3<f32>(1.0 / 2.4)) * 1.055 - 0.055,
        clamped_rgb * 12.92,
        LessThan(clamped_rgb, 0.0031308)
    );
}

fn SRGBToLinear(rgb: vec3<f32>) -> vec3<f32> {
    let clamped_rgb = clamp(rgb, vec3(0.0), vec3(1.0));
    return mix(
        pow((clamped_rgb + 0.055) / 1.055, vec3<f32>(2.4)),
        clamped_rgb / 12.92,
        LessThan(clamped_rgb, 0.04045)
    );
}

fn ACESFilm(x: vec3<f32>) -> vec3<f32> {
    let a: f32 = 2.51;
    let b: f32 = 0.03;
    let c: f32 = 2.43;
    let d: f32 = 0.59;
    let e: f32 = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e), vec3(0.0), vec3(1.0));
}





fn color_corection(in_color: vec3<f32>) -> vec3<f32> {
    var color = in_color;

    color *= 1.0; // exposure

    color = ACESFilm(color);

    color = LinearToSRGB(color);

    return color;
}




@group(0) @binding(0)
var read_texture: texture_storage_2d<rgba32float, read_write>;

@fragment
fn fs_main(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    var uv = in.uv * 0.5 + 0.5;

    let dimensions = textureDimensions(read_texture);

    let uv_nearest = vec2<i32>(floor(uv * vec2<f32>(dimensions.xy)));

    var color = textureLoad(read_texture, uv_nearest).rgb;

    color = color_corection(color);

    return vec4(color,  1.0);
}