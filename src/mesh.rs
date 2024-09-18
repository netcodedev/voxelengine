use gl::types::{GLsizeiptr, GLuint, GLvoid};
use cgmath::{EuclideanSpace, Matrix};
use libnoise::prelude::*;
use ndarray::{ArrayBase, Dim, Array3};

use crate::{camera::{Camera, Projection}, line::Line};

pub const CHUNK_SIZE: usize = 128;

pub struct Mesh {
    vertices: Vec<f32>,
    indices: Vec<u32>,
    normals: Vec<f32>,
    block_type: Vec<u32>,
    vao: u32,
    vbo: u32,
    ebo: u32,
    pub initialized: bool,
}

impl Mesh {
    pub fn new(vertices: Vec<f32>, indices: Vec<u32>, normals: Vec<f32>, block_type: Vec<u32>) -> Self {
        let mesh = Mesh {
            vertices,
            indices,
            normals,
            block_type,
            vao: 0,
            vbo: 0,
            ebo: 0,
            initialized: false,
        };
        mesh
    }

    fn init(&mut self) {
        unsafe {
            // Generate VAO, VBO, and EBO
            gl::GenVertexArrays(1, &mut self.vao);
            gl::GenBuffers(1, &mut self.vbo);
            gl::GenBuffers(1, &mut self.ebo);

            // Bind VAO
            gl::BindVertexArray(self.vao);

            // Bind and fill VBO
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            let vertex_data: Vec<f32> = self.vertices.iter().cloned().chain(self.normals.iter().cloned()).chain(self.block_type.iter().map(|s| *s as f32)).collect();
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertex_data.len() * std::mem::size_of::<f32>()) as GLsizeiptr,
                vertex_data.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );

            // Bind and fill EBO
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (self.indices.len() * std::mem::size_of::<u32>()) as GLsizeiptr,
                self.indices.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );

            // Set vertex attribute pointers
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, std::ptr::null());
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, 0, (self.vertices.len() * std::mem::size_of::<f32>()) as *const GLvoid);
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(2, 1, gl::FLOAT, gl::FALSE, 0, (self.vertices.len() * std::mem::size_of::<f32>() + self.normals.len() * std::mem::size_of::<f32>()) as *const GLvoid);
            gl::EnableVertexAttribArray(2);

            // Unbind VBO and VAO (optional, but good practice)
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }
        self.initialized = true;
    }

    pub fn render(&self, shader_program: GLuint, position: (f32, f32, f32)) {
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::CULL_FACE);
            gl::UseProgram(shader_program);
            let model = cgmath::Matrix4::from_translation(cgmath::Vector3::new(position.0, position.1, position.2));
            let model_loc = gl::GetUniformLocation(shader_program, "model\0".as_ptr() as *const i8);
            gl::UniformMatrix4fv(model_loc, 1, gl::FALSE, model.as_ptr());

            gl::BindVertexArray(self.vao);
            gl::DrawElements(gl::TRIANGLES, self.indices.len() as i32, gl::UNSIGNED_INT, std::ptr::null());
            gl::BindVertexArray(0);
            gl::Disable(gl::CULL_FACE);
            gl::Disable(gl::DEPTH_TEST);
        }
    }
}

pub struct Block {
    pub type_id: u32,
}

impl Block {
    pub fn new(type_id: u32) -> Self {
        Block { type_id }
    }
}

pub struct Chunk {
    position: (f32, f32, f32),
    blocks: ArrayBase<ndarray::OwnedRepr<Option<Block>>, ndarray::Dim<[usize; 3]>>,
    pub mesh: Option<Mesh>,
}

#[derive(Eq, PartialEq, Hash, Debug)]
pub struct ChunkBounds {
    pub min: (i32, i32, i32),
    pub max: (i32, i32, i32),
}

impl ChunkBounds {
    pub fn parse(position: cgmath::Vector3<f32>) -> Self {
        let chunk_pos = (
            (position.x / CHUNK_SIZE as f32).floor() as i32,
            (position.y / CHUNK_SIZE as f32).floor() as i32,
            (position.z / CHUNK_SIZE as f32).floor() as i32,
        );
        let min = (
            chunk_pos.0 * CHUNK_SIZE as i32,
            chunk_pos.1 * CHUNK_SIZE as i32,
            chunk_pos.2 * CHUNK_SIZE as i32,
        );
        let max = (
            (chunk_pos.0 + 1) * CHUNK_SIZE as i32,
            (chunk_pos.1 + 1) * CHUNK_SIZE as i32,
            (chunk_pos.2 + 1) * CHUNK_SIZE as i32,
        );
        ChunkBounds { min, max }
    }

