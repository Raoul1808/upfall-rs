#version 150

in vec2 v_uv;
in vec4 v_color;

uniform sampler2D u_texture;
uniform int u_flip;

out vec4 o_color;

void main() {
    vec4 col = v_color * texture(u_texture, v_uv);
    o_color = u_flip == 1 ? vec4(1.0) - vec4(col.bbb, 0.0) : vec4(col.rrra);
}
