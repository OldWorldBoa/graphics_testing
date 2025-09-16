#version 450

layout(binding = 0) uniform UniformBufferObject {
	mat4 view;
	mat4 proj;
} ubo;

layout(push_constant) uniform PushConstants {
	mat4 model;
} pcs;

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec3 in_colour;
layout(location = 2) in vec2 in_tex_coord;

layout(location = 0) out vec3 frag_colour;
layout(location = 1) out vec2 frag_tex_coord;

void main() {
	gl_Position = ubo.proj * ubo.view * pcs.model * vec4(in_position, 1.0);
	frag_colour = in_colour;
	frag_tex_coord = in_tex_coord;
}
