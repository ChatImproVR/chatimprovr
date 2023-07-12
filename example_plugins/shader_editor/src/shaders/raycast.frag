#version 450
precision mediump float;

uniform mat4 view;
uniform mat4 proj;

//uniform mat4 transf;
//uniform mat4 extra;

in vec4 f_color;
in mat4 iv;
in vec3 cam_pos;

out vec4 out_color;

// Ray-sphere intersection test:
// returns -1 if no hit,
// positive ray parameter t otherwise.
float sphere(vec3 ray, vec3 pos, float r) {
    float proj = dot(ray, pos);
    float disc = proj*proj - dot(pos, pos) + r*r;
    if (disc >= 0.) {
    	return proj - sqrt(disc);    
    } else {
        return -1.;
    }   
}

void main() {
    vec2 uv = f_color.xy;
    vec3 screenspace_ray = vec3(uv * 2. - 1., 0.2);
    vec3 ray = normalize((iv * vec4(screenspace_ray, 1)).xyz);

    vec3 sp_pos = vec3(1,0.3,0);
    float hit = sphere(ray, sp_pos - cam_pos,1.);
    vec3 color = vec3(sp_pos - (ray * hit + cam_pos));

    vec3 wc = hit * ray;
    if (hit < 0.) {
        color = vec3(ray)/10.;
        gl_FragDepth = 0.999999;
    } else {
        vec3 wc = hit * ray + cam_pos;
        vec4 k = (proj * view * vec4(wc,1));
        gl_FragDepth = (k.z/k.w + 1.) / 2.;
    }

    out_color = vec4(color,1);
}

