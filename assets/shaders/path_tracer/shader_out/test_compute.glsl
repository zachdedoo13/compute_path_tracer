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


#define MDEF Mat(vec3(0.0), vec3(0.0), 0.0, vec3(0.0), 0.0, 0.0, 0.0, 0.0, vec3(0.0))


// included "assets/shaders/path_tracer\\shapes.glsl"
float sdSphere(vec3 p, float s) {
    return length(p) - s;
}

float sdCube(vec3 p, vec3 b )
{
    vec3 q = abs(p) - b;
    return length(max(q,0.0)) + min(max(q.x,max(q.y,q.z)),0.0);
}

float sdOctahedronExact(vec3 p, float s)
{
    p = abs(p);
    float m = p.x+p.y+p.z-s;
    vec3 q;
    if( 3.0*p.x < m ) q = p.xyz;
    else if( 3.0*p.y < m ) q = p.yzx;
    else if( 3.0*p.z < m ) q = p.zxy;
    else return m*0.57735027;

    float k = clamp(0.5*(q.z-q.y+s),0.0,s);
    return length(vec3(q.x,q.y-s+k,q.z-k));
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
// included override "assets/shaders/path_tracer\\map"
Hit map(vec3 pos) { 
Hit[8] shapes;
vec3 tr;

      tr = pos;
      tr = move(tr, vec3(0, -2.32, 0));
      //rot
      shapes[0] = Hit(
         sdCube(tr * 1, vec3(11.63, 1, 12.26)) / 1,
         
         Mat(vec3(0, 0.6313726, 1), vec3(0, 0, 0), 0.75, vec3(0.50980395, 0.53333336, 1), 0.2, 0, 0, 0.015, vec3(0.4509804, 0, 0))
      
      );
      
      tr = pos;
      tr = move(tr, vec3(0, 3.25, 0));
      //rot
      shapes[1] = Hit(
         sdCube(tr * 1, vec3(11.63, 1, 12.26)) / 1,
         
         Mat(vec3(0, 0.2509804, 1), vec3(0, 0, 0), 0, vec3(0, 0, 0), 0, 0, 0.003, 0.015, vec3(0.4509804, 0, 0))
      
      );
      
      tr = pos;
      tr = move(tr, vec3(0, 3.23, 1.85));
      //rot
      shapes[2] = Hit(
         sdCube(tr * 1, vec3(1, 1, 2)) / 1,
         
         Mat(vec3(0, 0, 0), vec3(1.2, 1.2, 1.2), 0, vec3(0, 0, 0), 0, 0, 0, 0, vec3(0, 0, 0))
      
      );
      
      tr = pos;
      tr = move(tr, vec3(0, 3.25, 3.75));
      tr = rot3D(tr, vec3(1.4, 0, 0));
      shapes[3] = Hit(
         sdCube(tr * 1, vec3(11.63, 1, 12.26)) / 1,
         
         Mat(vec3(1, 0, 0), vec3(0, 0, 0), 0, vec3(0, 0, 0), 0, 0, 0, 0, vec3(0, 0, 0))
      
      );
      
      tr = pos;
      tr = move(tr, vec3(0, -0.06, 1.8));
      tr = rot3D(tr, vec3(1.61, 2.82, 1.87));
      shapes[4] = Hit(
         sdOctahedronExact(tr * 1, 1) / 1,
         
         Mat(vec3(0, 0, 0), vec3(0, 0, 0), 1, vec3(1, 1, 1), 0.555, 0, 0, 0, vec3(0, 0, 0))
      
      );
      
      tr = pos;
      tr = move(tr, vec3(-0.05, -0.2, 1.8));
      tr = rot3D(tr, vec3(1.7, 1.8, 2.68));
      shapes[5] = Hit(
         sdOctahedronExact(tr * 1, 1) / 1,
         
         Mat(vec3(0, 0, 0), vec3(0, 0, 0), 1, vec3(1, 1, 1), 0.555, 0, 0, 0, vec3(0, 0, 0))
      
      );
      
      tr = pos;
      tr = move(tr, vec3(3, 0, 2.1));
      tr = rot3D(tr, vec3(0, -0.22, 0));
      shapes[6] = Hit(
         sdCube(tr * 1, vec3(1, 1.15, 7.2)) / 1,
         
         Mat(vec3(0, 0, 0), vec3(0, 0, 0), 0.5, vec3(1, 1, 1), 0, 0, 0, 0, vec3(0, 0, 0))
      
      );
      
      tr = pos;
      tr = move(tr, vec3(-3, 0, 2.1));
      tr = rot3D(tr, vec3(0, 0.22, 0));
      shapes[7] = Hit(
         sdCube(tr * 1, vec3(1, 1.15, 7.2)) / 1,
         
         Mat(vec3(0, 0, 0), vec3(0, 0, 0), 0.5, vec3(1, 1, 1), 0, 0, 0, 0, vec3(0, 0, 0))
      
      );
      
      Hit back = Hit(10000.0, MDEF);
      for (int i = 0; i < 8; i ++) {
         back = opUnion(back, shapes[i]);
      }

      return back;
   }

      
// end include
// included "assets/shaders/path_tracer\\funcs.glsl"
vec2 calc_uv(vec2 gl_uv, ivec2 dimentions) {
    vec2 uv = vec2(gl_uv.x / float(dimentions.x), gl_uv.y / float(dimentions.y));
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
    const vec3 e = vec3(.0001, 0.0, 0.0);
    return normalize(
        vec3(
            pull(p, e.xyy) - pull(p, -e.xyy),
            pull(p, e.yxy) - pull(p, -e.yxy),
            pull(p, e.yyx) - pull(p, -e.yyx)
        )
    );
}



// Color correction // in the wgsl code
vec3 LessThan(vec3 f, float value)
{
    return vec3(
    (f.x < value) ? 1.0f : 0.0f,
    (f.y < value) ? 1.0f : 0.0f,
    (f.z < value) ? 1.0f : 0.0f);
}

vec3 LinearToSRGB(vec3 rgb)
{
    rgb = clamp(rgb, 0.0f, 1.0f);

    return mix(
    pow(rgb, vec3(1.0f / 2.4f)) * 1.055f - 0.055f,
    rgb * 12.92f,
    LessThan(rgb, 0.0031308f)
    );
}

vec3 SRGBToLinear(vec3 rgb)
{
    rgb = clamp(rgb, 0.0f, 1.0f);

    return mix(
    pow(((rgb + 0.055f) / 1.055f), vec3(2.4f)),
    rgb / 12.92f,
    LessThan(rgb, 0.04045f)
    );
}

// ACES tone mapping curve fit to go from HDR to LDR
//https://knarkowicz.wordpress.com/2016/01/06/aces-filmic-tone-mapping-curve/
vec3 ACESFilm(vec3 x)
{
    float a = 2.51f;
    float b = 0.03f;
    float c = 2.43f;
    float d = 0.59f;
    float e = 0.14f;
    return clamp((x*(a*x + b)) / (x*(c*x + d) + e), 0.0f, 1.0f);
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


// included "assets/shaders/path_tracer\\tester.glsl"
struct Cube {
    vec3 min;
    vec3 max;
};

float expensiveFunction(vec3 p, float s) {
    float result = 0.0;
    for (int i = 0; i < 100; i++) {
        result += sqrt(float(i) * 0.3214);
    }
    return 1.0 / result;
}

float expensive_sdf(vec3 p, float s) {
    return length(p) - s - (expensiveFunction(p, s) * 0.0000001);
    //    return length(p) - s;
}

vec2 intersectAABB(Ray ray, Cube cube) {
    vec3 tMin = (cube.min - ray.ro) / ray.rd;
    vec3 tMax = (cube.max - ray.ro) / ray.rd;
    vec3 t1 = min(tMin, tMax);
    vec3 t2 = max(tMin, tMax);
    float tNear = max(max(t1.x, t1.y), t1.z);
    float tFar = min(min(t2.x, t2.y), t2.z);
    return vec2(tNear, tFar);
}

bool bool_hit(vec2 intersect) {
    return intersect.x < intersect.y && intersect.y > 0.0;
}

Cube from_pos_size(vec3 pos, vec3 size) {
    Cube cube;
    cube.min = pos - size;
    cube.max = pos + size;

    return cube;
}


#define A 4
const mat2x3[A] objects = mat2x3[A](
    mat2x3(vec3(0, -2.32, 0), vec3(11.63, 1, 12.26)), // floor
    mat2x3(vec3(0, 3.25, 0), vec3(1.0, 1.0, 1.0)),
    mat2x3(vec3(0, -0.06, 1.8), vec3(1.0)),
    mat2x3(vec3(-0.05, -0.2, 1.8), vec3(1.0))
);

#define OTA bool[A]

Hit test_map(vec3 pos, OTA test) {
    vec3 tr;
    Hit back = Hit(BIGNUM, MDEF);
    Hit temp;

    if (s.aabb == 1) {
        for (int i = 0; i < A; i++) { test[i] = true; }
    }

    // 0
    if (test[0]) {
        tr = pos;
        tr = move(tr, objects[0][0]);
        //rot
        temp = Hit(
        sdCube(tr * 1, objects[0][1]) / 1,

        Mat(vec3(0, 0.6313726, 1), vec3(0.0), 0.75, vec3(0.50980395, 0.53333336, 1), 0.2, 0, 0, 0.015, vec3(0.4509804, 0, 0))
        );

        back = opUnion(back, temp);
    }

    if (test[1]) {
        tr = pos;
        tr = move(tr, objects[1][0]);
        //rot
        temp = Hit(
        sdCube(tr * 1, objects[1][1]) / 1,

        Mat(vec3(0, 0, 0), vec3(2.0), 0, vec3(0, 0, 0), 0, 0, 0, 0, vec3(0, 0, 0))
        );

        back = opUnion(back, temp);
    }

    if (test[2]) {
        tr = pos;
        tr = move(tr, objects[2][0]);
        tr = rot3D(tr, vec3(1.61, 2.82, 1.87));
        temp =  Hit(
        sdOctahedronExact(tr * 1, 1) / 1,

        Mat(vec3(1, 0, 0), vec3(0), 0.0, vec3(0.5, 0, 0), 1.0, 0, 0, 0, vec3(0, 0, 0))
        );

        back = opUnion(back, temp);
    }

    if (test[3]) {
        tr = pos;
        tr = move(tr, objects[3][0]);
        tr = rot3D(tr, vec3(1.7, 1.8, 2.68));
        temp =  Hit(
        sdOctahedronExact(tr * 1, 1) / 1,

        Mat(vec3(0, 0, 0), vec3(0, 0, 0), 1, vec3(1, 1, 1), 0.555, 0, 0, 0, vec3(0, 0, 0))
        );

        back = opUnion(back, temp);
    }


    return back;
}


float t_pull(vec3 p, vec3 e, OTA hits)
{
    return test_map(p + e, hits).d;
}

vec3 test_calc_normal(vec3 p, OTA hits) {
    const vec3 e = vec3(.0001, 0.0, 0.0);
    return normalize(
        vec3(
            t_pull(p, e.xyy, hits) - t_pull(p, -e.xyy, hits),
            t_pull(p, e.yxy, hits) - t_pull(p, -e.yxy, hits),
            t_pull(p, e.yyx, hits) - t_pull(p, -e.yyx, hits)
        )
    );
}


OTA bounds_map(Ray ray, inout int bc) {
    vec2 inter;
    OTA hits;


    for (int i = 0; i < A; i++) {
        if (bool_hit(intersectAABB(ray, from_pos_size(objects[i][0], objects[i][1] * 1.5)))) {
            hits[i] = true;

            bc += 1;
        }
    }


    return hits;
}



Hit TestCastRay(Ray ray, inout int steps, OTA bounds) {
    float t = 0.0;
    Mat mat;
    for (steps = 0; steps < STEPS; steps++) {
        vec3 p = ray.ro + ray.rd * t;
        Hit hit = test_map(p, bounds);
        mat = hit.mat;
        t += hit.d;

        if (hit.d < MHD) break;
        if (t > FP) break;
    }
    return Hit(t, mat);
}



vec3 test_cast(Ray in_ray) {
    Ray ray = in_ray;

    vec3 col = vec3(0.0);
    vec3 intersect_color = vec3(0.0);

    int steps;

    if (s.bounces < 8)
    {
        OTA hits;
        for (int i = 0; i < A; i++) {hits[i] = true; }
        float d = TestCastRay(ray, steps, hits).d;
        if (d < FP) { col.b += d / 5.0; }
        col.g = float(steps) / float(STEPS);
        return col;
    }


        int bc = 0;
    OTA hits = bounds_map(ray, bc);

    col.r = float(bc) / float(A);


    float d = TestCastRay(ray, steps, hits).d;
    if (d < FP) { col.b += d / 5.0; }



    col.g = float(steps) / float(STEPS);

    return col + intersect_color;
}

// end include


Hit CastRay(Ray ray) {
    float t = 0.0;
    Mat mat;
    for (int i = 0; i < STEPS; i++) {
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

            ret += hit.mat.light * throughput;
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

    return test_cast(ray);
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