    pub fn contains(&self, position: cgmath::Point3<f32>) -> bool {
        position.x >= self.min.0 as f32 && position.x < self.max.0 as f32 &&
        position.y >= self.min.1 as f32 && position.y < self.max.1 as f32 &&
        position.z >= self.min.2 as f32 && position.z < self.max.2 as f32
    }

    pub fn get_chunk_bounds_on_line(line: &Line) -> Vec<ChunkBounds> {
        let mut bounds = Vec::new();
        let current_chunk = ChunkBounds::parse(line.position.to_vec());
        let step_size = 0.1;
        for i in 0..(line.length / step_size) as i32 {
            let position = line.position + line.direction * (i as f32 * step_size);
            let chunk = ChunkBounds::parse(position.to_vec());
            if current_chunk.contains(position) {
                continue;
            }
            if !bounds.contains(&chunk) {
                bounds.push(chunk);
            }
        }
        if !bounds.contains(&current_chunk) {
            bounds.push(current_chunk);
        }
        bounds
    }
}

impl Chunk {
    pub fn new(position: (f32, f32, f32)) -> Self {
        let generator = Source::perlin(1).scale([0.003; 2]);
        let hills = Source::perlin(1).scale([0.01; 2]);
        let tiny_hills = Source::perlin(1).scale([0.1; 2]);
        let offset: f64 = 16777216.0;
        let blocks: ArrayBase<ndarray::OwnedRepr<Option<Block>>, Dim<[usize; 3]>> = Array3::<Option<Block>>::from_shape_fn([CHUNK_SIZE, CHUNK_SIZE, CHUNK_SIZE], |(x,y,z)| {
            let sample_point = (
                (position.0 * CHUNK_SIZE as f32) as f64 + x as f64 + offset,
                (position.2 * CHUNK_SIZE as f32) as f64 + z as f64 + offset,
            );
            let noise_value = (1.0 + generator.sample([sample_point.0, sample_point.1]))/2.0;
            let hills_value = (1.0 + hills.sample([sample_point.0, sample_point.1]))/2.0 * 0.2;
            let tiny_hills_value = (1.0 + tiny_hills.sample([sample_point.0, sample_point.1]))/2.0 * 0.01;
            if ((noise_value + hills_value + tiny_hills_value) * CHUNK_SIZE as f64) < (y as f64) {
                return None;
            }
            Some(Block::new(1))
        });
        let mut chunk = Chunk { position, blocks, mesh: None };
        chunk.mesh = Some(chunk.calculate_mesh());
        chunk
    }

    pub fn render(&mut self, camera: &Camera, projection: &Projection, shader_program: GLuint) {
        if let Some(mesh) = &mut self.mesh {
            if !mesh.initialized {
                mesh.init();
            }
            unsafe {
                gl::UseProgram(shader_program);
                let view_loc = gl::GetUniformLocation(shader_program, "view\0".as_ptr() as *const i8);
                let projection_loc = gl::GetUniformLocation(shader_program, "projection\0".as_ptr() as *const i8);

                gl::UniformMatrix4fv(view_loc, 1, gl::FALSE, camera.calc_matrix().as_ptr());
                gl::UniformMatrix4fv(projection_loc, 1, gl::FALSE, projection.calc_matrix().as_ptr());
            }
            mesh.render(shader_program, (self.position.0 * CHUNK_SIZE as f32, self.position.1 * CHUNK_SIZE as f32, self.position.2 * CHUNK_SIZE as f32));
        }
    }

