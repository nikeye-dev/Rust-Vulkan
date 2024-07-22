#version 450

layout(binding = 0) uniform Transformation {
    mat4 view;
    mat4 projection;
} vp;

layout(push_constant) uniform PushConstants {
    mat4 model;
} pcs;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec4 inColor;

layout(location = 0) out vec4 fragColor;

void main() {
    gl_Position = vp.projection * vp.view * pcs.model * vec4(inPosition, 1.0);
    fragColor = inColor;
}
