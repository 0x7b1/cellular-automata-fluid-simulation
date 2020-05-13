#version 440 core
layout (location = 0) in vec2 uv;

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

void main() {
    ivec2 xy = ivec2(int(uv.x * u_field_size.x), int(uv.y * u_field_size.y));
    int curr_coord = xy.x + xy.y * int(u_field_size.x);

    Cell cell = curr_gen[curr_coord];
    int cell_type = cell.element_type;

    vec2 st = xy / u_field_size.xy;
    vec3 color = vec3(0.0);

    if (cell_type == CELL_WATER) {
        //        FragColor = vec4(0, 0, 1, 1);
        color += vec3(0, 0, rand(xy));
        //        FragColor = vec4(0, 0, 1-cell.mass, 1);
    } else if (cell_type == CELL_BLOCK) {
        float rnd = rand(xy);
        color += vec3(u_time * rnd, rnd, 0);
    } else {
        //        st.x *= u_field_size.x / u_field_size.y;
        color += vec3(st.x, st.y, abs(sin(u_time)));
    }

    //    vec2 dist = u_mouse/u_field_size - st.xy;
    //    float mouse_pct = length(dist);
    //    mouse_pct = step(0.1, mouse_pct);
    //    vec3 m_color = vec3(mouse_pct);
    //    color += m_color;

    //    FragColor = vec4(color, 1.0);

    vec2 p = xy / u_field_size.xx;
    vec4 m = vec4(u_mouse.xy, u_mouse.xy + vec2(u_brush_size)) / u_field_size.xxxx;

    if (m.z > 0.0)
    {
        float d = distanceToSegment(m.xy, m.zw, p);
        color = mix(color, vec3(1.0, 1.0, 0.0), 1.0 - smoothstep(.004, 0.008, d));
    }

    color = mix(color, vec3(1.0, 0.0, 0.0), 1.0 - smoothstep(0.003, 0.03, length(p - m.xy)));

    FragColor = vec4(color, 1.0);
}