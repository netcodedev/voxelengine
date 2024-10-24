use cgmath::{EuclideanSpace, InnerSpace, Point3, Vector3};
use gl::types::GLuint;
use glfw::MouseButton;
use libnoise::prelude::*;

use crate::{
    camera::{Camera, Projection},
    line::Line,
    shader::{Shader, VertexAttributes},
    terrain::{Chunk, ChunkBounds},
};

use super::{ChunkMesh, DualContouringChunk, Vertex, CHUNK_SIZE, CHUNK_SIZE_FLOAT, ISO_VALUE};

impl DualContouringChunk {
    fn get_density_at(&self, (x, y, z): (usize, usize, usize)) -> f32 {
        let offset: f64 = 16777216.0;
        let sample_point = (
            (self.position.0 * CHUNK_SIZE_FLOAT) as f64 + x as f64 + offset,
            (self.position.1 * CHUNK_SIZE_FLOAT) as f64 + y as f64 + offset,
            (self.position.2 * CHUNK_SIZE_FLOAT) as f64 + z as f64 + offset,
        );

        let noise_value = (1.0 + self.noises[0].sample([sample_point.0, sample_point.2])) / 2.0;
        let hills_value =
            (1.0 + self.noises[1].sample([sample_point.0, sample_point.2])) / 2.0 * 0.2;
        let tiny_hills_value =
            (1.0 + self.noises[2].sample([sample_point.0, sample_point.2])) / 2.0 * 0.01;
        let height =
            ((noise_value + hills_value + tiny_hills_value) as f32 * CHUNK_SIZE_FLOAT) - y as f32;
        let iso = (1.0
            + self
                .cave
                .sample([sample_point.0, sample_point.1, sample_point.2]) as f32)
            / 2.0;
        let height_iso = (height as f32 * CHUNK_SIZE_FLOAT) - y as f32;
        height_iso - iso
    }

    fn generate_mesh(&self) -> ChunkMesh<Vertex> {
        let mut vertices = Vec::<Vertex>::new();
        let mut indices = Vec::<u32>::new();
        let mut vertex_grid =
            vec![vec![vec![false; self.chunk_size + 1]; self.chunk_size + 1]; self.chunk_size + 1];
        let mut index_grid =
            vec![vec![vec![0; self.chunk_size + 1]; self.chunk_size + 1]; self.chunk_size + 1];
        let mut index: u32 = 0;
        let size_multiplier = CHUNK_SIZE / self.chunk_size;
        for x in 0..self.chunk_size + 1 {
            for y in 0..self.chunk_size + 1 {
                for z in 0..self.chunk_size + 1 {
                    if self.is_surface_voxel((
                        x * size_multiplier,
                        y * size_multiplier,
                        z * size_multiplier,
                    )) {
                        let mut corners: [(Point3<f32>, f32); 8] =
                            [(Point3::new(0.0, 0.0, 0.0), ISO_VALUE); 8];
                        for i in 0..8 {
                            let x_add = i & 1;
                            let y_add = (i >> 1) & 1;
                            let z_add = (i >> 2) & 1;
                            let x_n = x + x_add;
                            let y_n = y + y_add;
                            let z_n = z + z_add;
                            corners[i] = (
                                Point3::new(x_add as f32, y_add as f32, z_add as f32),
                                self.get_density_at((
                                    x_n * size_multiplier,
                                    y_n * size_multiplier,
                                    z_n * size_multiplier,
                                )),
                            );
                        }
                        let position = self.calculate_vertex_position(
                            (
                                x * size_multiplier,
                                y * size_multiplier,
                                z * size_multiplier,
                            ),
                            &corners,
                        );
                        let normal = DualContouringChunk::calculate_gradient(&corners, position);

                        let vertex = Vertex {
                            position: [position.x, position.y, position.z],
                            normal: [normal.x, normal.y, normal.z],
                            color: [0.0, 0.5, 0.1],
                        };
                        index_grid[x][y][z] = index;
                        vertices.push(vertex);
                        vertex_grid[x][y][z] = true;
                        if x > 0 && vertex_grid[x - 1][y][z] {
                            if y > 0 && vertex_grid[x - 1][y - 1][z] {
                                indices.push(index);
                                indices.push(index_grid[x - 1][y][z] as u32);
                                indices.push(index_grid[x - 1][y - 1][z] as u32);

                                if vertex_grid[x][y - 1][z] {
                                    indices.push(index);
                                    indices.push(index_grid[x][y - 1][z] as u32);
                                    indices.push(index_grid[x - 1][y - 1][z] as u32);
                                }
                            }
                            if z > 0 && vertex_grid[x - 1][y][z - 1] {
                                indices.push(index);
                                indices.push(index_grid[x - 1][y][z - 1] as u32);
                                indices.push(index_grid[x - 1][y][z] as u32);

                                if vertex_grid[x][y][z - 1] {
                                    indices.push(index);
                                    indices.push(index_grid[x][y][z - 1] as u32);
                                    indices.push(index_grid[x - 1][y][z - 1] as u32);
                                }
                            }
                        }
                        if y > 0 && vertex_grid[x][y - 1][z] {
                            if z > 0 && vertex_grid[x][y - 1][z - 1] {
                                indices.push(index);
                                indices.push(index_grid[x][y - 1][z] as u32);
                                indices.push(index_grid[x][y - 1][z - 1] as u32);

                                if vertex_grid[x][y][z - 1] {
                                    indices.push(index);
                                    indices.push(index_grid[x][y][z - 1] as u32);
                                    indices.push(index_grid[x][y - 1][z - 1] as u32);
                                }
                            }
                        }
                        index += 1;
                    }
                }
            }
        }
        // println!("Generated chunk with {} vertices and {} indices", vertices.len(), indices.len());
        ChunkMesh::new(vertices, Some(indices))
    }

