#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 tex_coord;

layout(location = 0) out vec2 frag_tex_coord;

layout(set = 0, binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view;
    mat4 proj;
} ubo;

void main() {
    gl_Position = ubo.proj * ubo.view * ubo.model * vec4(position, 0.0, 1.0);
    frag_tex_coord = tex_coord;
} 