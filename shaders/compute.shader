#version 440
layout(local_size_x = 8, local_size_y = 8) in;

#define CELL_EMPTY 0
#define CELL_BLOCK 1
#define CELL_WATER 2

#define DRAWING_OFF 0
#define DRAWING_ON 1

struct Cell {
    int element_type;
    float mass;
};

const float MAX_MASS = 10.0;
const float MIN_MASS = 0.0001;
const float MAX_COMPRESS = 0.02;
const float MIN_FLOW = 0.01;
const float FLOW_VARIANCE = 0.8;
const float MAX_SPEED = 1.0;

uniform vec2 u_field_size;
uniform float u_dt;
uniform float u_time;
uniform int u_drawing;
uniform int u_drawing_type;
uniform vec2 u_drawing_coords;

layout(shared, binding = 0) readonly buffer InputData {
    Cell curr_gen[];
//  Cell mass_values[width * height];
};

layout(shared, binding = 1) writeonly buffer OutputData {
    Cell next_gen[];
};

int toIndex(ivec2 pos) {
    return pos.x + pos.y * int(u_field_size.x);
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
    ivec2 curr_coord = ivec2(gl_GlobalInvocationID.xy);

    Cell above = curr_gen[toIndex(curr_coord + ivec2(0, 1))];
    Cell curr = curr_gen[toIndex(curr_coord)];
    Cell below = curr_gen[toIndex(curr_coord + ivec2(0, -1))];

    float flow = 0.0;
    float remaining_mass = 0.0;

    int curr_type = curr.element_type;
    float curr_mass = curr.mass;

    if (u_drawing == DRAWING_ON) {
        Cell new_cell = Cell (
        u_drawing_type,
        1.0
        );

        if (rand(curr_coord) <= 0.5) {
            new_cell.mass = 0.0;
        }


        int tmpX = int(u_drawing_coords.x);
        int tmpY = int(u_field_size.x) - int(u_drawing_coords.y);

        next_gen[toIndex(ivec2(tmpX, tmpY))] = new_cell;
        next_gen[toIndex(ivec2(tmpX, tmpY) + ivec2(1, 0))] = new_cell;
        next_gen[toIndex(ivec2(tmpX, tmpY) + ivec2(2, 0))] = new_cell;
        next_gen[toIndex(ivec2(tmpX, tmpY) + ivec2(3, 0))] = new_cell;
        next_gen[toIndex(ivec2(tmpX, tmpY) + ivec2(4, 0))] = new_cell;
        next_gen[toIndex(ivec2(tmpX, tmpY) + ivec2(0, 1))] = new_cell;
        next_gen[toIndex(ivec2(tmpX, tmpY) + ivec2(0, 2))] = new_cell;
        next_gen[toIndex(ivec2(tmpX, tmpY) + ivec2(0, 3))] = new_cell;
        next_gen[toIndex(ivec2(tmpX, tmpY) + ivec2(0, 4))] = new_cell;
        next_gen[toIndex(ivec2(tmpX, tmpY) + ivec2(1, 1))] = new_cell;
        next_gen[toIndex(ivec2(tmpX, tmpY) + ivec2(2, 2))] = new_cell;
        next_gen[toIndex(ivec2(tmpX, tmpY) + ivec2(3, 3))] = new_cell;
        next_gen[toIndex(ivec2(tmpX, tmpY) + ivec2(4, 4))] = new_cell;
        next_gen[toIndex(ivec2(tmpX, tmpY) + ivec2(1, -1))] = new_cell;
        next_gen[toIndex(ivec2(tmpX, tmpY) + ivec2(2, -2))] = new_cell;
        next_gen[toIndex(ivec2(tmpX, tmpY) + ivec2(3, -3))] = new_cell;
        next_gen[toIndex(ivec2(tmpX, tmpY) + ivec2(4, -4))] = new_cell;
        next_gen[toIndex(ivec2(tmpX, tmpY) + ivec2(-1, -1))] = new_cell;
        next_gen[toIndex(ivec2(tmpX, tmpY) + ivec2(-2, -2))] = new_cell;
        next_gen[toIndex(ivec2(tmpX, tmpY) + ivec2(-3, -3))] = new_cell;
        next_gen[toIndex(ivec2(tmpX, tmpY) + ivec2(-4, -4))] = new_cell;
    }

    next_gen[toIndex(curr_coord)] = curr;


    if (curr.element_type == CELL_BLOCK) {
        next_gen[toIndex(curr_coord)] = curr;
    } else if (curr.element_type == CELL_EMPTY) {
        if (above.element_type == CELL_WATER) {
            next_gen[toIndex(curr_coord)] = Cell (
            CELL_WATER,
            0.0
            );
        } else if (above.element_type == CELL_BLOCK) {
            next_gen[toIndex(curr_coord)] = Cell (
            CELL_EMPTY,
            0.0
            );
        }
    } else if (curr.element_type == CELL_WATER) {
        if (below.element_type == CELL_EMPTY) {
            next_gen[toIndex(curr_coord)] = Cell (
            CELL_EMPTY,
            0.0
            );
        } else if (below.element_type == CELL_BLOCK) {
            next_gen[toIndex(curr_coord)] = curr;
        }
    }

    /*
    flow = 0.0;
    remaining_mass = cell.mass;

    // Make the cell fall down
    if (remaining_mass > 0.0) {
        Cell block_down = curr_gen[toIndex(curr_coord + ivec2(0, -1))];
        // Perform the falling
        if (block_down.element_type != CELL_BLOCK) {
            flow = getStableState(remaining_mass + block_down.mass) - block_down.mass;
            if (flow > MIN_FLOW) {
                flow *= FLOW_VARIANCE;
            }

            flow = clamp(flow, 0.0, min(remaining_mass, MAX_SPEED));

            next_gen[toIndex(curr_coord)] = cell;

            next_gen[toIndex(curr_coord + ivec2(-1, 0))] = Cell (
            block_down.element_type,
            block_down.mass + flow
            );

            remaining_mass -= flow;
        }
    }

    float flow = 0.0;
    remaining_mass = cell.mass;

    if (remaining_mass <= 0.0) {
        next_gen[toIndex(curr_coord)] = cell;
        return;
    }

    // Down
    Cell block_down = curr_gen[toIndex(curr_coord + ivec2(0, -1))];
    if (block_down.element_type != CELL_BLOCK) {
        flow = getStableState(remaining_mass + block_down.mass) - block_down.mass;
        if (flow > MIN_FLOW) {
            flow *= FLOW_VARIANCE;
        }

        flow = clamp(flow, 0.0, remaining_mass);

        Cell tmp1 = Cell (
        block_down.element_type,
        block_down.mass - flow
        );

        next_gen[toIndex(curr_coord)] = Cell (
        block_down.element_type,
        block_down.mass - flow
        );

        next_gen[toIndex(curr_coord + ivec2(-1, 0))] = Cell (
        block_down.element_type,
        block_down.mass + flow
        );

        remaining_mass -= flow;
    }


    if (remaining_mass <= 0.0) {
        next_gen[toIndex(curr_coord)] = cell;
        return;
    }

    // Left
    Cell block_left = curr_gen[toIndex(curr_coord + ivec2(-1, 0))];
    if (block_left.element_type != CELL_BLOCK) {
        flow = (cell.mass - block_left.mass) / 4.0;
        if (flow > MIN_FLOW) {
            flow *= FLOW_VARIANCE;
        }
        flow = clamp(flow, 0.0, remaining_mass);

        next_gen[toIndex(curr_coord)] = Cell (
        block_left.element_type,
        block_left.mass - flow
        );

        next_gen[toIndex(curr_coord + ivec2(-1, 0))] = Cell (
        block_left.element_type,
        block_left.mass + flow
        );

        remaining_mass -= flow;
    }

    if (remaining_mass <= 0.0) {
        return;
    }

    // Right
    Cell block_right = curr_gen[toIndex(curr_coord + ivec2(1, 0))];
    if (block_right.element_type != CELL_BLOCK) {
        flow = (cell.mass - block_right.mass) / 4.0;
        if (flow > MIN_FLOW) {
            flow *= FLOW_VARIANCE;
        }
        flow = clamp(flow, 0.0, remaining_mass);

        next_gen[toIndex(curr_coord)] = Cell (
        block_right.element_type,
        block_right.mass - flow
        );

        next_gen[toIndex(curr_coord + ivec2(1, 0))] = Cell(
        block_right.element_type,
        block_right.mass + flow
        );

        remaining_mass -= flow;
    }

    if (remaining_mass <= 0.0) {
        return;
    }

    // Up
    Cell block_up = curr_gen[toIndex(curr_coord + ivec2(0, 1))];
    if (block_up.element_type != CELL_BLOCK) {
        flow = remaining_mass - getStableState(remaining_mass + block_up.mass);
        if (flow > MIN_FLOW) {
            flow *= FLOW_VARIANCE;
        }

        flow = clamp(flow, 0.0, min(remaining_mass, MAX_SPEED));

        next_gen[toIndex(curr_coord)] = Cell (
        block_up.element_type,
        block_up.mass - flow
        );

        next_gen[toIndex(curr_coord + ivec(0, 1))] = Cell (
        block_up.element_type,
        block_up.mass + flow
        );

        remaining_mass -= flow;
    }

    // Cell element_type placement
    if (cell.element_type != CELL_BLOCK) {
        if (cell.mass > MIN_MASS) {
            next_gen[toIndex(curr_coord)] = Cell (
            CELL_WATER,
            cell.mass
            );
        } else {
            next_gen[toIndex(curr_coord)] = Cell (
            CELL_EMPTY,
            cell.mass
            );
        }
    }
    */

    // TODO: Handle cells out of bounds
}