#version 450

layout(location = 0) in vec3 v_colour;
layout(location = 1) in vec2 v_uv;

layout(location = 0) out vec4 f_color;

layout(set = 1, binding = 0) uniform texture2D t_diffuse;
layout(set = 1, binding = 1) uniform sampler s_diffuse;

void main() {
    //f_color = vec4(v_colour, 1.0);
    f_color = texture(sampler2D(t_diffuse, s_diffuse), v_uv);
}