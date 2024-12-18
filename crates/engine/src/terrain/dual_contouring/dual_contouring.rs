use core::panic;

use cgmath::{Matrix4, Point3, Vector3};
use gl::types::GLuint;
use glfw::{Glfw, MouseButton, WindowEvent};
use libnoise::prelude::*;

use crate::{
    core::{
        entity::{component::Component, Entity},
        renderer::{line::Line, shader::VertexAttributes, texture::Texture},
        scene::Scene,
    },
    terrain::{Chunk, ChunkBounds, Terrain, CHUNK_SIZE, CHUNK_SIZE_FLOAT, USE_LOD},
};

use fast_surface_nets::{
    ndshape::{AbstractShape, RuntimeShape},
    {surface_nets, SurfaceNetsBuffer},
};

use super::{ChunkMesh, DualContouringChunk, Vertex};

impl DualContouringChunk {
    fn get_density_at(&self, (x, y, z): (usize, usize, usize)) -> f32 {
        let offset: f64 = 16777216.0;
        let sample_point = (
            (self.position.0 * CHUNK_SIZE_FLOAT) as f64 + x as f64 + offset,
            (self.position.1 * CHUNK_SIZE_FLOAT) as f64 + y as f64 + offset,
            (self.position.2 * CHUNK_SIZE_FLOAT) as f64 + z as f64 + offset,
        );

        let noise = ((1.0 + self.noise.sample([sample_point.0, sample_point.2])) / 2.0) as f32;
        let _iso = ((1.0
            + self
                .cave
                .sample([sample_point.0, sample_point.1, sample_point.2]))
            / 2.0) as f32;
        let height_iso = 1.0 - ((noise) / ((1.0 + y as f32) / CHUNK_SIZE_FLOAT));
        height_iso
    }

    fn generate_mesh(&self) -> ChunkMesh<Vertex> {
        let mut vertices = Vec::<Vertex>::new();
        let mut indices = Vec::<u32>::new();
        let size = (self.chunk_size + 2) as u32;
        let scale_factor = CHUNK_SIZE / self.chunk_size;
        let shape = RuntimeShape::<u32, 3>::new([size, size, size]);
        let mut sdf = vec![0.0; (size * size * size) as usize];
        for i in 0..sdf.len() {
            let [x, y, z] = shape.delinearize(i as u32);
            sdf[i as usize] = self.get_density_at((
                x as usize * scale_factor,
                y as usize * scale_factor,
                z as usize * scale_factor,
            ));
        }
        let mut buffer = SurfaceNetsBuffer::default();
        surface_nets(&sdf, &shape, [0; 3], [size as u32 - 1; 3], &mut buffer);
        for (i, vertex) in buffer.positions.into_iter().enumerate() {
            let normal = buffer.normals[i];
            vertices.push(Vertex {
                position: [
                    vertex[0] * scale_factor as f32,
                    vertex[1] * scale_factor as f32,
                    vertex[2] * scale_factor as f32,
                ],
                normal,
                color: [0.0, 0.5, 0.1],
            });
        }
        for index in buffer.indices {
            indices.push(index);
        }
        ChunkMesh::new(vertices, Some(indices))
    }

    fn calculate_chunk_size(lod: usize) -> usize {
        let lod = std::cmp::max(
            8,
            std::cmp::min(
                CHUNK_SIZE,
                CHUNK_SIZE / 2usize.pow(if lod > 0 { (lod - 1) as u32 } else { 0 }),
            ),
        );
        if USE_LOD {
            lod
        } else {
            CHUNK_SIZE
        }
    }
}

impl Chunk for DualContouringChunk {
    fn new(seed: u64, position: (f32, f32, f32), lod: usize) -> Self {
        let noise = Source::perlin(seed).scale([0.003; 2]).fbm(6, 1.0, 2.0, 0.5);
        let cave = Source::perlin(seed).scale([0.1; 3]);
        let mut chunk = Self {
            position,
            cave,
            noise,
            chunk_size: DualContouringChunk::calculate_chunk_size(lod),
            mesh: None,
        };
        chunk.mesh = Some(chunk.generate_mesh());
        chunk
    }

