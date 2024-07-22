#version 450

layout(binding = 0) uniform Transformation {
    mat4 model;
    mat4 view;
    mat4 projection;
} MVP;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec4 inColor;

layout(location = 0) out vec4 fragColor;

void main() {
    gl_Position = MVP.projection * MVP.view * MVP.model * vec4(inPosition, 1.0);
    fragColor = inColor;
}
