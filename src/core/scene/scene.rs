use cgmath::{Matrix4, SquareMatrix};
use glfw::{Glfw, WindowEvent};

use crate::core::{
    entity::{
        component::{camera_component::CameraComponent, Component},
        Entity,
    },
    renderer::{
        framebuffer::FrameBuffer,
        light::skylight::SkyLight,
        texture::{Texture, TextureRenderer},
    },
    window::Window,
};

use super::Scene;

impl Scene {
    pub fn new() -> Self {
        Scene {
            entities: Vec::new(),
            shadow_fbo: None,
            texture_renderer: TextureRenderer::new(),
        }
    }

    pub fn add_shadow_map(&mut self, width: u32, height: u32) {
        let mut fbo = FrameBuffer::new(width, height);
        let texture = Texture::new();
        texture.set_as_depth_texture(width, height);
        fbo.append_depth_texture(texture);
        fbo.depth_only();
        self.shadow_fbo = Some(fbo);
    }

    pub fn update(&mut self, delta_time: f64) {
        for i in 0..self.entities.len() {
            let mut entity = self.entities.remove(i);
            entity.update(self, delta_time);
            self.entities.insert(i, entity);
        }
    }

    pub fn render(&self, window: &Window) {
        let parent_transform = Matrix4::identity();

        // Shadow Pass
        if let Some(shadow_fbo) = &self.shadow_fbo {
            if let Some(skylight) = self.get_component::<SkyLight>() {
                let light_projection = skylight.get_projection();
                shadow_fbo.bind();
                window.clear_mask(gl::DEPTH_BUFFER_BIT);
                for entity in self.entities.iter() {
                    entity.render(self, &light_projection, parent_transform);
                }
                FrameBuffer::unbind();
                window.reset_viewport();
            }
        }

        // Render Pass
        let camera = self.get_component::<CameraComponent>().unwrap();
        let view_projection = camera.get_view_projection();
        if let Some(shadow_fbo) = &self.shadow_fbo {
            if let Some(texture) = &shadow_fbo.get_depth_texture() {
                unsafe {
                    gl::ActiveTexture(gl::TEXTURE0);
                }
                texture.bind();
            }
        }
        for entity in self.entities.iter() {
            entity.render(self, &view_projection, parent_transform);
        }

        // Render Shadow Map
        if let Some(shadow_fbo) = &self.shadow_fbo {
            if let Some(texture) = &shadow_fbo.get_depth_texture() {
                self.texture_renderer.render(texture);
            }
        }
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    pub fn handle_event(
        &mut self,
        glfw: &mut Glfw,
        window: &mut glfw::Window,
        event: &WindowEvent,
    ) {
        for entity in self.entities.iter_mut() {
            entity.handle_event(glfw, window, event);
        }
    }

    pub fn get_component<T>(&self) -> Option<&T>
    where
        T: Component,
    {
        for entity in self.entities.iter() {
            if let Some(component) = entity.get_component::<T>() {
                return Some(component);
            }
        }
        None
    }

    pub fn get_component_mut<T>(&mut self) -> Option<&mut T>
    where
        T: Component,
    {
        for entity in self.entities.iter_mut() {
            if let Some(component) = entity.get_component_mut::<T>() {
                return Some(component);
            }
        }
        None
    }

    // pub fn get_components<T>(&self) -> Vec<&T>
    // where
    //     T: Component,
    // {
    //     let mut components = Vec::new();
    //     for entity in self.entities.iter() {
    //         if let Some(component) = entity.get_component::<T>() {
    //             components.push(component);
    //         }
    //     }
    //     components
    // }

    pub fn get_entities_with_component<T>(&self) -> Vec<&Entity>
    where
        T: Component,
    {
        let mut entities = Vec::new();
        for entity in self.entities.iter() {
            entities.extend(entity.get_with_own_component::<T>());
        }
        entities
    }
}
