#version 440
layout(local_size_x = 8, local_size_y = 8) in;

#define CELL_EMPTY 0
#define CELL_BLOCK 1
#define CELL_WATER 2

#define DRAWING_ON 1

struct Cell {
    int type;
    float mass;
};

const float MAX_MASS = 10.0;
const float MIN_MASS = 0.0001;
const float MAX_COMPRESS = 0.02;
const float MIN_FLOW = 0.01;
const float FLOW_VARIANCE = 0.8;
const float MAX_SPEED = 1.0;

uniform vec2 u_resolution;//  Canvas size (width,height)
uniform float u_dt;
uniform float u_time;// Time in seconds since load
uniform float u_brush_size;
uniform int u_drawing;
uniform int u_drawing_type;
uniform vec2 u_mouse;// mouse position in screen pixels

layout(shared, binding = 0) readonly buffer InputData {
    Cell curr_gen[];
};

layout(shared, binding = 1) writeonly buffer OutputData {
    Cell next_gen[];
};

layout(shared, binding = 2) buffer TmpData {
    float mass_buffer[];
};

int toIndex(ivec2 pos) {
    return pos.x + pos.y * int(u_resolution.x);
}

float getStableState(float total_mass) {
    if (total_mass <= 1.0) {
        return 1.0;
    } else if (total_mass < 2.0 * MAX_MASS + MAX_COMPRESS) {
        return (MAX_MASS * MAX_MASS + total_mass * MAX_COMPRESS) / (MAX_MASS + MAX_COMPRESS);
    } else {
        return (total_mass + MAX_COMPRESS) / 2.0;
    }
}

float rand(vec2 co){
    return fract(sin(dot(co.xy, vec2(12.9898, 78.233))) * 43758.5453);
}

void main() {
    ivec2 xy_curr = ivec2(gl_GlobalInvocationID.xy);
    int xy = toIndex(xy_curr);
    int xy_above = toIndex(xy_curr + ivec2(0, 1));
    int xy_below = toIndex(xy_curr + ivec2(0, -1));
    int xy_right = toIndex(xy_curr + ivec2(1, 0));
    int xy_left = toIndex(xy_curr + ivec2(-1, 0));

    Cell curr = curr_gen[xy];
    Cell above = curr_gen[xy_above];
    Cell below = curr_gen[xy_below];
    Cell right = curr_gen[xy_right];
    Cell left = curr_gen[xy_left];

    if (u_drawing == DRAWING_ON) {
        Cell new_cell = Cell (
        u_drawing_type,
        0.0
        );

        if (u_drawing_type == CELL_WATER) {
            new_cell.mass = 1.0 * MAX_MASS;
        }

        int mouseX = int(u_mouse.x);
        int mouseY = int(u_resolution.y) - int(u_mouse.y);
        int radius = int(u_brush_size);

        for (int x = -radius; x < radius; x++) {
            int height = int(sqrt(radius * radius - x * x));
            for (int y = -height; y < height; y++) {
                int idx = toIndex(ivec2(mouseX, mouseY) + ivec2(x, y));
                next_gen[idx] = new_cell;
                mass_buffer[idx] = new_cell.mass;
            }
        }
    }

    if (curr.type == CELL_BLOCK) {
//        if (above.type == CELL_WATER) {
//            above.mass = above.mass - 1.0;
//            next_gen[xy_above] = above;
//            mass_buffer[xy_above] -= 1.0;
//        }

        next_gen[xy] = curr;
        mass_buffer[xy] = 0.0;
        return;

    }

    float flow = 0.0;
    float remaining_mass = curr.mass;

    if (remaining_mass > 0) {
        if (below.type != CELL_BLOCK) {
            flow = getStableState(remaining_mass + below.mass) - below.mass;
            if (flow > MIN_FLOW) {
                flow *= 0.8;
            }

            flow = clamp(flow, 0.0, min(remaining_mass, MAX_SPEED));

            mass_buffer[xy] -= flow;
            mass_buffer[xy_below] += flow;
            remaining_mass -= flow;
        }
    }

    if (remaining_mass > 0) {
        if (left.type != CELL_BLOCK) {
            flow = (curr.mass - left.mass) / 4.0;
            if (flow > MIN_FLOW) {
                flow *= 0.8;
            }

            flow = clamp(flow, 0.0, remaining_mass);

            mass_buffer[xy] -= flow;
            mass_buffer[xy_left] += flow;
            remaining_mass -= flow;
        }
    }

    if (remaining_mass > 0) {
        if (right.type != CELL_BLOCK) {
            flow = (curr.mass - right.mass) / 4.0;
            if (flow > MIN_FLOW) {
                flow *= 0.8;
            }

            flow = clamp(flow, 0.0, remaining_mass);

            mass_buffer[xy] -= flow;
            mass_buffer[xy_right] += flow;
            remaining_mass -= flow;
        }
    }

    if (curr.mass > MIN_MASS) {
        curr.type = CELL_WATER;
    } else {
        curr.type = CELL_EMPTY;
    }

    curr.mass = mass_buffer[xy];
    next_gen[xy] = curr;

    //    memoryBarrier();
    //    barrier();
}