    pub fn process_line(&mut self, line: &Line, button: &glfw::MouseButton) -> bool {
        // calculate the block that the line intersects with
        let step_size = 0.1;
        let max_distance = line.length;

        let mut modified = false;
        let mut last_position = (0,0,0);
        for i in 0..(max_distance / step_size) as i32 {
            let position = line.position + line.direction * (i as f32 * step_size);
            // check if position is within the bounds of this chunk
            if position.x < self.position.0 * CHUNK_SIZE as f32 || position.x >= (self.position.0 + 1.0) * CHUNK_SIZE as f32 {
                continue;
            }
            if position.y < self.position.1 * CHUNK_SIZE as f32 || position.y >= (self.position.1 + 1.0) * CHUNK_SIZE as f32 {
                continue;
            }
            if position.z < self.position.2 * CHUNK_SIZE as f32 || position.z >= (self.position.2 + 1.0) * CHUNK_SIZE as f32 {
                continue;
            }
            let block_position = (
                (position.x - self.position.0 * CHUNK_SIZE as f32) as usize,
                (position.y - self.position.1 * CHUNK_SIZE as f32) as usize,
                (position.z - self.position.2 * CHUNK_SIZE as f32) as usize,
            );
            if let Some(block) = self.blocks.get(block_position){
                if block.is_some() {
                    if button == &glfw::MouseButton::Button1 {
                        // println!("(Terrain {},{},{}) Block hit at {:?}", self.position.0, self.position.1, self.position.2, block_position);
                        self.blocks[[block_position.0, block_position.1, block_position.2]] = None;
                        self.mesh = Some(self.calculate_mesh());
                        modified = true;
                        break;
                    }
                    if button == &glfw::MouseButton::Button2 {
                        // println!("(Terrain {},{},{}) Block hit at {:?}", self.position.0, self.position.1, self.position.2, block_position);
                        self.blocks[[last_position.0, last_position.1, last_position.2]] = Some(Block::new(2));
                        self.mesh = Some(self.calculate_mesh());
                        modified = true;
                        break;
                    }
                }
            }
            last_position = block_position;
        }
        modified
    }