    fn calculate_chunk_size(lod: usize) -> usize {
        std::cmp::max(
            2,
            std::cmp::min(CHUNK_SIZE, CHUNK_SIZE / 2usize.pow(lod as u32 / 2)),
        )
    }

    fn calculate_vertex_position(
        &self,
        position: (usize, usize, usize),
        corners: &[(Point3<f32>, f32)],
    ) -> Point3<f32> {
        let mut v_pos = Point3::new(position.0 as f32, position.1 as f32, position.2 as f32);
        let relative_coordinates = DualContouringChunk::calculate_relative_coordinates(&corners);

        v_pos.x += relative_coordinates.x;
        v_pos.y += relative_coordinates.y;
        v_pos.z += relative_coordinates.z;

        v_pos
    }

    fn interpolate(p1: (Point3<f32>, f32), p2: (Point3<f32>, f32)) -> Point3<f32> {
        let t = (ISO_VALUE - p1.1) / (p2.1 - p1.1);
        p1.0 + (p2.0 - p1.0) * t
    }

    fn find_crossing_edges(
        vertices: &[(Point3<f32>, f32)],
    ) -> Vec<((Point3<f32>, f32), (Point3<f32>, f32))> {
        let mut crossing_edges = Vec::new();
        for (i, p1) in vertices.iter().enumerate() {
            for p2 in vertices.iter().skip(i + 1) {
                if (p1.1 <= ISO_VALUE && ISO_VALUE <= p2.1)
                    || (p2.1 <= ISO_VALUE && ISO_VALUE <= p1.1)
                {
                    crossing_edges.push((*p1, *p2));
                }
            }
        }
        crossing_edges
    }

    fn calculate_relative_coordinates(vertices: &[(Point3<f32>, f32)]) -> Point3<f32> {
        let crossing_edges = DualContouringChunk::find_crossing_edges(vertices);
        let interpolated_points: Vec<Point3<f32>> = crossing_edges
            .iter()
            .map(|&edge| DualContouringChunk::interpolate(edge.0, edge.1))
            .collect();

        // Berechne den Schwerpunkt der interpolierten Punkte
        let center_of_mass = interpolated_points
            .iter()
            .fold(Vector3::new(0.0, 0.0, 0.0), |acc, &p| acc + p.to_vec())
            / (interpolated_points.len() as f32);

        Point3::from_vec(center_of_mass)
    }

