#version 450

layout(location = 0) in vec3 v_colour;
layout(location = 1) in vec2 v_uv;
layout(location = 2) in vec2 v_luv;
layout(location = 3) in flat vec2 v_extent;
layout(location = 4) in flat int v_light_id;
layout(location = 5) in vec4 v_light_style;

layout(location = 0) out vec4 f_color;

layout(set = 1, binding = 0) uniform texture2D t_diffuse;
layout(set = 1, binding = 1) uniform sampler s_diffuse;

layout(set = 2, binding = 0) uniform texture2D lt_diffuse;
layout(set = 2, binding = 1) uniform sampler ls_diffuse;

void main() {
    //f_color = vec4(v_colour, 1.0);

    vec3 light_map_value = vec3(0.0, 0.0, 0.0);

    if (v_light_id >= 0) {

        ivec2 texture_size = textureSize(sampler2D(lt_diffuse, ls_diffuse), 0);

        float s = v_luv.x;
        int s_left = int(s);
        int s_right = s_left + 1;
        float s_lerp = s - s_left;

        float t = v_luv.y;
        int t_top = int(t);
        int t_bot = t_top + 1;
        float t_lerp = t - t_top;

        int w = int(v_extent.x);

        int top_left_id = v_light_id + s_left + t_top * w;
        int x = top_left_id % texture_size.x;
        int y = top_left_id / texture_size.x;
        vec3 top_left = texelFetch(sampler2D(lt_diffuse, ls_diffuse), ivec2(x, y), 0).rgb;

        int top_right_id = v_light_id + s_right + t_top * w;
        x = top_right_id % texture_size.x;
        y = top_right_id / texture_size.x;
        vec3 top_right = texelFetch(sampler2D(lt_diffuse, ls_diffuse), ivec2(x, y), 0).rgb;

        vec3 top = mix(top_left, top_right, s_lerp);

        int bottom_left_id = v_light_id + s_left + t_bot * w;
        x = bottom_left_id % texture_size.x;
        y = bottom_left_id / texture_size.x;
        vec3 bottom_left = texelFetch(sampler2D(lt_diffuse, ls_diffuse), ivec2(x, y), 0).rgb;

        int bottom_right_id = v_light_id + s_right + t_bot * w;
        x = bottom_right_id % texture_size.x;
        y = bottom_right_id / texture_size.x;
        vec3 bottom_right = texelFetch(sampler2D(lt_diffuse, ls_diffuse), ivec2(x, y), 0).rgb;

        vec3 bottom = mix(bottom_left, bottom_right, s_lerp);

        light_map_value = mix(top, bottom, t_lerp);
    }

    light_map_value = light_map_value;

    float light_value = v_light_style.x + v_light_style.y + v_light_style.z + v_light_style.w;

    vec4 tex = texture(sampler2D(t_diffuse, s_diffuse), v_uv);

    float brighten = 20.0;

    f_color = tex * (vec4(light_map_value, 1.0)) * vec4(light_value, light_value, light_value, 1.0) * vec4(brighten, brighten, brighten, 1.0);
}