    pub fn get_bounds(&self) -> ChunkBounds {
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

    fn calculate_mesh(&self) -> Mesh {
        let mut vertices: Vec<f32> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut normals: Vec<f32> = Vec::new();
        let mut block_type: Vec<u32> = Vec::new();

        // Sweep over each axis (X, Y and Z)
        for d in 0..3 {
            let u = (d + 1) % 3;
            let v = (d + 2) % 3;
            let mut x = vec![0i32; 3];
            let mut q = vec![0i32; 3];

            let mut mask = vec![false; CHUNK_SIZE * CHUNK_SIZE];
            let mut flip = vec![false; CHUNK_SIZE * CHUNK_SIZE];
            let mut b_t = vec![0; CHUNK_SIZE * CHUNK_SIZE];
            q[d] = 1;

            // Check each slice of the chunk one at a time
            x[d] = -1;
            while x[d] < CHUNK_SIZE as i32 {
                // Compute the mask
                let mut n = 0;
                x[v] = 0;
                while x[v] < CHUNK_SIZE as i32 {
                    x[u] = 0;
                    while x[u] < CHUNK_SIZE as i32 {
                        let current_block = self.blocks.get(((x[0]) as usize, (x[1]) as usize, (x[2]) as usize));
                        let current_block_type = if let Some(block) = current_block {
                            if block.is_some() {
                                block.as_ref().unwrap().type_id
                            } else {
                                0
                            }
                        } else {
                            0
                        };
                        let compare_block = self.blocks.get(((x[0] + q[0]) as usize, (x[1] + q[1]) as usize, (x[2] + q[2]) as usize));
                        let compare_block_type = if let Some(block) = compare_block {
                            if block.is_some() {
                                block.as_ref().unwrap().type_id
                            } else {
                                0
                            }
                        } else {
                            0
                        };
                        let block_type = if current_block_type != 0 {
                            current_block_type
                        } else {
                            compare_block_type
                        };
                        let block_current = if 0 <= x[d] {
                            current_block.unwrap().is_none()
                        } else {
                            true
                        };
                        let block_compare = if x[d] < CHUNK_SIZE as i32 - 1 {
                            compare_block.unwrap().is_none()
                        } else {
                            true
                        };
                        mask[n] = block_current != block_compare;
                        flip[n] = block_compare;
                        b_t[n] = block_type;
                        x[u] += 1;
                        n += 1;
                    }
                    x[v] += 1;
                }

                x[d] += 1;

                n = 0;
                
                // Generate a mesh from the mask using lexicographic ordering,
                // by looping over each block in this slice of the chunk
                for j in 0..CHUNK_SIZE {
                    let mut i = 0;
                    while i < CHUNK_SIZE {
                        if mask[n] {
                            // Compute the width of this quad and store it in w
                            // This is done by searching along the current axis until mask[n + w] is false
                            let mut w = 1;
                            while i + w < CHUNK_SIZE && mask[n + w] && flip[n] == flip[n + w] && b_t[n] == b_t[n + w] {
                                w += 1;
                            }

                            // Compute the height of this quad and store it in h
                            // This is done by checking if every block next to this row (range 0 to w) is also part of the mask.
                            // For example, if w is 5 we currently have a quad of dimensions 1 x 5. To reduce triangle count,
                            // greedy meshing will attempt to expand this quad out to CHUNK_SIZE x 5, but will stop if it reaches a hole in the mask
                            let mut h = 1;
                            'outer: while j + h < CHUNK_SIZE {
                                for k in 0..w {
                                    if !mask[n + k + h * CHUNK_SIZE] || flip[n] != flip[n + k + h * CHUNK_SIZE] || b_t[n] != b_t[n + k + h * CHUNK_SIZE] {
                                        break 'outer;
                                    }
                                }
                                h += 1;
                            }

                            x[u] = i as i32;
                            x[v] = j as i32;

                            // du and dv determine the size and orientation of this face
                            let mut du = vec![0; 3];
                            du[u] = w as i32;

                            let mut dv = vec![0; 3];
                            dv[v] = h as i32;

                            // Create a quad for this face. Colour, normal or textures are not stored in this block vertex format.
                            if !flip[n] {
                                vertices.extend_from_slice(&[
                                    (x[0] + du[0]) as f32,          (x[1] + du[1]) as f32,          (x[2] + du[2]) as f32,
                                    x[0] as f32,                    x[1] as f32,                    x[2] as f32,
                                    (x[0] + du[0] + dv[0]) as f32,  (x[1] + du[1] + dv[1]) as f32,  (x[2] + du[2] + dv[2]) as f32,
                                    (x[0] + dv[0]) as f32,          (x[1] + dv[1]) as f32,          (x[2] + dv[2]) as f32,
                                ]);
                            } else {
                                vertices.extend_from_slice(&[
                                    x[0] as f32,                    x[1] as f32,                    x[2] as f32,
                                    (x[0] + du[0]) as f32,          (x[1] + du[1]) as f32,          (x[2] + du[2]) as f32,
                                    (x[0] + dv[0]) as f32,          (x[1] + dv[1]) as f32,          (x[2] + dv[2]) as f32,
                                    (x[0] + du[0] + dv[0]) as f32,  (x[1] + du[1] + dv[1]) as f32,  (x[2] + du[2] + dv[2]) as f32,
                                ]);
                            }

                            let vert_count = vertices.len() as u32 / 3;
                            indices.extend_from_slice(&[
                                vert_count - 4, vert_count - 3, vert_count - 2,
                                vert_count - 2, vert_count - 3, vert_count - 1,
                            ]);

                            match d {
                                0 => normals.extend(vec![
                                    0.0, 1.0, 0.0,
                                    0.0, 1.0, 0.0,
                                    0.0, 1.0, 0.0,
                                    0.0, 1.0, 0.0,
                                ]),
                                1 => normals.extend(vec![
                                    1.0, 0.0, 0.0,
                                    1.0, 0.0, 0.0,
                                    1.0, 0.0, 0.0,
                                    1.0, 0.0, 0.0,
                                ]),
                                2 => normals.extend(vec![
                                    0.0, 0.0, 1.0,
                                    0.0, 0.0, 1.0,
                                    0.0, 0.0, 1.0,
                                    0.0, 0.0, 1.0,
                                ]),
                                _ => (),
                            }

                            block_type.extend(vec![
                                b_t[n], b_t[n], b_t[n], b_t[n],
                            ]);

                            // Clear this part of the mask, so we don't add duplicate faces
                            for l in 0..h {
                                for k in 0..w {
                                    mask[n + k + l * CHUNK_SIZE] = false;
                                }
                            }

                            // Increment counters and continue
                            i += w;
                            n += w;
                        } else {
                            i += 1;
                            n += 1;
                        }
                    }
                }
            }
        }

        Mesh::new(vertices, indices, normals, block_type)
    }
}