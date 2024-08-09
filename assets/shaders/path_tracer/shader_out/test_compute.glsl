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

// included "assets/shaders/path_tracer\\aabb.glsl"


struct AABB {
    vec3 min;
    vec3 max;
};






AABB from_pos_size(vec3 pos, vec3 size) {
    AABB cube;
    cube.min = pos - size;
    cube.max = pos + size;

    return cube;
}

vec2 intersectAABB(Ray ray, AABB cube) {
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
// end include
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

Hit opSubtraction(Hit v1, Hit v2) {
//    return -v1.d > v2.d ? v1 : v2;
    Hit nv1 = Hit(-v1.d, v1.mat);
    float depth = max(nv1.d, v2.d);
    return depth == nv1.d ? nv1 : v2;
}
// end include
// included override "assets/shaders/path_tracer\\map"

         #define CHECK_ARRAY bool[7]

         CHECK_ARRAY bounds(Ray ray, inout vec3 debug) {
            debug = vec3(0.0);
            CHECK_ARRAY back;
            float scale;

            
      if (false) {

      back[0] = true;
      debug += 0.1;

      }
      
      if (false) {

      back[1] = true;
      debug += 0.1;

      }
      
      if (bool_hit(intersectAABB(ray, from_pos_size(vec3(data[68], data[69], data[70]) + vec3(data[76], data[77], data[78]), (vec3(data[83]) * ( data[67] * data[75])) * data[82] )))) {

      back[2] = true;
      debug += 0.1;

      }
      
      if (bool_hit(intersectAABB(ray, from_pos_size(vec3(data[68], data[69], data[70]) + vec3(data[103], data[104], data[105]), (vec3(data[110], data[111], data[112]) * ( data[67] * data[102])) * data[109] )))) {

      back[3] = true;
      debug += 0.1;

      }
      
      if (bool_hit(intersectAABB(ray, from_pos_size(vec3(data[68], data[69], data[70]) + vec3(data[132], data[133], data[134]), (vec3(data[139]) * ( data[67] * data[131])) * data[138] )))) {

      back[4] = true;
      debug += 0.1;

      }
      
      if (bool_hit(intersectAABB(ray, from_pos_size(vec3(data[68], data[69], data[70]) + vec3(data[159], data[160], data[161]), (vec3(data[166]) * ( data[67] * data[158])) * data[165] )))) {

      back[5] = true;
      debug += 0.1;

      }
      
      if (bool_hit(intersectAABB(ray, from_pos_size(vec3(data[68], data[69], data[70]) + vec3(data[186], data[187], data[188]), (vec3(data[193], data[194], data[195]) * ( data[67] * data[185])) * data[192] )))) {

      back[6] = true;
      debug += 0.1;

      }
      

            return back;

         }

       

 
      #define MAXHIT Hit(10000.0, MDEF)

      Hit map(vec3 pu0, CHECK_ARRAY check) {
         Hit start = MAXHIT;

      {
Hit u1 = MAXHIT; 
vec3 pu1 = pu0;
 pu1 *= 1.0 / data[1];
 pu1 = move(pu1, vec3(data[2], data[3], data[4]) * (1.0 / data[1]));
 pu1 = rot3D(pu1, vec3(data[5], data[6], data[7]));
if (true) {

      vec3 u1s0p = pu1;
 u1s0p *= 1.0 / data[9];
 u1s0p = move(u1s0p, vec3(data[10], data[11], data[12]) * (1.0 / data[9]));
 u1s0p = rot3D(u1s0p, vec3(data[13], data[14], data[15]));

      Hit u1s0 = Hit(
         sdCube(u1s0p, vec3(data[17], data[18], data[19])),
         Mat(vec3(data[20], data[21], data[22]), data[23], vec3(data[24], data[25], data[26]), data[27], vec3(data[28], data[29], data[30]), data[31], data[32], data[33], data[34], vec3(data[35], data[36], data[37]))
      );
      u1s0.d /= 1.0 / data[9];

      u1 = u1s0;


      }
if (true) {

      vec3 u1s1p = pu1;
 u1s1p *= 1.0 / data[38];
 u1s1p = move(u1s1p, vec3(data[39], data[40], data[41]) * (1.0 / data[38]));
 u1s1p = rot3D(u1s1p, vec3(data[42], data[43], data[44]));

      Hit u1s1 = Hit(
         sdCube(u1s1p, vec3(data[46], data[47], data[48])),
         Mat(vec3(data[49], data[50], data[51]), data[52], vec3(data[53], data[54], data[55]), data[56], vec3(data[57], data[58], data[59]), data[60], data[61], data[62], data[63], vec3(data[64], data[65], data[66]))
      );
      u1s1.d /= 1.0 / data[38];

      u1 = opSubtraction(u1, u1s1);


      }
u1.d /= 1.0 / data[1];
start = opUnion(start, u1);} //BOX
{
Hit u1 = MAXHIT; 
vec3 pu1 = pu0;
 pu1 *= 1.0 / data[67];
 pu1 = move(pu1, vec3(data[68], data[69], data[70]) * (1.0 / data[67]));
 pu1 = rot3D(pu1, vec3(data[71], data[72], data[73]));
if (check[2])  {

      vec3 u1s0p = pu1;
 u1s0p *= 1.0 / data[75];
 u1s0p = move(u1s0p, vec3(data[76], data[77], data[78]) * (1.0 / data[75]));
 u1s0p = rot3D(u1s0p, vec3(data[79], data[80], data[81]));

      Hit u1s0 = Hit(
         sdSphere(u1s0p, data[83]),
         Mat(vec3(data[84], data[85], data[86]), data[87], vec3(data[88], data[89], data[90]), data[91], vec3(data[92], data[93], data[94]), data[95], data[96], data[97], data[98], vec3(data[99], data[100], data[101]))
      );
      u1s0.d /= 1.0 / data[75];

      u1 = u1s0;


      }
if (check[3])  {

      vec3 u1s1p = pu1;
 u1s1p *= 1.0 / data[102];
 u1s1p = move(u1s1p, vec3(data[103], data[104], data[105]) * (1.0 / data[102]));
 u1s1p = rot3D(u1s1p, vec3(data[106], data[107], data[108]));

      Hit u1s1 = Hit(
         sdCube(u1s1p, vec3(data[110], data[111], data[112])),
         Mat(vec3(data[113], data[114], data[115]), data[116], vec3(data[117], data[118], data[119]), data[120], vec3(data[121], data[122], data[123]), data[124], data[125], data[126], data[127], vec3(data[128], data[129], data[130]))
      );
      u1s1.d /= 1.0 / data[102];

      u1 = opUnion(u1, u1s1);


      }
if (check[4])  {

      vec3 u1s2p = pu1;
 u1s2p *= 1.0 / data[131];
 u1s2p = move(u1s2p, vec3(data[132], data[133], data[134]) * (1.0 / data[131]));
 u1s2p = rot3D(u1s2p, vec3(data[135], data[136], data[137]));

      Hit u1s2 = Hit(
         sdSphere(u1s2p, data[139]),
         Mat(vec3(data[140], data[141], data[142]), data[143], vec3(data[144], data[145], data[146]), data[147], vec3(data[148], data[149], data[150]), data[151], data[152], data[153], data[154], vec3(data[155], data[156], data[157]))
      );
      u1s2.d /= 1.0 / data[131];

      u1 = opUnion(u1, u1s2);


      }
if (check[5])  {

      vec3 u1s3p = pu1;
 u1s3p *= 1.0 / data[158];
 u1s3p = move(u1s3p, vec3(data[159], data[160], data[161]) * (1.0 / data[158]));
 u1s3p = rot3D(u1s3p, vec3(data[162], data[163], data[164]));

      Hit u1s3 = Hit(
         sdSphere(u1s3p, data[166]),
         Mat(vec3(data[167], data[168], data[169]), data[170], vec3(data[171], data[172], data[173]), data[174], vec3(data[175], data[176], data[177]), data[178], data[179], data[180], data[181], vec3(data[182], data[183], data[184]))
      );
      u1s3.d /= 1.0 / data[158];

      u1 = opUnion(u1, u1s3);


      }
if (check[6])  {

      vec3 u1s4p = pu1;
 u1s4p *= 1.0 / data[185];
 u1s4p = move(u1s4p, vec3(data[186], data[187], data[188]) * (1.0 / data[185]));
 u1s4p = rot3D(u1s4p, vec3(data[189], data[190], data[191]));

      Hit u1s4 = Hit(
         sdCube(u1s4p, vec3(data[193], data[194], data[195])),
         Mat(vec3(data[196], data[197], data[198]), data[199], vec3(data[200], data[201], data[202]), data[203], vec3(data[204], data[205], data[206]), data[207], data[208], data[209], data[210], vec3(data[211], data[212], data[213]))
      );
      u1s4.d /= 1.0 / data[185];

      u1 = opUnion(u1, u1s4);


      }
u1.d /= 1.0 / data[67];
start = opUnion(start, u1);} //stuffs

         return start;
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


float pull(vec3 p, vec3 e, CHECK_ARRAY check)
{
    return map(p + e, check).d;
}

vec3 calc_normal(vec3 p, CHECK_ARRAY check) {
    const vec3 e = vec3(.0001, 0.0, 0.0);
    return normalize(
        vec3(
            pull(p, e.xyy, check) - pull(p, -e.xyy, check),
            pull(p, e.yxy, check) - pull(p, -e.yxy, check),
            pull(p, e.yyx, check) - pull(p, -e.yyx, check)
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




Hit CastRay(Ray ray, CHECK_ARRAY check) {
    float t = 0.0;
    Mat mat;
    for (int i = 0; i < STEPS; i++) {
        vec3 p = ray.ro + ray.rd * t;
        Hit hit = map(p, check);
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
        vec3 cube_debug;
        CHECK_ARRAY check = bounds(ray, cube_debug);

        Hit hit = CastRay(ray, check);

        // out of bounds
        if (hit.d > FP) {
            break;
        }

        // update the ray position
        vec3 hit_pos = calc_point(ray, hit.d);


//       vec3 hit_normal = test_calc_normal(hit_pos, hits);

        vec3 hit_normal = calc_normal(hit_pos, check);

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
    vec3 cube_debug;
    CHECK_ARRAY check = bounds(ray, cube_debug);

    Hit test = CastRay(ray, check);

    if (test.d > FP) { return cube_debug; }

    return (normalize(calc_normal(calc_point(ray, test.d), check)) * 0.5 + 0.5) * 0.2 + cube_debug;
}



vec3 colors(Ray ray) {

//    Hit test = CastRay(ray);

//    if (test.d > FP) { return vec3(0.0); }

//    return test.mat.col;
    vec3 cube_debug;
    CHECK_ARRAY check = bounds(ray, cube_debug);

    return CastRay(ray, check).mat.col;
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
