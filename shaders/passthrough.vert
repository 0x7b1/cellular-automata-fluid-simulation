#version 440 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec2 aUV;

layout (location = 0) out vec2 oUV;

void main() {
    gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
    oUV = aUV;
}