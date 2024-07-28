vec2 calc_uv(ivec2 gl_uv, ivec2 dimentions) {
    vec2 uv = vec2(float(gl_uv.x) / float(dimentions.x), float(gl_uv.y) / float(dimentions.y));
    uv = uv * 2.0 - 1.0;
    uv.x *= c.aspect;

    return uv;
}