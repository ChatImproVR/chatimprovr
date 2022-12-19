#version 450

uniform mat4 view;
uniform mat4 proj;

layout (location = 0) in vec3 pos;
layout (location = 1) in vec3 color;
layout (location = 2) in mat4 transform;

out vec4 f_color;

void main() {
    gl_Position = proj * view * transform * vec4(pos, 1.0);
    f_color = vec4(color, 1.);
}


