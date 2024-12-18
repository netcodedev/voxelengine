#version 460 core

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normals;
layout (location = 2) in vec3 color;

out vec3 Normal;
out vec3 Color;
out vec3 toLightVector;

uniform vec3 lightPosition;
uniform mat4 model;
uniform mat4 viewProjection;

void main()
{
    vec4 worldPosition = model * vec4(position, 1.0);
    gl_Position = viewProjection * worldPosition;
    Normal = normals;
    Color = color;
    toLightVector = lightPosition - worldPosition.xyz;
}