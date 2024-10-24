use std::collections::HashMap;

use cgmath::Matrix4;
use russimp::{material::TextureType, scene::Scene};

use crate::{
    shader::{DynamicVertexArray, Shader},
    texture::Texture,
};

pub mod model;

pub struct Model {
    model: Scene,
    meshes: HashMap<String, ModelMesh>,
    animations: HashMap<String, Animation>,
    current_animation: Option<Animation>,
    shader: Shader,
    textures: HashMap<TextureType, Texture>,
    position: cgmath::Vector3<f32>,
    scale: f32,
}

#[derive(Debug, Clone)]
#[repr(C)]
struct ModelMeshVertex {
    position: (f32, f32, f32),
    normal: (f32, f32, f32),
    texture_coords: (f32, f32),
    bone_ids: (u32, u32, u32, u32),
    bone_weights: (f32, f32, f32, f32),
}

struct ModelMesh {
    vertex_array: Option<DynamicVertexArray<ModelMeshVertex>>,
    indices: Vec<u32>,
    vertices: Vec<ModelMeshVertex>,
    root_bone: Option<Bone>,
}

#[allow(dead_code)]
#[derive(Clone)]
struct Bone {
    id: usize,
    name: String,
    transformation_matrix: Matrix4<f32>,
    offset_matrix: Matrix4<f32>,
    weights: Vec<(u32, f32)>,
    children: Option<Vec<Bone>>,
    current_animation: Option<Channel>,
    current_animation_time: f32,
    current_transform: Matrix4<f32>,
}

#[allow(dead_code)]
#[derive(Clone)]
struct Animation {
    name: String,
    duration: f32,
    ticks_per_second: f32,
    channels: HashMap<String, Channel>,
}

#[allow(dead_code)]
#[derive(Clone)]
struct Channel {
    bone_id: String,
    position_keys: Vec<(f32, cgmath::Vector3<f32>)>,
    rotation_keys: Vec<(f32, cgmath::Quaternion<f32>)>,
    scaling_keys: Vec<(f32, cgmath::Vector3<f32>)>,
}
