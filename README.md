# Engine

This is a personal project to get a better understanding of rust and OpenGL. The goal is to create a 3D engine that can be used to create
applications of various types.

## Features

- Chunked multithreaded terrain generation
- Voxel Greedy meshing
- Text rendering
- Line rendering
- Rendering of 3D models
- Model animation
- Animation blending
- Root Motion
- Basic UI
  - Movable Panel
  - Button
  - Text
- Voxel terrain generation
- Marching cubes terrain generation
- Dual Contouring terrain generation (WIP)

## Planned Features

- Inverse Kinematics
- Infinite world generation
- Lighting
- Shadows
- Water
- Particles
- Physics

## Building

The engine requires the following packages on Linux:

- libxrandr-dev
- libxinerama-dev
- libxcursor-dev
- libxi-dev
- libassimp5

### Install dependencies

#### Ubuntu

```shell
apt install -y libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev libassimp5
```
