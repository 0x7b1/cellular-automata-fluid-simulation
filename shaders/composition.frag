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

    if (cell_type == CELL_WATER) {
        //        FragColor = vec4(0, 0, 1, 1);
//        color += vec3(0, 0, 1.0);
//        color += vec3(rand(xy), 0, 0);
        color += vec3(0, 0, rand(xy));
//        color += vec3(0, 0, 1);
        //        FragColor = vec4(0, 0, 1-cell.mass, 1);
    } else if (cell_type == CELL_BLOCK) {
        float rnd = rand(xy);
        color += vec3(u_time * rnd, rnd, 0);
    } else {
        //        st.x *= u_resolution.x / u_resolution.y;
        color += vec3(st.x, st.y, abs(sin(u_time)));
        //        color += vec3(1.0, 1.0, 0.6);
    }

    //    vec2 dist = u_mouse/u_resolution - st.st;
    //    float mouse_pct = length(dist);
    //    mouse_pct = step(0.1, mouse_pct);
    //    vec3 m_color = vec3(mouse_pct);
    //    color += m_color;

    //    FragColor = vec4(color, 1.0);

    //    vec2 p = st / u_resolution.xx;
    //    vec4 m = vec4(mouse_coord.xy, mouse_coord.xy + vec2(u_brush_size)) / u_resolution.xxxx;
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

//    if (tmp_data[curr_coord] > 0.0) {
//        FragColor = vec4(1, 0, 0, 1.0);
//    } else {
//        FragColor = vec4(0, 1, 0, 1.0);
//    }

        FragColor = vec4(color, 1.0);
//        FragColor  = vec4(sin(vec3(tmp_data[curr_coord])), 1.0);
}

/*
https://thebookofshaders.com/edit.php?log=160909064723
https://thebookofshaders.com/edit.php?log=160909064528
https://thebookofshaders.com/edit.php?log=161127202429
*/