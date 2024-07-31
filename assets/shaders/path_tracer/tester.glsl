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
