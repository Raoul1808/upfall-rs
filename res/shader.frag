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

out vec4 o_color;

float circle(vec2 uv, vec2 circle_pos, float radius) {
    const float SMOOTHNESS = 1.0;
    float dist = distance(uv * u_resolution, circle_pos);
    float rad = radius * 2.0;
    return 1.0 - smoothstep(rad - SMOOTHNESS, rad + SMOOTHNESS, dist);
}

void main() {
    vec2 uv = gl_FragCoord.xy / u_resolution;
    uv.y = 1.0 - uv.y;
    vec4 col = v_color * texture(u_texture, v_uv);
    float circ = u_circle_radius == 0.0 ? 0.0 : circle(uv, u_circle_pos, u_circle_radius);
    float grayscale = mix(col.r, 1.0 - col.b, circ);
    o_color = mix(vec4(u_color_a.rgb, 1.0), vec4(u_color_b.rgb, 1.0), grayscale);
}
