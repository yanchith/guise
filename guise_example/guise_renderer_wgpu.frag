#version 450

// TODO(yan): @Cleanup This shader is currently not used, both because wgpu is
// beginning to drop spirv-cross (and only supports
// Features::SPIRV_SHADER_PASSTHROUGH on a few platforms) AND we'd like to not
// use shaderc if possible AND naga is not stable enough for GLSL just yet,
// although the situation is improving.
//
// It can be re-used for the Vulkan backend, if it ever exists, or (with some
// minor tweaks) the GLes backend.

layout(set = 0, binding = 0) uniform Transform {
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
