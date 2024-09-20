#version 330 core

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normals;
layout (location = 2) in vec2 texCoords;

out vec3 Normal;
out vec3 toLightVector;
out vec2 TexCoords;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

void main()
{
    vec4 worldPosition = model * vec4(position, 1.0);
    gl_Position = projection * view * worldPosition;
    Normal = normals;
    TexCoords = texCoords;
    toLightVector = vec3(100.0, 100.0, 100.0) - worldPosition.xyz;
}