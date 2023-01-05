#version 450

uniform mat4 view;
uniform mat4 proj;
uniform mat4 transf;
uniform mat4 extra;

layout (location = 0) in vec3 pos;
layout (location = 1) in vec3 uvw;

out vec4 f_color;

void main() {
    gl_Position = proj * view * transf * vec4(pos, 1.);
    vec4 extra_c = extra[0];
    vec3 color = mix(uvw, extra_c.xyz, extra_c.w);
    f_color = vec4(color, 1.);
}
