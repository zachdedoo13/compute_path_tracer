Hit map(vec3 pos) {
    Hit[2] shapes;
    vec3 tr;

    tr = pos;
    tr = move(tr, vec3(2.46, 0.35, 0));
    //rot
    shapes[0] = Hit(
    sdCube(tr * 1, vec3(1, 1, 1)) / 1,

    Mat(vec3(1, 1, 1), vec3(0, 0, 0), 0, vec3(0, 0, 0), 0)

    );

    tr = pos;
    //pos
    //rot
    shapes[1] = Hit(
    sdSphere(tr * 1, 1) / 1,

    Mat(vec3(1, 1, 1), vec3(0, 0, 0), 0, vec3(0, 0, 0), 0)

    );

    Hit back = shapes[0];
    for (int i = 1; i < 2; i ++) {
        back = opUnion(back, shapes[i]);
    }

    return back;
}

// unused due to overide
