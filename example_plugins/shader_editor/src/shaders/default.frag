#version 450
precision mediump float;

in vec4 f_color;

out vec4 out_color;
uniform mat4 extra;

void main() {
    out_color = f_color + extra[0];
}

