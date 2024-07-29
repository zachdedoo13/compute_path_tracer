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
#define FP 100.0

//!code start flag

struct Ray {vec3 ro; vec3 rd; };

struct Mat {vec3 col; };
#define MDEF Mat(vec3(0.0))

struct Hit {float d; Mat mat; };


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
Hit[1] shapes;
vec3 tr;

      tr = pos;
      //pos
      tr = rot3D(tr, vec3(0.62, 0.47, 0));
      shapes[0] = Hit(
         sdCube(tr * 1, vec3(1, 1, 1)) / 1,
         
      Mat(vec3(0.17254902, 0, 1))
      
      );
      
      Hit back = Hit(10000.0, MDEF);
      for (int i = 0; i < 1; i ++) {
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

vec3 path_trace(Ray ray) {

    Hit test = CastRay(ray);

    if (test.d > FP) { return vec3(0.0); }

    return calc_normal(calc_point(ray, test.d)) * test.mat.col;
//    return test.mat.col;
}


void main() {
    ivec2 gl_uv = ivec2(gl_GlobalInvocationID.xy);
    ivec2 dimentions = imageSize(the_texture);
    if (bounds_check(gl_uv, dimentions)) { return; }
    vec2 uv = calc_uv(gl_uv, dimentions);





    Ray ray = Ray(
        vec3(0.0, 0.0, -3.0), // origin
        normalize(vec3(uv, 1.0)) // direction
    );

    vec3 col = path_trace(ray);

    imageStore(the_texture, gl_uv, vec4(col, 1.0));
}
