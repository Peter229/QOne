#version 450

layout(location = 0) in vec3 v_colour;
layout(location = 1) in vec2 v_uv;
layout(location = 2) in vec3 v_dir;

layout(location = 0) out vec4 f_color;

layout(set = 1, binding = 0) uniform texture2D t_diffuse;
layout(set = 1, binding = 1) uniform sampler s_diffuse;

layout(set = 2, binding = 0) uniform texture2D lt_diffuse;
layout(set = 2, binding = 1) uniform sampler ls_diffuse;

void main() {

    vec3 dir = v_dir;
    dir.z *= 3.0;
    dir = normalize(dir) * 6 * 63 / 128.0f;

    vec2 tex_coord_front = v_uv + vec2(dir.x, -dir.y);
    vec2 tex_coord_back = (v_uv / 2.0) + vec2(dir.x / 0.5, -dir.y);

    vec2 position = vec2(0.0, 0.0);
    vec2 size = vec2(0.5, 1.0);
    vec2 real_coord_front = position + size * fract(tex_coord_front);
    
    position = vec2(0.5, 0.0);
    vec2 real_coord_back = position + size * fract(tex_coord_back);

    vec3 front_colour = texture(sampler2D(t_diffuse, s_diffuse), real_coord_front).rgb;
    vec3 back_colour = texture(sampler2D(t_diffuse, s_diffuse), real_coord_back).rgb;
    vec3 colour = front_colour;
    if (front_colour.x + front_colour.y + front_colour.z < 0.01) {
        colour = back_colour;
    }
    f_color = vec4(colour, 1.0);
}