    fn buffer_data(&mut self) {
        if let Some(mesh) = &mut self.mesh {
            mesh.buffer_data();
        }
    }

    fn get_bounds(&self) -> ChunkBounds {
        ChunkBounds {
            min: (
                (self.position.0 * CHUNK_SIZE as f32) as i32,
                (self.position.1 * CHUNK_SIZE as f32) as i32,
                (self.position.2 * CHUNK_SIZE as f32) as i32,
            ),
            max: (
                ((self.position.0 + 1.0) * CHUNK_SIZE as f32) as i32,
                ((self.position.1 + 1.0) * CHUNK_SIZE as f32) as i32,
                ((self.position.2 + 1.0) * CHUNK_SIZE as f32) as i32,
            ),
        }
    }

    fn process_line(&mut self, _: &Line, _: &MouseButton) -> bool {
        false
    }

    fn get_position(&self) -> Point3<f32> {
        Point3::new(
            self.position.0 * CHUNK_SIZE_FLOAT,
            self.position.1 * CHUNK_SIZE_FLOAT,
            self.position.2 * CHUNK_SIZE_FLOAT,
        )
    }

    fn get_shader_source() -> (String, String) {
        (
            include_str!("vertex.glsl").to_string(),
            include_str!("fragment.glsl").to_string(),
        )
    }

    fn get_textures() -> Vec<Texture> {
        Vec::new()
    }

    fn get_triangle_count(&self) -> usize {
        if let Some(mesh) = &self.mesh {
            mesh.get_triangle_count()
        } else {
            0
        }
    }

    fn get_vertices(&self) -> Vec<[f32; 3]> {
        if let Some(mesh) = &self.mesh {
            mesh.vertices
                .iter()
                .map(|v| [v.position[0], v.position[1], v.position[2]])
                .collect()
        } else {
            Vec::new()
        }
    }

    fn get_indices(&self) -> Vec<[u32; 3]> {
        if let Some(mesh) = &self.mesh {
            if let Some(indices) = &mesh.indices {
                return indices.chunks(3).map(|c| [c[0], c[1], c[2]]).collect();
            }
        }
        Vec::new()
    }
}

impl Component for DualContouringChunk {
    fn update(&mut self, _: &mut Scene, _: &mut Entity, _: f64) {}

    fn render(
        &self,
        scene: &Scene,
        _: &Entity,
        view_projection: &Matrix4<f32>,
        parent_transform: &Matrix4<f32>,
    ) {
        if let Some(terrain) = scene.get_component::<Terrain<DualContouringChunk>>() {
            let shader = terrain.get_shader();
            if let Some(mesh) = &self.mesh {
                if !mesh.is_buffered() {
                    panic!("Mesh is not buffered");
                }
                shader.bind();
                shader.set_uniform_mat4("viewProjection", &view_projection);
                unsafe {
                    gl::Enable(gl::CULL_FACE);
                }
                mesh.render(
                    &shader,
                    &(parent_transform
                        * Matrix4::from_translation(Vector3::new(
                            self.position.0 * CHUNK_SIZE_FLOAT,
                            self.position.1 * CHUNK_SIZE_FLOAT,
                            self.position.2 * CHUNK_SIZE_FLOAT,
                        ))),
                    None,
                );
                unsafe {
                    gl::Disable(gl::CULL_FACE);
                }
            }
        }
    }

    fn handle_event(&mut self, _: &mut Glfw, _: &mut glfw::Window, _: &WindowEvent) {}
}

impl VertexAttributes for Vertex {
    fn get_vertex_attributes() -> Vec<(usize, GLuint)> {
        vec![(3, gl::FLOAT), (3, gl::FLOAT), (3, gl::FLOAT)]
    }
}
