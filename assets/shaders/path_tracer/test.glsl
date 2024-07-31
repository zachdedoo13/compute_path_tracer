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


const vec4[4] objects = vec4[4](
    vec4(-2.0, 0.0, 3.0, 1.0),
    vec4(2.0, 0.0, 3.0, 1.0),
    vec4(0.0, -2.0, 3.0, 1.0),
    vec4(0.0, 2.0, 3.0, 1.0)
);

#define OTA bool[4]

Hit test_map(vec3 pos, OTA test) {

    Hit back = Hit(BIGNUM, MDEF);
    Hit temp;

    if (test[0]) {
        temp.d = expensive_sdf(pos - objects[0].xyz, objects[0].w);
        temp.mat = MDEF;

        back = opUnion(back, temp);
    }

    if (test[1]) {
        temp.d = expensive_sdf(pos - objects[1].xyz, objects[1].w);
        temp.mat = MDEF;

        back = opUnion(back, temp);
    }

    if (test[2]) {
        temp.d = expensive_sdf(pos - objects[2].xyz, objects[2].w);
        temp.mat = MDEF;

        back = opUnion(back, temp);
    }

    if (test[3]) {
        temp.d = expensive_sdf(pos - objects[3].xyz, objects[3].w);
        temp.mat = MDEF;

        back = opUnion(back, temp);
    }

    return back;
}

OTA


bounds_map(Ray ray) {
    vec2 inter;
    OTA hits;

    // 0
    inter = intersectAABB(ray, from_pos_size(vec3(objects[0].xyz), vec3(objects[0].w)));
    if (bool_hit(inter)) {
        hits[0] = true;
    }

    inter = intersectAABB(ray, from_pos_size(vec3(objects[1].xyz), vec3(objects[1].w)));
    if (bool_hit(inter)) {
        hits[1] = true;
    }

    inter = intersectAABB(ray, from_pos_size(vec3(objects[2].xyz), vec3(objects[2].w)));
    if (bool_hit(inter)) {
        hits[2] = true;
    }

    inter = intersectAABB(ray, from_pos_size(vec3(objects[3].xyz), vec3(objects[3].w)));
    if (bool_hit(inter)) {
        hits[3] = true;
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

    if (s.bounces < 8) {
        OTA hits = OTA(true, true, true, true);
        float d = TestCastRay(ray, steps, hits).d;
        if (d < FP) { col.b += d / 5.0; }
        col.g = float(steps) / float(STEPS);
        return col;
    }


    OTA hits = bounds_map(ray);


    float d = TestCastRay(ray, steps, hits).d;
    if (d < FP) { col.b += d / 5.0; }



    col.g = float(steps) / float(STEPS);

    return col + intersect_color;
}
