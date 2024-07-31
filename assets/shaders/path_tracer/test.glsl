struct Cube {
    vec3 min;
    vec3 max;
};

float expensiveFunction(vec3 p, float s) {
    float result = 0.0;
    for (int i = 0; i < 5000; i++) {
        result += sqrt(float(i) * 0.3214);
    }
    return 1.0 / result;
}

float expensive_sdf(vec3 p, float s) {
    return length(p) - s - (expensiveFunction(p, s) * 0.0000001);
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

Hit test_map(vec3 pos) {
    Hit sphere;
    sphere.d = expensive_sdf(pos - vec3(0.0, sin(c.time) * 2.0, 0.0), 1.0);
    sphere.mat = MDEF;

    return sphere;
}

Hit TestCastRay(Ray ray) {
    float t = 0.0;
    Mat mat;
    for (int i = 0; i < STEPS; i++) {
        vec3 p = ray.ro + ray.rd * t;
        Hit hit = test_map(p);
        mat = hit.mat;
        t += hit.d;

        if (abs(hit.d) < MHD) break;
        if (t > FP) break;
    }
    return Hit(t, mat);
}


vec3 test_cast(Ray in_ray) {
    Ray ray = in_ray;
    ray.ro.x += sin(c.time) * 3.0;

    vec3 col = vec3(0.0);
    vec3 intersect_color = vec3(0.0);

    float d = TestCastRay(ray).d;
    if (d < FP) { col.b += d / 5.0; }

//    vec2 intersect = intersectAABB(ray, from_pos_size(vec3(0.0, sin(c.time) * 2.0, 0.0), vec3(1.0)));  // Assuming the cube remains the same
//    if (bool_hit(intersect)) {
//        intersect_color.r = 0.01;  // Return a color (e.g., white) if the ray intersects the AABB
//
//        float d = TestCastRay(ray).d;
//        if (d < FP) { col.b += d / 5.0; }
//    }

    return col + intersect_color;
}
