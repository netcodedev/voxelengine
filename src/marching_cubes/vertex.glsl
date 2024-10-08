#version 460 core

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normals;
layout (location = 2) in vec3 color;

out vec3 Normal;
out vec3 Color;
out vec3 toLightVector;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

void main()
{
    vec4 worldPosition = model * vec4(position, 1.0);
    gl_Position = projection * view * worldPosition;
    Normal = normals;
    Color = color;
    toLightVector = vec3(0.0, 2000.0, 0.0) - worldPosition.xyz;
}