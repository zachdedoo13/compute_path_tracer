#version 450

layout (local_size_x = 16, local_size_y = 16, local_size_z = 1) in;
layout(set = 0, binding = 0, rgba32f) uniform image2D the_texture;

layout(set = 1, binding = 0) uniform Constants {
    float time;
    int cframe;
    float aspect;
    int last_clear;
} c;

layout(set = 2, binding = 0) uniform Settings {
    int debug;
    int bounces;
} s;


#define MHD 0.001
#define FP 100.0
#define OFFSET 0.03


const float PI = 3.14159265359;
const float PI2 = 2.0f * PI;


//!code start flag

struct Ray {vec3 ro; vec3 rd; }
;
struct Mat {
    vec3 col;
    vec3 light;

    float spec;
    vec3 spec_col;
    float roughness;

//    float ior;
//    float refrac;
};
struct Hit {float d; Mat mat; };


#define MDEF Mat(vec3(0.0), vec3(0.0), 0.0, vec3(0.0), 0.0)


#include "shapes.glsl"
#include "map.glsl"
#include "funcs.glsl"
#include "rng.glsl"




Hit CastRay(Ray ray) {
    float t = 0.0;
    Mat mat;
    for (int i = 0; i < 80; i++) {
        vec3 p = ray.ro + ray.rd * t;
        Hit hit = map(p);
        mat = hit.mat;
        t += hit.d;

        if (abs(hit.d) < MHD) break;
        if (t > FP) break;
    }
    return Hit(t, mat);
}


vec3 path_trace(Ray start_ray, uint rng) {
    // init
    vec3 ret = vec3(0.0);
    vec3 throughput = vec3(1.0);
    Ray ray = start_ray;

    // path traceing loop 
    for (int i = 0; i <= s.bounces; i++) {
        Hit hit = CastRay(ray);

        // out of bounds
        if (hit.d > FP) {
            ret += vec3(0.0);
            break;
        }

        vec3 hit_pos = calc_point(ray, hit.d);
        vec3 hit_normal = calc_normal(hit_pos);
        ray.ro = hit_pos + hit_normal * OFFSET;

        ray.rd = normalize(hit_normal + RandomUnitVector(rng));

        ret += hit.mat.light * throughput;

        throughput *= hit.mat.col;
    }

    return ret;
}

vec3 normals(Ray ray) {

    Hit test = CastRay(ray);

    if (test.d > FP) { return vec3(0.0); }

    return normalize(calc_normal(calc_point(ray, test.d))) * 0.5 + 0.5;
}

vec3 colors(Ray ray) {

    Hit test = CastRay(ray);

    if (test.d > FP) { return vec3(0.0); }

    return test.mat.col;
}

vec3 calc_color(Ray ray, uint rng) {
    if (s.debug == 0) {
        return path_trace(ray, rng);
    }

    else if (s.debug == 1) {
        return normals(ray);
    }

    else if (s.debug == 2) {
        return colors(ray);
    }

    else {
        return vec3(0.0);
    }
}


void main() {
    ivec2 gl_uv = ivec2(gl_GlobalInvocationID.xy);
    ivec2 dimentions = imageSize(the_texture);
    if (bounds_check(gl_uv, dimentions)) { return; }
    vec2 uv = calc_uv(gl_uv, dimentions);

    uint rng = gen_rng(gl_uv, c.cframe, dimentions);


    Ray ray = Ray(
        vec3(0.0, 0.0, -3.0), // origin
        normalize(vec3(uv, 1.0)) // direction
    );

    vec3 col = calc_color(ray, rng);

    if (s.debug != 0) { imageStore(the_texture, gl_uv, vec4(col, 1.0)); return; }

    vec3 last_col = imageLoad(the_texture, gl_uv).rgb;

    col = mix(last_col, col, 1.0 / float(c.last_clear + 1));

    imageStore(the_texture, gl_uv, vec4(col, 1.0));
}