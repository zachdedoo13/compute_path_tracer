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

#define MAXHIT Hit(10000.0, MDEF)

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

Hit map(vec3 pu0) {
    Hit u0 = MAXHIT;

    { // bounds 1
        Hit u1 = MAXHIT;
        vec3 pu1 = move(pu0, vec3(0.0));

        {  // bounds 2
            Hit u1s0 = Hit(
                sdCube(move(pu1, vec3(1.0)) * 1, vec3(1, 1, 1)) / 1,
                MDEF
            );
            u1 = opUnion(u1, u1s0);
        }

        u0 = opUnion(u0, u1);
    }

    { // bounds 3
        Hit u0s0 = Hit(
            sdSphere(move(pu0, vec3(0.0)) * 1, 0.5) / 1,
            MDEF
        );
        u0 = opUnion(u0, u0s0);
    }


    return u0;
}



// unused due to overide
