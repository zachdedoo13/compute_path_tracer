
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




@group(0) @binding(0)
var read_texture: texture_storage_2d<rgba32float, read_write>;

@fragment
fn fs_main(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    var uv = in.uv * 0.5 + 0.5;

    let dimensions = textureDimensions(read_texture);

    let uv_nearest = vec2<i32>(floor(uv * vec2<f32>(dimensions.xy)));

    let color = textureLoad(read_texture, uv_nearest).rgb;

    return vec4(color,  1.0);
}