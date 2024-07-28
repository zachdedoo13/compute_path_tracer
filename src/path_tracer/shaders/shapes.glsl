float sdSphere(vec3 p, float s) {
    return length(p) - s;
}


// unions
Hit opUnion(Hit v1, Hit v2) {
    return v1.d < v2.d ? v1 : v2;
}