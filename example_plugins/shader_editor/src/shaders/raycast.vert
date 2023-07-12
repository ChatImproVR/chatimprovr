#version 450
out vec4 f_color;
out mat4 iv;
out vec3 cam_pos;

uniform mat4 view;
uniform mat4 proj;

// https://www.saschawillems.de/blog/2016/08/13/vulkan-tutorial-on-rendering-a-fullscreen-quad-without-buffers/
void main() {
    vec2 uv = vec2((gl_VertexID << 1) & 2, gl_VertexID & 2);
    f_color = vec4(uv, 0, 0);
    iv = inverse(proj * mat4(mat3(view)));
    cam_pos = (inverse(view) * vec4(0,0,0,1)).xyz;
    gl_Position = vec4(uv.xy * 2.0f + -1.0f, 0.0, 1.0f);
}

