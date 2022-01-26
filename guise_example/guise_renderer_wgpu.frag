#version 450

layout(set = 0, binding = 0) uniform TransformUniforms {
    mat4 u_matrix;
};

layout(set = 1, binding = 0) uniform texture2D u_texture;
layout(set = 1, binding = 1) uniform sampler u_sampler;

layout(location = 0) in vec2 v_tex_coord;
layout(location = 1) in vec4 v_color;

layout(location = 0) out vec4 f_color;

void main() {
    f_color = v_color * texture(sampler2D(u_texture, u_sampler), v_tex_coord);
}
