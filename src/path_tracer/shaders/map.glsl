Hit map(vec3 pos) {
    Hit back;
    Hit[2] shapes;
    vec3 tr;

    tr = pos;
    tr = move(tr, vec3(-1.0, 0.0, 0.0));
    shapes[0] = Hit(
        sdSphere(tr, 1.0)
    );

    shapes[1] = Hit(sdSphere(pos - vec3(1.0, 0.0, 0.0), 1.0));


    back = Hit(10000.0);
    for (int i = 0; i < 2; i ++) {
        back = opUnion(back, shapes[i]);
    }

    return back;
}

// unused due to overide
