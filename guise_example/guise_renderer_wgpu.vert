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

layout(location = 0) in vec2 a_position;
layout(location = 1) in vec2 a_tex_coord;
layout(location = 2) in uint a_color;

layout(location = 0) out vec2 v_tex_coord;
layout(location = 1) out vec4 v_color;

void main() {
    v_tex_coord = a_tex_coord;
    v_color = vec4((a_color >> 24) & 0xff,
                   (a_color >> 16) & 0xff,
                   (a_color >> 8) & 0xff,
                   (a_color >> 0) & 0xff) / 255.0;
    gl_Position = u_matrix * vec4(a_position, 0, 1);
}
