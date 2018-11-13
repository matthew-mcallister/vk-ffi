// Evaluates the gamma special function at the points of a grid to form
// an image. The dimensions of the image are determined by the x and y
// components of the work group count vector.

#version 450

#pragma shader_stage(compute)

layout(constant_id = 0) const float MIN_X = 1.0f;
layout(constant_id = 1) const float MIN_Y = -2.0f;
layout(constant_id = 2) const float MAX_X = 5.0f;
layout(constant_id = 3) const float MAX_Y = 2.0f;

layout(set = 0, binding = 0) buffer _buf {
    vec2 image[];
};

const float PI = radians(180.0f);

const uint LANCZOS_G = 5;
const uint LANCZOS_N = 6;
const float LANCZOS_COEFFS[LANCZOS_N] = {
    76.18009172947146f,
    -86.50532032941677f,
    24.01409824083091f,
    -1.231739572450155f,
    0.1208650973866179e-2f,
    -0.5395239384953e-5f,
};

vec2 real(float x) { return vec2(x, 0.0f); }

vec2 cmul(vec2 z, vec2 w)
    { return vec2(z.x * w.x - z.y * w.y, z.x * w.y + z.y * w.x); }
float cmodsq(vec2 z) { return dot(z, z); }
float cmod(vec2 z) { return length(z); }
float carg(vec2 z) { return atan(z.y, z.x); }
vec2 cconj(vec2 z) { return vec2(z.x, -z.y); }
vec2 cinv(vec2 z) { return cconj(z) / cmodsq(z); }

vec2 cexp(vec2 z) { return exp(z.x) * vec2(cos(z.y), sin(z.y)); }
vec2 clog(vec2 z) { return vec2(log(cmod(z)), carg(z)); }
vec2 cpow(vec2 z, vec2 w) { return cexp(cmul(w, clog(z))); }

vec2 gamma(vec2 z) {
    z.x -= 1;
    vec2 x = real(1.000000000190015f);
    for (uint k = 0; k < LANCZOS_N; k++) {
        x += LANCZOS_COEFFS[k] * cinv(z + real(k + 1.0f));
    }
    vec2 p = z + real(0.5f);
    vec2 q = p + real(LANCZOS_G);
    vec2 c = cpow(q, p);
    vec2 d = cexp(-q);
    return sqrt(2.0f * PI) * cmul(c, cmul(d, x));
}

void main() {
    vec2 min_bounds = vec2(MIN_X, MIN_Y);
    vec2 max_bounds = vec2(MAX_X, MAX_Y);
    vec2 dimensions = max_bounds - min_bounds;

    uvec2 idx = gl_GlobalInvocationID.xy;
    uvec2 img_dims = gl_NumWorkGroups.xy;

    vec2 z =
        min_bounds + dimensions * vec2(idx) / vec2(img_dims - uvec2(1, 1));

    uint flat_idx = idx.y * img_dims.x + idx.x;
    image[flat_idx] = gamma(z);
}
