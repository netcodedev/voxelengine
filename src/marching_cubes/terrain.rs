use std::{collections::HashMap, sync::mpsc, thread};

use crate::{camera::{Camera, Projection, ViewFrustum}, marching_cubes::MarchingCubesChunk, shader::Shader, terrain::{Chunk, ChunkBounds, Terrain}};

use super::MarchingCubesTerrain;

impl MarchingCubesTerrain {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        let origin = MarchingCubesChunk::new((0.0, 0.0, 0.0));
        tx.send(origin).unwrap();

        let shader = Shader::new(include_str!("vertex.glsl"), include_str!("fragment.glsl"));

        let tx1 = tx.clone();
        let tx2 = tx.clone();
        let tx3 = tx.clone();
        let tx4 = tx.clone();
        const RADIUS: i32 = 5;
        let _ = thread::spawn(move || chunkloader(RADIUS,1,1,tx1));
        let _ = thread::spawn(move || chunkloader(RADIUS,-1,1,tx2));
        let _ = thread::spawn(move || chunkloader(RADIUS,1,-1,tx3));
        let _ = thread::spawn(move || chunkloader(RADIUS,-1,-1,tx4));

        Self {
            chunks: HashMap::<ChunkBounds, MarchingCubesChunk>::new(),
            chunk_receiver: rx,
            shader,
        }
    }

}

impl Terrain for MarchingCubesTerrain {
    fn update(&mut self) {
        if let Ok(chunk) = self.chunk_receiver.try_recv() {
            self.chunks.insert(chunk.get_bounds(), chunk);
        }
    }

    fn render(&mut self, camera: &Camera, projection: &Projection) {
        for (_, chunk) in &mut self.chunks {
            if ViewFrustum::is_bounds_in_frustum(projection, camera, chunk.get_bounds()) {
                chunk.render(camera, projection, &self.shader);
            }
        }
    }
    
    fn process_line(&mut self, _: Option<(crate::line::Line, glfw::MouseButton)>) {
        
    }
}

fn chunkloader(radius: i32, x_dir: i32, z_dir: i32, tx: mpsc::Sender<MarchingCubesChunk>) {
    let mut x: i32 = 1;
    let mut z: i32 = 0;

    loop {
        if x > radius {
            break;
        }
        let new_chunk: MarchingCubesChunk;
        if z_dir > 0 {
            new_chunk = MarchingCubesChunk::new(((x * x_dir) as f32, 0.0, z as f32));
        } else {
            new_chunk = MarchingCubesChunk::new(((z * z_dir) as f32, 0.0, (x * x_dir) as f32));
        }
        
        let result = tx.send(new_chunk);
        if result.is_err() {
            break;
        }

        z = -z;
        if z == -x*z_dir {
            x += 1;
            z = 0;
        } else if z >= 0 {
            z += 1;
        }
    }
}