    fn calculate_corner_gradients(
        vertices: &[(Point3<f32>, f32)],
    ) -> Vec<(Point3<f32>, Vector3<f32>)> {
        let mut corner_gradients = Vec::new();
        for (i, &(point, value)) in vertices.iter().enumerate() {
            let mut gradient = Vector3::new(0.0, 0.0, 0.0);
            for j in 0..vertices.len() {
                if i != j {
                    let other_point = vertices[j].0;
                    let other_value = vertices[j].1;
                    let direction = other_point - point;
                    let distance = direction.magnitude();
                    if distance > 0.0 {
                        gradient += direction * (other_value - value) / distance.powi(2);
                    }
                }
            }
            corner_gradients.push((point, gradient.normalize()));
        }
        corner_gradients
    }

    fn calculate_gradient(vertices: &[(Point3<f32>, f32)], point: Point3<f32>) -> Vector3<f32> {
        let corner_gradients = DualContouringChunk::calculate_corner_gradients(vertices);

        let mut gradient = Vector3::new(0.0, 0.0, 0.0);
        for i in 0..8 {
            let c = corner_gradients[i].1;
            let p = corner_gradients[i].0;
            let weight = (1.0 - (point.x - p.x).abs())
                * (1.0 - (point.y - p.y).abs())
                * (1.0 - (point.z - p.z).abs());
            gradient += c * weight;
        }

        gradient.normalize()
    }

    fn is_surface_voxel(&self, position: (usize, usize, usize)) -> bool {
        let mut corners = [0.0; 8];
        let size_multiplier = CHUNK_SIZE / self.chunk_size;
        for i in 0..8 {
            let x = position.0 + ((i & 1) * size_multiplier);
            let y = position.1 + (((i >> 1) & 1) * size_multiplier);
            let z = position.2 + (((i >> 2) & 1) * size_multiplier);
            corners[i] = self.get_density_at((x, y, z));
        }
        let mut cube_index = 0;
        for i in 0..8 {
            if corners[i] < ISO_VALUE {
                cube_index |= 1 << i;
            }
        }
        cube_index != 0 && cube_index != 255
    }
}

impl Chunk for DualContouringChunk {
    fn new(position: (f32, f32, f32), lod: usize) -> Self {
        let noises = [
            Source::perlin(1).scale([0.003; 2]),
            Source::perlin(1).scale([0.01; 2]),
            Source::perlin(1).scale([0.1; 2]),
        ];
        let cave = Source::perlin(1).scale([0.1; 3]);
        let mut chunk = Self {
            position,
            cave,
            noises,
            chunk_size: DualContouringChunk::calculate_chunk_size(lod),
            mesh: None,
        };
        chunk.mesh = Some(chunk.generate_mesh());
        chunk
    }

    fn render(&mut self, camera: &Camera, projection: &Projection, shader: &Shader) {
        if let Some(mesh) = &mut self.mesh {
            if !mesh.is_buffered() {
                mesh.buffer_data();
            }
            shader.bind();
            shader.set_uniform_mat4("view", &camera.calc_matrix());
            shader.set_uniform_mat4("projection", &projection.calc_matrix());
            unsafe {
                gl::Disable(gl::CULL_FACE);
            }
            mesh.render(
                &shader,
                (
                    self.position.0 * CHUNK_SIZE as f32,
                    self.position.1 * CHUNK_SIZE as f32,
                    self.position.2 * CHUNK_SIZE as f32,
                ),
                None,
            );
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

    fn get_shader_source() -> (String, String) {
        (
            include_str!("vertex.glsl").to_string(),
            include_str!("fragment.glsl").to_string(),
        )
    }

    fn get_textures() -> Vec<crate::texture::Texture> {
        Vec::new()
    }
}

impl VertexAttributes for Vertex {
    fn get_vertex_attributes() -> Vec<(usize, GLuint)> {
        vec![(3, gl::FLOAT), (3, gl::FLOAT), (3, gl::FLOAT)]
    }
}
