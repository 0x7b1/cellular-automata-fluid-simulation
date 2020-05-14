#version 440 core
layout (location = 0) in vec2 st;

uniform vec2 u_field_size;
uniform float u_time;
uniform float u_dt;
uniform vec2 u_mouse;
uniform float u_brush_size;


#define CELL_EMPTY 0
#define CELL_BLOCK 1
#define CELL_WATER 2

struct Cell {
    int element_type;
    float mass;
};

layout(shared, binding = 0) readonly buffer OutputData {
    Cell curr_gen[];
};

out vec4 FragColor;


float rand(vec2 co){
    return fract(sin(dot(co.xy, vec2(12.9898, 78.233))) * 43758.5453);
}

float distanceToSegment(vec2 a, vec2 b, vec2 p) {
    vec2 pa = p - a, ba = b - a;
    float h = clamp(dot(pa, ba)/dot(ba, ba), 0.0, 1.0);
    return length(pa - ba*h);
}


float smoothedge(float v, float f) {
    return smoothstep(0.0, f / u_field_size.x, v);
}


float circle(vec2 p, float radius) {
    return length(p) - radius;
}

float ring(vec2 p, float radius, float width) {
    return abs(length(p) - radius * 0.5) - width;
}

void main() {
    ivec2 xy = ivec2(int(st.x * u_field_size.x), int(st.y * u_field_size.y));
    int curr_coord = xy.x + xy.y * int(u_field_size.x);

    vec2 mouse_coord = vec2(u_mouse.x / u_field_size.x, 1 - u_mouse.y / u_field_size.y);

    Cell cell = curr_gen[curr_coord];
    int cell_type = cell.element_type;

    //    vec2 st = st / u_field_size.st;
    vec3 color = vec3(0.0);

    if (cell_type == CELL_WATER) {
        //        FragColor = vec4(0, 0, 1, 1);
        color += vec3(0, 0, rand(st));
        //        FragColor = vec4(0, 0, 1-cell.mass, 1);
    } else if (cell_type == CELL_BLOCK) {
        float rnd = rand(xy);
        color += vec3(u_time * rnd, rnd, 0);
    } else {
        //        st.x *= u_field_size.x / u_field_size.y;
                color += vec3(st.x, st.y, abs(sin(u_time)));
//        color += vec3(1.0, 1.0, 0.6);
    }

    //    vec2 dist = u_mouse/u_field_size - st.st;
    //    float mouse_pct = length(dist);
    //    mouse_pct = step(0.1, mouse_pct);
    //    vec3 m_color = vec3(mouse_pct);
    //    color += m_color;

    //    FragColor = vec4(color, 1.0);

    //    vec2 p = st / u_field_size.xx;
    //    vec4 m = vec4(mouse_coord.xy, mouse_coord.xy + vec2(u_brush_size)) / u_field_size.xxxx;
    //
    //    if (m.z > 0.0) {
    //        float d = distanceToSegment(m.xy, m.zw, p);
    //        color = mix(color, vec3(1.0, 1.0, 0.0), 1.0 - smoothstep(.004, 0.008, d));
    //    }
    //
    //    color = mix(color, vec3(1.0, 0.0, 0.0), 1.0 - smoothstep(0.003, 0.03, length(p - m.xy)));

    // MOUSE RING
    // float d = circle(st - vec2(0.2), 0.01);
    float rad = mix(0.03, 0.08, u_brush_size / 7);
    float d = min(1.0, ring(st - mouse_coord, rad, 0.001));
    d = smoothedge(d, 1.3);

//    color = mix(vec3(0.0, 0.1, 0.2), vec3(1.0, 1.0, 0.6), d);
        color = mix(1-color, color, d);

    FragColor = vec4(color, 1.0);
}
