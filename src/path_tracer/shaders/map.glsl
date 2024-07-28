Hit map(vec3 p) {
    Hit back;
    Hit[2] shapes;

    shapes[0] = Hit(sdSphere(p - vec3(-1.0, 0.0, 0.0), 1.0));

    shapes[1] = Hit(sdSphere(p - vec3(1.0, 0.0, 0.0), 1.0));


    back = Hit(10000.0);
    for (int i = 0; i < 2; i ++) {
        back = opUnion(back, shapes[i]);
    }

    return back;
}