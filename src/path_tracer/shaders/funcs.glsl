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