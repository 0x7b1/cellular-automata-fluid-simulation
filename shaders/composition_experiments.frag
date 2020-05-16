#version 440 core
layout (location = 0) in vec2 uv;

uniform vec2 u_resolution;
uniform float u_time;
uniform float u_dt;
uniform vec2 u_mouse;
uniform float u_brush_size;

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

vec3 mod289(vec3 x) { return x - floor(x * (1.0 / 289.0)) * 289.0; }
vec2 mod289(vec2 x) { return x - floor(x * (1.0 / 289.0)) * 289.0; }
vec3 permute(vec3 x) { return mod289(((x*34.0)+1.0)*x); }

float random (in vec2 st) {
    return fract(sin(dot(st.xy,
    vec2(12.9898, 78.233)))
    * 43758.5453123);
}

float snoise(vec2 v) {
    const vec4 C = vec4(0.211324865405187, // (3.0-sqrt(3.0))/6.0
    0.366025403784439, // 0.5*(sqrt(3.0)-1.0)
    -0.577350269189626, // -1.0 + 2.0 * C.x
    0.024390243902439);// 1.0 / 41.0
    vec2 i  = floor(v + dot(v, C.yy));
    vec2 x0 = v -   i + dot(i, C.xx);
    vec2 i1;
    i1 = (x0.x > x0.y) ? vec2(1.0, 0.0) : vec2(0.0, 1.0);
    vec4 x12 = x0.xyxy + C.xxzz;
    x12.xy -= i1;
    i = mod289(i);// Avoid truncation effects in permutation
    vec3 p = permute(permute(i.y + vec3(0.0, i1.y, 1.0))
    + i.x + vec3(0.0, i1.x, 1.0));

    vec3 m = max(0.5 - vec3(dot(x0, x0), dot(x12.xy, x12.xy), dot(x12.zw, x12.zw)), 0.0);
    m = m*m;
    m = m*m;
    vec3 x = 2.0 * fract(p * C.www) - 1.0;
    vec3 h = abs(x) - 0.5;
    vec3 ox = floor(x + 0.5);
    vec3 a0 = x - ox;
    m *= 1.79284291400159 - 0.85373472095314 * (a0*a0 + h*h);
    vec3 g;
    g.x  = a0.x  * x0.x  + h.x  * x0.y;
    g.yz = a0.yz * x12.xz + h.yz * x12.yw;
    return 130.0 * dot(m, g);
}

float level(vec2 st) {
    float n = 0.0;
    for (float i = 1.0; i < 8.0; i ++) {
        float m = pow(2.0, i);
        n += snoise(st * m) * (1.0 / m);
    }
    return n * 0.5 + 0.5;
}


vec3 normal(vec2 st) {
    float d = 0.0001;
    float l0 = level(st);
    float l1 = level(st + vec2(d, 0.0));// slightly offset the x-coord
    float l2 = level(st + vec2(0.0, d));// slightly offset the y-coord
    // return normalized vector perpendicular to the surface using the noise values as the elevation of these points
    return normalize(vec3(-(l1 - l0), -(l2 - l0), d));
}

vec3 phong(vec2 st, vec3 normal, vec3 lightPos) {
    vec3 lightDir = normalize(vec3(lightPos - vec3(st, 0.0)));
    float diffuse = max(0.0, dot(normal, lightDir));
    vec3 vReflection = normalize(reflect(-lightDir, normal));
    float specular = pow(max(0.0, dot(normal, vReflection)), 8.0);
    vec3 ambientColor = vec3(0.183, 0.000, 0.365);
    vec3 diffuseColor = vec3(0.0, 0.5, 0.2);
    return min(vec3(1.0), ambientColor + diffuseColor * diffuse + specular);
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
    ivec2 xy = ivec2(int(uv.x * u_resolution.x), int(uv.y * u_resolution.y));
    int curr_coord = xy.x + xy.y * int(u_resolution.x);
    //    vec3 color = vec3(0.35, 0.48, 0.95);
    vec2 mouse_coord = vec2(u_mouse.x / u_resolution.x, 1 - u_mouse.y / u_resolution.y);

    vec2 st = uv;
    //    st.x *= u_resolution.x / u_resolution.y;
//    float t = u_time;
        float t = 1.0;
    vec3 color = phong(st, normal(st), vec3(cos(t) * 0.5 + 0.5, sin(t) * 0.5 + 0.5, 1.0));
    // water if the elevation is less than a threshold


    Cell cell = curr_gen[curr_coord];
    int cell_type = cell.element_type;

    //    vec2 st = st / u_resolution.st;
    //    vec3 color = vec3(0.0);

    if (cell_type == CELL_WATER) {
        //        color = vec3(0, 0, 1);
        color = vec3(0, 0, random(xy));

        //        color += vec3(0, 0, rand(st));
        //        FragColor = vec4(0, 0, 1-cell.mass, 1);
    } else if (cell_type == CELL_BLOCK) {
        //        float rnd = rand(xy);
        //        color += vec3(u_time * rnd, rnd, 0);
        color += vec3(1.0, 1.0, .0);
    } else if (cell_type == CELL_EMPTY) {
        //        st.x *= u_resolution.x / u_resolution.y;
        //                        color += vec3(st.x, st.y, abs(sin(u_time)));
        float rad = mix(0.03, 0.08, u_brush_size / 7);
        float d = min(1.0, ring(st - mouse_coord, rad, 0.001));
        d = smoothedge(d, 1.3);

        //    color = mix(vec3(0.0, 0.1, 0.2), vec3(1.0, 1.0, 0.6), d);
        color = mix(1-color, color, d);


//        color += vec3(1.0, 1.0, 0.6);
    }

    //    float n = level(st);
    //    if (n < 0.496) {color = vec3(0.0, 0.0, 0.2);}




    FragColor = vec4(color, 1.0);
}