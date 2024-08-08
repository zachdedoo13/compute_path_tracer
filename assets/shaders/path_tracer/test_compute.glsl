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
    float scale;
    float fov;
    int aabb;
} s;

layout(set = 3, binding = 0) buffer DataBuffer {
    float data[];
};


#define STEPS 80

#define MHD 0.001
#define FP 100.0
#define OFFSET 0.03

#define AMBENT 0.2


#define BIGNUM 100000.0

const float PI = 3.14159265359;
const float PI2 = 2.0f * PI;


//!code start flag

struct Ray {vec3 ro; vec3 rd; };

struct Mat {
    vec3 col;

    float brightness;
    vec3 light;

    float spec;
    vec3 spec_col;
    float roughness;

    float IOR;
    float reftact_chance;
    float refract_roughness;
    vec3 refreact_col;
};
struct Hit {float d; Mat mat; };


#define MDEF Mat(vec3(0.0), 0.0, vec3(0.0), 0.0, vec3(0.0), 0.0, 0.0, 0.0, 0.0, vec3(0.0))


#include "shapes.glsl"
#include "map"
#include "funcs.glsl"
#include "rng.glsl"




Hit CastRay(Ray ray) {
    float t = 0.0;
    Mat mat;
    for (int i = 0; i < STEPS; i++) {
        vec3 p = ray.ro + ray.rd * t;
        Hit hit = map(p);
        mat = hit.mat;
        t += hit.d;

        if (abs(hit.d) < MHD) break;

        if (t > FP) return Hit(t, MDEF);
    }

    return Hit(t, mat);
}

vec3 path_trace(Ray start_ray, uint rng) {
    // init
    vec3 ret = vec3(0.0);
    vec3 throughput = vec3(1.0);
    Ray ray = start_ray;

    // path traceing loop
    int i;
    for (i = 0; i <= s.bounces; i++) {
//        OTA hits;
//        Hit hit;
//        int bc = 0;
//        hits = bounds_map(ray, bc);
//
//        int steps;
//        hit = TestCastRay(ray, steps, hits);

        Hit hit = CastRay(ray);


        // out of bounds
        if (hit.d > FP) {
            break;
        }

        // update the ray position
        vec3 hit_pos = calc_point(ray, hit.d);


//       vec3 hit_normal = test_calc_normal(hit_pos, hits);

        vec3 hit_normal = calc_normal(hit_pos);

        ray.ro = hit_pos + hit_normal * OFFSET;

        // lighting
        {
            float spec_chance = hit.mat.spec;

            if (spec_chance > 0.0) { }; // frenel

            bool do_spec = (RandomFloat01(rng) < spec_chance);

            // get the probability for choosing the ray type we chose
            float ray_prob = do_spec? spec_chance : 1.0 - spec_chance;
            ray_prob = max(ray_prob, 0.0001);

            {
                // Calculate a new ray direction.
                // Diffuse uses a normal oriented cosine weighted hemisphere sample.
                vec3 diffuse_ray_dir = normalize(hit_normal + RandomUnitVector(rng));

                if (do_spec) {
                    vec3 spec_ray_dir = reflect(ray.rd, hit_normal);
                    spec_ray_dir = normalize(mix(spec_ray_dir, diffuse_ray_dir, hit.mat.roughness * hit.mat.roughness));
                    ray.rd = spec_ray_dir;
                } else {
                    ray.rd = diffuse_ray_dir;
                }
            }

            ret += (normalize(hit.mat.light) * hit.mat.brightness) * throughput;
            throughput *= mix(hit.mat.col, hit.mat.spec_col, float(do_spec));

            throughput /= ray_prob;
        }

        // Russian Roulette
        {
            float p = max(throughput.r, max(throughput.g, throughput.b));
            if (RandomFloat01(rng) > p) break;

            // Add the energy we 'lose' by randomly terminating paths
            throughput *= 1.0f / p;
        }

    }

    if (s.debug == 3) { return vec3(float(i) / float(s.bounces)); }

    return ret;
}



vec3 normals(Ray ray) {

    Hit test = CastRay(ray);

    if (test.d > FP) { return vec3(0.0); }

    return normalize(calc_normal(calc_point(ray, test.d))) * 0.5 + 0.5;
}



vec3 colors(Ray ray) {

//    Hit test = CastRay(ray);

//    if (test.d > FP) { return vec3(0.0); }

//    return test.mat.col;

    return CastRay(ray).mat.col;
    //return test_cast(ray);
}



vec3 calc_color(Ray ray, uint rng) {
    if (s.debug == 0 || s.debug == 3) {
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
//    return;
    ivec2 gl_uv = ivec2(gl_GlobalInvocationID.xy);
    ivec2 dimentions = imageSize(the_texture);
    if (bounds_check(gl_uv, dimentions)) { return; }

    uint rng = gen_rng(gl_uv, c.cframe, dimentions);

    // calculate subpixel camera jitter for anti aliasing
    vec2 jitter = vec2(RandomFloat01(rng), RandomFloat01(rng)) - 0.5;

    vec2 uv = calc_uv(vec2(gl_uv) + jitter, dimentions);


    Ray ray = Ray(
        vec3(0.0, 0.0, -3.0), // origin
        normalize(vec3(uv, s.fov)) // direction
    );

    // pathtraceing
    vec3 col = calc_color(ray, rng);

    if (s.debug != 0) { imageStore(the_texture, gl_uv, vec4(col, 1.0)); return; } // instant return if not 0
//
    vec3 last_col = imageLoad(the_texture, gl_uv).rgb;
    col = mix(last_col, col, 1.0 / float(c.last_clear + 1));

    imageStore(the_texture, gl_uv, vec4(col, 1.0));
}