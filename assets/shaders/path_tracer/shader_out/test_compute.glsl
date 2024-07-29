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


// included "assets/shaders/path_tracer\\shapes.glsl"
float sdSphere(vec3 p, float s) {
    return length(p) - s;
}

float sdCube(vec3 p, vec3 b )
{
    vec3 q = abs(p) - b;
    return length(max(q,0.0)) + min(max(q.x,max(q.y,q.z)),0.0);
}



// methods
vec3 move(vec3 p, vec3 by) {
    return p - by;
}

vec3 rot3D(vec3 p, vec3 rot) {
    // Rotation around X-axis
    float cosX = cos(rot.x);
    float sinX = sin(rot.x);
    mat3 rotX = mat3(
    1.0, 0.0, 0.0,
    0.0, cosX, -sinX,
    0.0, sinX, cosX
    );

    // Rotation around Y-axis
    float cosY = cos(rot.y);
    float sinY = sin(rot.y);
    mat3 rotY = mat3(
    cosY, 0.0, sinY,
    0.0, 1.0, 0.0,
    -sinY, 0.0, cosY
    );

    // Rotation around Z-axis
    float cosZ = cos(rot.z);
    float sinZ = sin(rot.z);
    mat3 rotZ = mat3(
    cosZ, -sinZ, 0.0,
    sinZ, cosZ, 0.0,
    0.0, 0.0, 1.0
    );

    // Apply rotations
    p = rotX * p;
    p = rotY * p;
    p = rotZ * p;

    return p;
}


// unions
Hit opUnion(Hit v1, Hit v2) {
    return v1.d < v2.d ? v1 : v2;
}
// end include
// included override "assets/shaders/path_tracer\\map.glsl"
Hit map(vec3 pos) { 
Hit[8] shapes;
vec3 tr;

      tr = pos;
      tr = move(tr, vec3(0, -0.92, 0));
      //rot
      shapes[0] = Hit(
         sdCube(tr * 1, vec3(5.36, 0.01, 4.25)) / 1,
         
         Mat(vec3(1, 1, 1), vec3(0, 0, 0), 0, vec3(0, 0, 0), 0)
      
      );
      
      tr = pos;
      tr = move(tr, vec3(0, 0, 2.25));
      //rot
      shapes[1] = Hit(
         sdCube(tr * 1, vec3(4.55, 4.41, 0.47)) / 1,
         
         Mat(vec3(1, 0, 0), vec3(0, 0, 0), 0, vec3(0, 0, 0), 0)
      
      );
      
      tr = pos;
      tr = move(tr, vec3(-0.32, 0, 0));
      tr = rot3D(tr, vec3(0.51, 0.71, 0.67));
      shapes[2] = Hit(
         sdCube(tr * 4.4444447, vec3(1, 1, 1)) / 4.4444447,
         
         Mat(vec3(0, 0, 0), vec3(0, 0, 0), 0, vec3(0, 0, 0), 0)
      
      );
      
      tr = pos;
      tr = move(tr, vec3(2.22, 0, 2.25));
      tr = rot3D(tr, vec3(0, 1.53, 0));
      shapes[3] = Hit(
         sdCube(tr * 1, vec3(4.55, 4.41, 0.47)) / 1,
         
         Mat(vec3(0.3137255, 1, 0), vec3(0, 0, 0), 0, vec3(0, 0, 0), 0)
      
      );
      
      tr = pos;
      tr = move(tr, vec3(-2.45, 0, 2.25));
      tr = rot3D(tr, vec3(0, 1.53, 0));
      shapes[4] = Hit(
         sdCube(tr * 1, vec3(4.55, 4.41, 0.47)) / 1,
         
         Mat(vec3(0, 0.1882353, 1), vec3(0, 0, 0), 0, vec3(0, 0, 0), 0)
      
      );
      
      tr = pos;
      tr = move(tr, vec3(0, 5.65, 0));
      //rot
      shapes[5] = Hit(
         sdCube(tr * 1, vec3(5.36, 0.22, 4.25)) / 1,
         
         Mat(vec3(0, 0, 0), vec3(1.45, 1.45, 1.45), 0, vec3(0, 0, 0), 0)
      
      );
      
      tr = pos;
      tr = move(tr, vec3(-1.3, -0.25, 0));
      //rot
      shapes[6] = Hit(
         sdSphere(tr * 1, 0.4) / 1,
         
         Mat(vec3(0.90588236, 0.93333334, 0), vec3(0, 0, 0), 0, vec3(0, 0, 0), 0)
      
      );
      
      tr = pos;
      tr = move(tr, vec3(0.95, 0, 0));
      tr = rot3D(tr, vec3(0.51, 0.71, 0.67));
      shapes[7] = Hit(
         sdCube(tr * 2.7777777, vec3(1, 1, 1)) / 2.7777777,
         
         Mat(vec3(1, 1, 1), vec3(0, 0, 0), 0, vec3(0, 0, 0), 0)
      
      );
      
      Hit back = Hit(10000.0, MDEF);
      for (int i = 0; i < 8; i ++) {
         back = opUnion(back, shapes[i]);
      }

      return back;
   }

      
// end include
// included "assets/shaders/path_tracer\\funcs.glsl"
vec2 calc_uv(ivec2 gl_uv, ivec2 dimentions) {
    vec2 uv = vec2(float(gl_uv.x) / float(dimentions.x), float(gl_uv.y) / float(dimentions.y));
    uv = uv * 2.0 - 1.0;
    uv.x *= c.aspect;

    return uv;
}

bool bounds_check(ivec2 gl_uv, ivec2 dimentions) {
    if (gl_uv.x > dimentions.x || gl_uv.y > dimentions.y) {
        return true;
    }
    return false;
}

vec3 calc_point(Ray ray, float dist) {
    return ray.ro + ray.rd * dist;
}


float pull(vec3 p, vec3 e)
{
    return map(p + e).d;
}

vec3 calc_normal(vec3 p) {
    const vec3 e = vec3(.001, 0.0, 0.0);
    return normalize(
        vec3(
            pull(p, e.xyy) - pull(p, -e.xyy),
            pull(p, e.yxy) - pull(p, -e.yxy),
            pull(p, e.yyx) - pull(p, -e.yyx)
        )
    );
}
// end include
// included "assets/shaders/path_tracer\\rng.glsl"
uint wang_hash(inout uint seed)
{
    seed = uint(seed ^ uint(61)) ^ uint(seed >> uint(16));
    seed *= uint(9);
    seed = seed ^ (seed >> 4);
    seed *= uint(0x27d4eb2d);
    seed = seed ^ (seed >> 15);
    return seed;
}
 
float RandomFloat01(inout uint state)
{
    return float(wang_hash(state)) / 4294967296.0;
}
 
vec3 RandomUnitVector(inout uint state)
{
    float z = RandomFloat01(state) * 2.0f - 1.0f;
    float a = RandomFloat01(state) * PI2;
    float r = sqrt(1.0f - z * z);
    float x = r * cos(a);
    float y = r * sin(a);
    return vec3(x, y, z);
}

uint gen_rng(ivec2 gl_uv, int frame, ivec2 dimentions) 
{
    vec2 f_dim = vec2(float(dimentions.x), float(dimentions.y));
    return uint(
        uint(( float(gl_uv.x) * 0.5 + 0.5) *
        float(f_dim.x)) * uint(1973) +
        uint((float(gl_uv.y) * 0.5 + 0.5) *
        float(f_dim.y)) * uint(9277) +
        uint(frame) * uint(26699)
    ) | uint(1);
}
// end include




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
