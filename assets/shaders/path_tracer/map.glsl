//Hit map(vec3 pos) {
//    Hit[2] shapes;
//    vec3 tr;
//
//    tr = pos;
//    tr = move(tr, vec3(2.46, 0.35, 0));
//    //rot
//    shapes[0] = Hit(
//    sdCube(tr * 1, vec3(1, 1, 1)) / 1,
//
//    Mat(vec3(1, 1, 1), vec3(0, 0, 0), 0, vec3(0, 0, 0), 0)
//
//    );
//
//    tr = pos;
//    //pos
//    //rot
//    shapes[1] = Hit(
//    sdSphere(tr * 1, 1) / 1,
//
//    Mat(vec3(1, 1, 1), vec3(0, 0, 0), 0, vec3(0, 0, 0), 0)
//
//    );
//
//    Hit back = shapes[0];
//    for (int i = 1; i < 2; i ++) {
//        back = opUnion(back, shapes[i]);
//    }
//
//    return back;
//}

//#define MAXHIT Hit(10000.0, MDEF)

//Hit map(vec3 pu0) {
//    // top level is allways min union
//    Hit u0 = MAXHIT;
//
//        Hit u1 = MAXHIT;
//            // first union transform
//            vec3 pu1 = move(pu0, vec3(0.0));
//
//            Hit u2 = MAXHIT;
//                // second union transform
//                vec3 pu2 = move(pu1, vec3(0.0));
//
//                Hit u2s1 = Hit(
//                    sdCube(move(pu2, vec3(1.0)) * 1, vec3(1, 1, 1)) / 1,
//                    MDEF
//                );
//                u2 = opUnion(u2, u2s1);
//
//            u1 = opUnion(u1, u2);
//
//    u0 = opUnion(u0, u1);
//
//
//    Hit u0s1 = Hit(
//        sdSphere(move(pu0, vec3(0.0)) * 1, 0.5) / 1,
//        MDEF
//    );
//    u0 = opUnion(u0, u0s1);
//
//    return u0;
//}

//Hit map(vec3 pu0) {
//    Hit u0 = MAXHIT;
//
//    // union 1
//    { // bounds 1
//        Hit u1 = MAXHIT;
//        vec3 pu1 = move(pu0, vec3(0.0));
//
//        {  // bounds 2
//            Hit u1s0 = Hit(
//                sdCube(move(pu1, vec3(1.0)) * 1, vec3(1, 1, 1)) / 1,
//                MDEF
//            );
//            u1 = opUnion(u1, u1s0);
//        }
//
//        u0 = opUnion(u0, u1);
//    }
//
//    { // bounds 3
//        Hit u0s0 = Hit(
//            sdSphere(move(pu0, vec3(0.0)) * 1, 0.5) / 1,
//            MDEF
//        );
//        u0 = opUnion(u0, u0s0);
//    }
//
//
//    return u0;
//}


#define MAXHIT Hit(10000.0, MDEF)
#define CHECK_ARRAY bool[2]

Hit map(vec3 pu0, CHECK_ARRAY check) {
    Hit start = MAXHIT;

    if (check[0]) {
        Hit u1 = MAXHIT;
        vec3 pu1 = pu0;
        pu1 *= 1.0 / 1.0;
        pu1 = move(pu1, vec3(0, 0, 0));
        pu1 = rot3D(pu1, vec3(0, 0, 0));

        if (check[1]) {
            vec3 u1s0p = pu1;
            u1s0p *= 1.0 / 0.5;
            u1s0p = move(u1s0p, vec3(0, 0, 0));
            u1s0p = rot3D(u1s0p, vec3(0, 0, 0));

            Hit u1s0 = Hit(
                sdSphere(u1s0p, 1),
                MDEF
            );
            u1s0.d /= 1.0 / 0.5;

            u1 = opUnion(u1, u1s0);
        }

        u1.d /= 1.0 / 1.0;
        start = opUnion(start, u1);
    }

    return start;
}

bool[2] bounds(Ray ray, inout vec3 debug) {
    debug = vec3(0.0);
    bool[2] back;
    float scale;

    scale = 1.0;
    if (bool_hit(intersectAABB(ray, from_pos_size(vec3(0.0), vec3(1.0) * scale)))) {
        back[0] = true;
        debug.g += 0.3;

        scale *= 0.5;
        if (bool_hit(intersectAABB(ray, from_pos_size(vec3(0.0), vec3(1.0) * scale )))) {
            back[1] = true;
            debug.r += 0.3;
        }
    }


    return back;
}




// unused due to overide
