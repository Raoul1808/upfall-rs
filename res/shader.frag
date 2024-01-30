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
    float dist = distance(uv * u_resolution, circle_pos);
    return step(dist, radius * 2.0);
}

void main() {
    vec2 uv = gl_FragCoord.xy / u_resolution;
    uv.y = 1.0 - uv.y;
    vec4 col = v_color * texture(u_texture, v_uv);
    float circ = circle(uv, u_circle_pos, u_circle_radius);
    float grayscale = circ > 0.5 ? 1.0 - col.b : col.r;
    o_color = grayscale > 0.5 ? vec4(u_color_b.rgb, 1.0) : vec4(u_color_a.rgb, 1.0);
}
