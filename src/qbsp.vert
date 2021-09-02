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
layout(location = 2) out vec2 v_luv;
layout(location = 3) out flat vec2 v_extent;
layout(location = 4) out flat int v_light_id;
layout(location = 5) out vec4 v_light_style;

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
    v_uv = a_uv;
    v_luv = a_luv;
    v_extent = a_extent;
    v_light_id = a_light_id;
    v_light_style = vec4(0.0, 0.0, 0.0, 0.0);
    if (a_light_style.x < 12) {
        v_light_style.x = lights[a_light_style.x];
    }
    if (a_light_style.y < 12) {
        v_light_style.y = lights[a_light_style.y];
    }
    if (a_light_style.z < 12) {
        v_light_style.z = lights[a_light_style.z];
    }
    if (a_light_style.w < 12) {
        v_light_style.w = lights[a_light_style.w];
    }

    gl_Position = proj * view * model * vec4(a_position, 1.0);
}