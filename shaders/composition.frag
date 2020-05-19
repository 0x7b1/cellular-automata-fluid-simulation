#version 440 core
layout (location = 0) in vec2 st;

uniform vec2 u_resolution;
uniform float u_time;
uniform float u_dt;
uniform vec2 u_mouse;
uniform float u_brush_size;

#define CELL_EMPTY 0
#define CELL_BLOCK 1
#define CELL_WATER 2
#define CELL_ACID 3
#define CELL_SAND 4

struct Cell {
    int type;
    float mass;
};

layout(shared, binding = 0) readonly buffer OutputData {
    Cell curr_gen[];
};

layout(shared, binding = 2) buffer TmpData {
    float tmp_data[];
};

out vec4 FragColor;

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

float rand(vec2 co){
    return fract(sin(dot(co.xy, vec2(12.9898, 78.233))) * 43758.5453);
}

float distanceToSegment(vec2 a, vec2 b, vec2 p) {
    vec2 pa = p - a, ba = b - a;
    float h = clamp(dot(pa, ba)/dot(ba, ba), 0.0, 1.0);
    return length(pa - ba*h);
}


float smoothedge(float v, float f) {
    return smoothstep(0.0, f / u_resolution.x, v);
}


float circle(vec2 p, float radius) {
    return length(p) - radius;
}

float ring(vec2 p, float radius, float width) {
    return abs(length(p) - radius * 0.5) - width;
}

void main() {
    ivec2 xy = ivec2(int(st.x * u_resolution.x), int(st.y * u_resolution.y));
    int curr_coord = xy.x + xy.y * int(u_resolution.x);

    vec2 mouse_coord = vec2(u_mouse.x / u_resolution.x, 1 - u_mouse.y / u_resolution.y);

    Cell cell = curr_gen[curr_coord];
    int cell_type = cell.type;

    //    vec2 st = st / u_resolution.st;
    vec3 color = vec3(0.0);

    if (cell_type == 99) {
        FragColor = vec4(1.0, 0.0, 0.0, 1.0);
        return;
    } else if (cell_type == CELL_ACID) {
        color = vec3(0, rand(xy), 0);
    } else if (cell_type == CELL_SAND)  {
        color = vec3(1, 1, 0);
    } else if (cell_type == CELL_WATER) {
        color += hsv2rgb(vec3(0.61, 1.0, mix(0.7, 1.0, cell.mass)));
    } else if (cell_type == CELL_BLOCK) {
        color +=hsv2rgb(vec3(0.075, 0.6, mix(rand(xy), 0.46, 0.77)));
    } else {
        color +=hsv2rgb(vec3(0, 0.0, clamp(rand(xy), 0.0, 0.15)));
    }

    // MOUSE RING
    float rad = mix(0.03, 0.08, u_brush_size / 7);
    float d = min(1.0, ring(st - mouse_coord, rad, 0.001));
    d = smoothedge(d, 1.1);

    color = mix(1-color, color, d);

    FragColor = vec4(color, 1.0);
}