#version 440 core
layout (location = 0) in vec2 uv;

uniform vec2 u_field_size;

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

void main() {
    ivec2 xy = ivec2(int(uv.x * u_field_size.x), int(uv.y * u_field_size.y));
    int curr_coord = xy.x + xy.y * int(u_field_size.x);

    Cell cell = curr_gen[curr_coord];
    int cell_type = cell.element_type;

    //    if (cell_type == CELL_EMPTY) {
    //        FragColor = vec4(0, 0, 0, 1);
    //    } else
    if (cell_type == CELL_WATER) {
        FragColor = vec4(0, 0, 1, 1);
    } else if (cell_type == CELL_BLOCK) {
        FragColor = vec4(0, 0, 0, 1);
    } else  {
        //        FragColor = vec4(1, 0, 0, 1);
        vec2 st = xy / u_field_size.xy;
        vec3 color = vec3(0.0);
        vec2 grid_st = st * 300.0;
        float res = 0.164;
        vec2 vc = fract(st * res);
        float tmp = 1.0 - (step(res, vc.x) * step(res, vc.y));
        color += vec3(0.2) * tmp;
        vec3 buff_sample = vec3(1, uv.xy);
        color += buff_sample;

        FragColor = vec4(color, 1.0);
    }
}