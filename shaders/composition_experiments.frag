#version 440 core
layout (location = 0) in vec2 uv;

uniform vec2 u_field_size;
uniform float u_time;
uniform float u_dt;
uniform vec2 u_mouse;

uniform vec4 blue_tint = vec4(.5);
uniform float scale_x = 0.67;



#define CELL_EMPTY 0
#define CELL_BLOCK 1
#define CELL_WATER 2

uniform int OCTAVES = 4;

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

float rand2(vec2 coord){
	return fract(sin(dot(coord, vec2(56, 78)) * 1000.0) * 1000.0);
}


float noise(vec2 coord){
    vec2 i = floor(coord);
	vec2 f = fract(coord);

	// 4 corners of a rectangle surrounding our point
	float a = rand(i);
	float b = rand(i + vec2(1.0, 0.0));
	float c = rand(i + vec2(0.0, 1.0));
	float d = rand(i + vec2(1.0, 1.0));

	vec2 cubic = f * f * (3.0 - 2.0 * f);

	return mix(a, b, cubic.x) + (c - a) * cubic.y * (1.0 - cubic.x) + (d - b) * cubic.x * cubic.y;
}

float fbm(vec2 coord){
	float value = 0.0;
	float scale = 0.5;

	for(int i = 0; i < OCTAVES; i++){
		value += noise(coord) * scale;
		coord *= 2.0;
		scale *= 0.5;
	}
	return value;
}

void main() {
    ivec2 xy = ivec2(int(uv.x * u_field_size.x), int(uv.y * u_field_size.y));
    int curr_coord = xy.x + xy.y * int(u_field_size.x);
    vec3 color = vec3(0.35, 0.48, 0.95);

    /*
    Cell cell = curr_gen[curr_coord];
    int cell_type = cell.element_type;

    vec2 st = xy / u_field_size.xy;

	vec2 coord = uv * 20.0;

	vec2 motion = vec2( fbm(coord + vec2(u_time * -0.5, u_time * 0.5)) );

	float final = fbm(coord + motion);

    if (cell_type == CELL_WATER) {
        //        FragColor = vec4(0, 0, 1, 1);
        color += vec3(0, 0, rand(xy));
        //        FragColor = vec4(0, 0, 1-cell.mass, 1);
    } else if (cell_type == CELL_BLOCK) {
        float rnd = rand(xy);
        color += vec3(rnd, rnd, 0);
    } else {
        //        st.x *= u_field_size.x / u_field_size.y;
        color += vec3(st.x, st.y, abs(sin(u_time)));
    }

    color -= final;
	FragColor = vec4(color, final * 0.5);
    */


    vec2 noisecoord1 = uv * u_field_size * scale_x;
	vec2 noisecoord2 = uv * u_field_size * scale_x + 4.0;

	vec2 motion1 = vec2(u_time * 0.3, u_time * -0.4);
	vec2 motion2 = vec2(u_time * 0.1, u_time * 0.5);

	vec2 distort1 = vec2(noise(noisecoord1 + motion1), noise(noisecoord2 + motion1)) - vec2(0.5);
	vec2 distort2 = vec2(noise(noisecoord1 + motion2), noise(noisecoord2 + motion2)) - vec2(0.5);

	vec2 distort_sum = (distort1 + distort2) / 60.0;

	vec4 color = vec4(1.0);

//	color = mix(color, blue_tint, 0.3);
//	color.rgb = mix(vec3(0.5), color.rgb, 1.4);

	float near_top = (uv.y + distort_sum.y) / (0.2 / u_field_size.y);
	near_top = clamp(near_top, 0.0, 1.0);
	near_top = 1.0 - near_top;

//	color = mix(color, vec4(1.0), near_top);

	float edge_lower = 0.6;
	float edge_upper = edge_lower + 0.1;

	if(near_top > edge_lower){
		color.a = 0.0;

		if(near_top < edge_upper){
			color.a = (edge_upper - near_top) / (edge_upper - edge_lower);
		}
	}

	FragColor = color;
}