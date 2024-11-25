use cgmath::{Point3, Quaternion};
use component::Component;

pub mod component;
mod entity;

pub struct Entity {
    name: String,
    children: Vec<Entity>,
    components: Vec<Box<dyn Component>>,
    position: Point3<f32>,
    rotation: Quaternion<f32>,
}
