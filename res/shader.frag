#version 150

in vec2 v_uv;
in vec4 v_color;

uniform sampler2D u_texture;
uniform vec2 u_resolution;
uniform float u_circle_radius;
uniform vec2 u_circle_pos;
uniform int u_flip;
uniform vec4 u_color_a;
uniform vec4 u_color_b;
uniform vec2 u_circle_offset;
uniform float u_scale_factor;
uniform vec2 u_scale_offset;

out vec4 o_color;

float circle(vec2 uv, vec2 circle_pos, float radius) {
    const float SMOOTHNESS = 0.5;
    float dist = distance(uv * u_resolution, circle_pos);
    float rad = radius * 2.0;
    return 1.0 - smoothstep(rad - SMOOTHNESS, rad + SMOOTHNESS, dist);
}

void main() {
    vec2 uv = gl_FragCoord.xy / u_resolution / u_scale_factor;
    uv.y = 1.0 - uv.y;
    uv += u_circle_offset / u_resolution + vec2(-u_scale_offset.x, u_scale_offset.y) / u_resolution / u_scale_factor;
    vec4 col = v_color * texture(u_texture, v_uv);
    float circ = u_circle_radius == 0.0 ? 0.0 : circle(uv, u_circle_pos, u_circle_radius);
    float grayscale = mix(col.r, 1.0 - col.b, circ);
    o_color = mix(vec4(u_color_a.rgb, 1.0), vec4(u_color_b.rgb, 1.0), grayscale);
}
