#version 450

layout (local_size_x = 16, local_size_y = 16, local_size_z = 1) in;

layout(set = 0, binding = 0, rgba32f) uniform image2D the_texture;

layout(set = 1, binding = 0) uniform Constants {
    float time;
    int cframe;
    float aspect;
    int last_clear;
} c;

#define O1 sin(c.time)
#define AO1 abs(sin(c.time))
#define SAO1 smoothstep(0.0, 3.0, abs(sin(c.time)))


#define MHD 0.001
#define FP 5.0

//!code start flag

struct Ray {vec3 ro; vec3 rd; };
struct Hit {float d; };

#include "funcs.glsl"



Hit map(vec3 p) {
    return Hit(length(p) - 1.0);
}

Hit CastRay(Ray ray) {
    float t = 0.0;
    for (int i = 0; i < 80; i++) {
        vec3 p = ray.ro + ray.rd * t;
        Hit hit = map(p);
        t += hit.d;

        if (abs(hit.d) < MHD) break;
        if (t > FP) break;
    }
    return Hit(t);
}


void main() {
    ivec2 gl_uv = ivec2(gl_GlobalInvocationID.xy);
    ivec2 dimentions = imageSize(the_texture);
    vec2 uv = calc_uv(gl_uv, dimentions);



    Ray ray = Ray(
        vec3(0.0, 0.0, -3.0), // origin
        normalize(vec3(uv, 1.0)) // direction
    );


    Hit test = CastRay(ray);



    imageStore(the_texture, gl_uv, vec4( SAO1 / vec3(test.d / FP), 1.0));
}