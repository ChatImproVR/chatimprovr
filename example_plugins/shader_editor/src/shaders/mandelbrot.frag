#version 450
precision mediump float;

in vec4 f_color;

out vec4 out_color;
uniform mat4 extra;

const float pi = 3.141592;
const float tau = 2. * pi;

vec3 pixel(vec2 coord) {
    vec2 st = coord * 2. - 1.;

    st *= pow(2., extra[3].z);
    //st -= vec2(0.330,-0.100) * 3.;
    st -= extra[3].xy / 50.;

    const float scale = pow(0.080, 2.);
    st *= scale;
 	st -= vec2((-0.413), -0.216);
    
    
    float a = st.x;
    float b = st.y;
    int finish = 0;
    const int steps = 200;
    for (int i = 0; i < steps; i++) {
        float tmp = a * a - b * b + st.x;
        //float tmp = b * b - a * a + st.x;
        b = 2. * a * b + st.y;
        a = tmp;
        if (a > 9e9 || b > 9e9) {
			finish = i;
            break;
        }
    }
    if (finish == 0) return vec3(0);
    
    float g = float(finish) / float(steps);
    g = pow(g, .5);
    g = cos(g * tau * 3.);
    
    vec3 color = mix(
        mix(extra[0].rgb, extra[1].rgb, g),
        mix(extra[1].rgb, extra[2].rgb, g),
        g
    );


    return color;
}

void main() {
    out_color = vec4(pixel(f_color.xy), 1);
}
