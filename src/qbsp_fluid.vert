#version 450

layout(location = 0) in vec3 a_position;
layout(location = 1) in vec3 a_colour;
layout(location = 2) in vec2 a_uv;
layout(location = 3) in vec2 a_luv;
layout(location = 4) in uvec4 a_light_style;
layout(location = 5) in vec2 a_extent;
layout(location = 6) in int a_light_id;

layout(location = 0) out vec3 v_colour;
layout(location = 1) out vec2 v_uv;

layout(set = 0, binding = 0)
uniform Uniforms {
    mat4 proj;
    mat4 view;
    mat4 model;
    float lights[12];
    vec3 eye;
    float time;
};

void main() {

    v_colour = a_colour;
    vec2 uv = a_uv;
    v_uv = a_uv + vec2(0.1 * sin(time + uv.y), 0.1 * sin(time + uv.x));
    gl_Position = proj * view * model * vec4(a_position, 1.0);
}